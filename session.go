package main

import (
	"bytes"
	"encoding/json"
	"fmt"
	"io"
	"net/http"
	"os"
	"path/filepath"
	"regexp"
	"sort"
	"strconv"
	"strings"
	"time"
)

var (
	// Fixed regex patterns to properly capture the command content
	pandocRE = regexp.MustCompile(`(?m)` +
		`\x60\x60\x60\{\.(?:shell|bash|sh|powershell|ps1)(?:[^\}]*?)data-action="execute"[^\}]*\}\s*\n([\s\S]*?)\x60\x60\x60`)

	// Enhanced fallback regex to capture commands in regular code blocks
	fallbackRE = regexp.MustCompile(`(?m)(?:\x60\x60\x60(?:shell|bash|sh|powershell|ps1|)\s*\n|\x60\x60\x60\s*\n)([\s\S]*?)\x60\x60\x60`)
)

// PrimeSession represents a session with the Prime assistant
type PrimeSession struct {
	// Base paths
	BaseDir    string
	SessionID  string
	SessionDir string

	// Message tracking
	messageCounter uint

	// Ollama configuration
	OllamaModel  string
	OllamaAPIURL string

	// Components
	CommandProcessor CommandProcessor
	MemoryManager    MemoryManager

	// HTTP client
	client *http.Client
}

// NewPrimeSession creates a new Prime session
func NewPrimeSession(baseDir, ollamaModel, ollamaAPI string) (*PrimeSession, error) {
	// Create session ID with timestamp
	sessionID := fmt.Sprintf("session_%s", time.Now().Format("20060102_150405"))

	// Create required directories
	sessionDir := filepath.Join(baseDir, "conversations", sessionID)
	if err := os.MkdirAll(sessionDir, 0755); err != nil {
		return nil, fmt.Errorf("failed to create session directory: %v", err)
	}

	// Create memory directory if it doesn't exist
	memoryDir := filepath.Join(baseDir, "memory")
	if err := os.MkdirAll(memoryDir, 0755); err != nil {
		return nil, fmt.Errorf("failed to create memory directory: %v", err)
	}

	// Initialize memory files if they don't exist
	for _, memoryFile := range []string{"long_term.md", "short_term.md"} {
		filePath := filepath.Join(memoryDir, memoryFile)
		if _, err := os.Stat(filePath); os.IsNotExist(err) {
			header := fmt.Sprintf("# Prime %s Memory\n\n",
				map[string]string{
					"long_term.md":  "Long-term",
					"short_term.md": "Short-term",
				}[memoryFile])
			if err := os.WriteFile(filePath, []byte(header), 0644); err != nil {
				return nil, fmt.Errorf("failed to create memory file %s: %v", memoryFile, err)
			}
		}
	}

	// Create HTTP client
	client := &http.Client{
		Timeout: 60 * time.Second,
	}

	// Initialize session
	session := &PrimeSession{
		BaseDir:          baseDir,
		SessionID:        sessionID,
		SessionDir:       sessionDir,
		messageCounter:   0,
		OllamaModel:      ollamaModel,
		OllamaAPIURL:     fmt.Sprintf("%s/api/generate", strings.TrimRight(ollamaAPI, "/")),
		CommandProcessor: NewCommandProcessor(),
		MemoryManager:    NewMemoryManager(filepath.Join(baseDir, "memory")),
		client:           client,
	}

	return session, nil
}

// nextMessageNumber gets the next sequential message number
func (s *PrimeSession) nextMessageNumber() uint {
	s.messageCounter++
	return s.messageCounter
}

// AddUserMessage adds a user message to the conversation
func (s *PrimeSession) AddUserMessage(content string) error {
	messageNumber := s.nextMessageNumber()
	fileName := fmt.Sprintf("%03d_user.md", messageNumber)
	filePath := filepath.Join(s.SessionDir, fileName)

	timestamp := time.Now().Format("2006-01-02 15:04:05")
	messageContent := fmt.Sprintf("# User Message\nTimestamp: %s\n\n%s", timestamp, content)

	if err := os.WriteFile(filePath, []byte(messageContent), 0644); err != nil {
		return fmt.Errorf("failed to write user message: %v", err)
	}
	return nil
}

// AddPrimeMessage adds a Prime (AI) message to the conversation
func (s *PrimeSession) AddPrimeMessage(content string) error {
	messageNumber := s.nextMessageNumber()
	fileName := fmt.Sprintf("%03d_prime.md", messageNumber)
	filePath := filepath.Join(s.SessionDir, fileName)

	timestamp := time.Now().Format("2006-01-02 15:04:05")
	messageContent := fmt.Sprintf("# Prime Response\nTimestamp: %s\n\n%s", timestamp, content)

	if err := os.WriteFile(filePath, []byte(messageContent), 0644); err != nil {
		return fmt.Errorf("failed to write prime message: %v", err)
	}
	return nil
}

// AddSystemMessage adds a system message to the conversation (command output)
func (s *PrimeSession) AddSystemMessage(command string, exitCode int, output string) error {
	messageNumber := s.nextMessageNumber()
	fileName := fmt.Sprintf("%03d_system.md", messageNumber)
	filePath := filepath.Join(s.SessionDir, fileName)

	timestamp := time.Now().Format("2006-01-02 15:04:05")
	messageContent := fmt.Sprintf(
		"# System Output\nTimestamp: %s\nCommand: %s\nExit Code: %d\n\n```\n%s\n```",
		timestamp, command, exitCode, output)

	if err := os.WriteFile(filePath, []byte(messageContent), 0644); err != nil {
		return fmt.Errorf("failed to write system message: %v", err)
	}
	return nil
}

// GeneratePrimeResponse generates a response from Prime using the LLM with streaming
func (s *PrimeSession) GeneratePrimeResponse(currentTurnPrompt string, isErrorCorrectionTurn bool) (string, error) {
	var ollamaPromptPayload strings.Builder
	var fullResponse strings.Builder
	lastLineLength := 0

	// Build the prompt
	systemPrompt, err := s.getSystemPrompt()
	if err != nil {
		return "", fmt.Errorf("failed to get system prompt: %v", err)
	}
	ollamaPromptPayload.WriteString(systemPrompt)
	ollamaPromptPayload.WriteString("\n\n")

	historyLimit := uint(10)
	conversationHistory, err := s.getFullConversationHistoryPrompt(historyLimit)
	if err != nil {
		return "", fmt.Errorf("failed to get conversation history: %v", err)
	}
	if conversationHistory != "" {
		ollamaPromptPayload.WriteString("## Recent Conversation History:\n")
		ollamaPromptPayload.WriteString(conversationHistory)
	}

	if isErrorCorrectionTurn {
		ollamaPromptPayload.WriteString("## Error Correction Task:\n")
	} else {
		ollamaPromptPayload.WriteString("## Current User Request:\n")
	}
	ollamaPromptPayload.WriteString(currentTurnPrompt)
	ollamaPromptPayload.WriteString("\n\n# Prime Response:\n")

	// Prepare streaming request
	requestBody := map[string]interface{}{
		"model":  s.OllamaModel,
		"prompt": ollamaPromptPayload.String(),
		"stream": true,
		"options": map[string]interface{}{
			"temperature": 0.5,
			"top_p":       0.9,
		},
	}

	requestJSON, err := json.Marshal(requestBody)
	if err != nil {
		return "", fmt.Errorf("failed to marshal request body: %v", err)
	}

	resp, err := s.client.Post(s.OllamaAPIURL, "application/json", bytes.NewBuffer(requestJSON))
	if err != nil {
		return "", fmt.Errorf("failed to send request to Ollama API: %v", err)
	}
	defer resp.Body.Close()

	if resp.StatusCode != http.StatusOK {
		bodyBytes, _ := io.ReadAll(resp.Body)
		return "", fmt.Errorf("Ollama API error (%d): %s", resp.StatusCode, string(bodyBytes))
	}

	// Process streaming response
	decoder := json.NewDecoder(resp.Body)
	fmt.Print("\n") // Start response on new line

	for {
		var streamResponse struct {
			Response string `json:"response"`
			Done     bool   `json:"done"`
		}

		if err := decoder.Decode(&streamResponse); err != nil {
			if err == io.EOF {
				break
			}
			return "", fmt.Errorf("failed to decode stream: %v", err)
		}

		// Clear current line and print new content
		if lastLineLength > 0 {
			fmt.Printf("\r%s\r", strings.Repeat(" ", lastLineLength))
		}

		fullResponse.WriteString(streamResponse.Response)
		currentText := fullResponse.String()
		fmt.Print(currentText[len(currentText)-len(streamResponse.Response):])
		lastLineLength = len(streamResponse.Response)

		if streamResponse.Done {
			break
		}
	}

	generatedText := strings.TrimSpace(fullResponse.String())
	fmt.Print("\n") // End response with newline

	// Save the AI response
	if err := s.AddPrimeMessage(generatedText); err != nil {
		return "", fmt.Errorf("failed to save AI response: %v", err)
	}

	return generatedText, nil
}

// getFullConversationHistoryPrompt gets a string representation of the recent conversation history
func (s *PrimeSession) getFullConversationHistoryPrompt(limit uint) (string, error) {
	messages, err := s.GetMessages(limit)
	if err != nil {
		return "", err
	}

	var contextStr strings.Builder
	for _, message := range messages {
		contextStr.WriteString(message.Content)
		contextStr.WriteString("\n\n")
	}

	return contextStr.String(), nil
}

// getSystemPrompt gets system prompt for Prime
func (s *PrimeSession) getSystemPrompt() (string, error) {
	memory, err := s.MemoryManager.ReadMemory("")
	if err != nil {
		return "", fmt.Errorf("failed to read memory: %v", err)
	}

	const systemPromptTemplate = `# Prime System Instructions

You are Prime, an advanced terminal development environment that helps users write, test, and manage code.
You can execute shell commands by including them in properly formatted Pandoc attributed markdown code blocks.

## Communication Guidelines
- Respond in a clear, concise, professional manner
- When suggesting actions, provide specific commands in code blocks with proper Pandoc attributes
- After complex operations, summarize what was done
- If code changes are needed, explain the changes being made
- Focus on delivering working solutions rather than explanations

## Command Execution
When you want to run a shell command, include it in a code block with Pandoc attributes like this:
` + "```{.powershell data-action=\"execute\"}" + `
Get-Date  # Example command
` + "```" + `

The system will automatically execute these commands and capture their output.
Wait for command results before continuing with multi-step processes.

## Memory Context
The following represents your current memory about the project and environment:

%s

## Guidelines
- Break complex tasks into smaller, logical steps
- Always verify command results before proceeding
- Be cautious with destructive operations
- Validate system state when needed
- Use proper Pandoc markdown format for code blocks
- Target Windows PowerShell environment
- Prefer PowerShell commands over Unix/Linux equivalents
- Write clean, maintainable code
- Follow project conventions and patterns
- Add appropriate error handling
- Consider performance implications
`

	return fmt.Sprintf(systemPromptTemplate, memory), nil
}

// ProcessCommands processes any commands in Prime's response
func (s *PrimeSession) ProcessCommands(response string) ([]CommandExecutionResult, error) {
	var results []CommandExecutionResult

	// Extract commands using both patterns
	var matches [][]string
	matches = pandocRE.FindAllStringSubmatch(response, -1)

	// If no Pandoc blocks found, try fallback pattern
	if len(matches) == 0 {
		matches = fallbackRE.FindAllStringSubmatch(response, -1)
	}

	// Process all found commands
	for _, match := range matches {
		if len(match) < 2 {
			continue
		}

		commandStr := strings.TrimSpace(match[1])
		if commandStr == "" {
			continue
		}

		// Check for dangerous commands
		if s.CommandProcessor.IsDestructiveCommand(commandStr) {
			return nil, fmt.Errorf("refusing to execute potentially destructive command: %s", commandStr)
		}

		exitCode, output, err := s.CommandProcessor.ExecuteCommand(commandStr)
		if err != nil {
			if logErr := s.AddSystemMessage(commandStr, -1, fmt.Sprintf("Error: %v", err)); logErr != nil {
				return nil, fmt.Errorf("command failed and logging failed: %v, log error: %v", err, logErr)
			}
			continue
		}

		if err := s.AddSystemMessage(commandStr, exitCode, output); err != nil {
			return nil, fmt.Errorf("failed to log command output: %v", err)
		}

		results = append(results, CommandExecutionResult{
			Command:  commandStr,
			ExitCode: exitCode,
			Output:   output,
			Success:  exitCode == 0,
		})
	}

	return results, nil
}

// GetMessages gets list of messages in the session
func (s *PrimeSession) GetMessages(limit uint) ([]Message, error) {
	entries, err := os.ReadDir(s.SessionDir)
	if err != nil {
		return nil, fmt.Errorf("failed to read session directory: %v", err)
	}

	var messages []Message
	for _, entry := range entries {
		if entry.IsDir() {
			continue
		}

		fileName := entry.Name()
		if !strings.HasSuffix(fileName, ".md") {
			continue
		}

		parts := strings.SplitN(fileName, "_", 2)
		if len(parts) != 2 {
			continue
		}

		number, err := parseMessageNumber(parts[0])
		if err != nil {
			continue
		}

		msgType := strings.TrimSuffix(parts[1], ".md")
		filePath := filepath.Join(s.SessionDir, fileName)
		content, err := os.ReadFile(filePath)
		if err != nil {
			return nil, fmt.Errorf("failed to read message file %s: %v", fileName, err)
		}

		messages = append(messages, Message{
			Number:  number,
			Type:    msgType,
			Path:    filePath,
			Content: string(content),
		})
	}

	// Sort by message number
	sort.Slice(messages, func(i, j int) bool {
		return messages[i].Number < messages[j].Number
	})

	// Apply limit if provided and if there are more messages than the limit
	if limit > 0 && uint(len(messages)) > limit {
		// Take the most recent messages
		start := uint(len(messages)) - limit
		messages = messages[start:]
	}

	return messages, nil
}

// ListMessages lists messages in the current session
func (s *PrimeSession) ListMessages() ([]string, error) {
	messages, err := s.GetMessages(0) // 0 means no limit
	if err != nil {
		return nil, err
	}

	var result []string
	for _, message := range messages {
		// Extract first line of content for summary
		firstLine := "[Empty message]"
		lines := strings.Split(message.Content, "\n")
		for _, line := range lines {
			if strings.TrimSpace(line) != "" {
				firstLine = line
				break
			}
		}

		result = append(result, fmt.Sprintf("%03d - %s: %s",
			message.Number, message.Type, firstLine))
	}

	return result, nil
}

// ReadMessage reads a specific message by number
func (s *PrimeSession) ReadMessage(number uint) (string, error) {
	// Format number with leading zeros
	fileName := fmt.Sprintf("%03d_*.md", number)
	matches, err := filepath.Glob(filepath.Join(s.SessionDir, fileName))
	if err != nil {
		return "", fmt.Errorf("failed to search for message file: %v", err)
	}

	if len(matches) == 0 {
		return "", fmt.Errorf("message %d not found", number)
	}

	content, err := os.ReadFile(matches[0])
	if err != nil {
		return "", fmt.Errorf("failed to read message file: %v", err)
	}

	return string(content), nil
}

// ReadMemory reads memory (wrapper for memory manager)
func (s *PrimeSession) ReadMemory(memoryType string) (string, error) {
	return s.MemoryManager.ReadMemory(memoryType)
}

func parseMessageNumber(s string) (uint, error) {
	num, err := parseUint(s)
	if err != nil {
		return 0, fmt.Errorf("invalid message number format: %v", err)
	}
	return num, nil
}

func parseUint(s string) (uint, error) {
	num, err := strconv.ParseUint(strings.TrimSpace(s), 10, 32)
	if err != nil {
		return 0, err
	}
	return uint(num), nil
}
