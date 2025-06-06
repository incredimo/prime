package main

import (
	"bufio"
	"fmt"
	"io"
	"os"
	"os/signal"
	"path/filepath"
	"strconv"
	"strings"
	"syscall"

	"github.com/fatih/color"
)

const (
	AppName      = "Prime"
	Version      = "1.0.0"
	bannerWidth  = 70
	contentWidth = 50
)

type Prime struct {
	session *PrimeSession
}

func main() {
	// Clear screen and show banner
	fmt.Print("\033[2J\033[H")
	bar := strings.Repeat("─", bannerWidth)

	fmt.Println(color.BlueString(bar))
	fmt.Printf("  %s %s | %s\n",
		color.New(color.FgBlue).Add(color.Bold).Sprint(AppName),
		color.New(color.Faint).Sprint(fmt.Sprintf("v%s", Version)),
		color.New(color.Faint).Sprint("Development Environment"))
	fmt.Println(color.BlueString(bar))

	// Initialize session
	prime, err := initPrime()
	if err != nil {
		color.Red("[ERROR] %v", err)
		os.Exit(1)
	}

	// Setup graceful shutdown
	signalChan := make(chan os.Signal, 1)
	signal.Notify(signalChan, os.Interrupt, syscall.SIGTERM)

	go func() {
		<-signalChan
		fmt.Print("\n")
		color.Yellow("Shutting down...")
		os.Exit(0)
	}()

	// Start main loop
	if err := prime.run(); err != nil {
		color.Red("\n[ERROR] %v", err)
		os.Exit(1)
	}
}

func initPrime() (*Prime, error) {
	// Get configuration
	ollamaModel := getEnvOrDefault("OLLAMA_MODEL", "gemma3:latest")
	ollamaAPI := getEnvOrDefault("OLLAMA_API", "http://localhost:11434")

	// Setup directories
	homeDir, err := os.UserHomeDir()
	if err != nil {
		return nil, fmt.Errorf("failed to get home directory: %v", err)
	}
	baseDir := filepath.Join(homeDir, ".prime")

	// Display settings
	fmt.Printf("\nConfiguration:\n")
	fmt.Printf("  Model:      %s\n", color.CyanString(ollamaModel))
	fmt.Printf("  API:        %s\n", color.CyanString(ollamaAPI))
	fmt.Printf("  Data Path:  %s\n", color.CyanString(baseDir))
	fmt.Println(color.BlueString(strings.Repeat("─", bannerWidth)))

	// Initialize session
	session, err := NewPrimeSession(baseDir, ollamaModel, ollamaAPI)
	if err != nil {
		return nil, fmt.Errorf("session initialization failed: %v", err)
	}

	return &Prime{session: session}, nil
}

func (p *Prime) run() error {
	fmt.Printf(`
Commands:
  %-15s Exit session
  %-15s Show help
  %-15s Clear screen
  %-15s View memory
  %-15s Message history

`,
		color.GreenString("exit, !exit"),
		color.YellowString("!help"),
		color.YellowString("!clear"),
		color.YellowString("!memory"),
		color.YellowString("!list"))

	for {
		// Show prompt
		fmt.Printf("%s> ", color.New(color.FgBlue).Add(color.Bold).Sprint(AppName))

		// Read input
		reader := bufio.NewReader(os.Stdin)
		line, err := reader.ReadString('\n')
		if err != nil {
			if err == io.EOF {
				fmt.Println("\nSession ended")
				return nil
			}
			return fmt.Errorf("input error: %v", err)
		}

		input := strings.TrimSpace(line)
		if input == "" {
			continue
		}

		// Handle exit
		if strings.EqualFold(input, "exit") || strings.EqualFold(input, "quit") {
			fmt.Println(color.New(color.Faint).Sprint("Session ended"))
			return nil
		}

		// Handle commands
		if strings.HasPrefix(input, "!") {
			shouldContinue, err := p.handleSpecialCommand(input[1:])
			if err != nil {
				color.Red("[ERROR] %v", err)
				continue
			}
			if !shouldContinue {
				return nil
			}
			continue
		}

		// Process input
		if err := p.processUserInput(input); err != nil {
			color.Red("[ERROR] %v", err)
		}
	}
}

func getEnvOrDefault(key, defaultValue string) string {
	if value := os.Getenv(key); value != "" {
		return value
	}
	return defaultValue
}

func (p *Prime) handleSpecialCommand(cmd string) (bool, error) {
	parts := strings.SplitN(cmd, " ", 2)
	command := strings.ToLower(parts[0])
	args := ""
	if len(parts) > 1 {
		args = strings.TrimSpace(parts[1])
	}

	switch command {
	case "clear", "cls":
		fmt.Print("\033[H\033[2J")
		return true, nil

	case "memory":
		return p.showMemory(args)

	case "list":
		return p.listMessages()

	case "read":
		return p.readMessage(args)

	case "help":
		return p.showHelp()

	case "exit", "quit":
		fmt.Println(color.New(color.Faint).Sprint("Session ended"))
		return false, nil

	default:
		fmt.Printf("Unknown command. Type !help for usage.\n")
		return true, nil
	}
}

func (p *Prime) showMemory(memType string) (bool, error) {
	if memType == "" {
		memType = "all"
	}

	content, err := p.session.ReadMemory(memType)
	if err != nil {
		return true, fmt.Errorf("failed to read memory: %v", err)
	}

	bar := strings.Repeat("─", contentWidth)
	fmt.Printf("\nMemory (%s)\n%s\n%s\n%s\n",
		memType,
		color.BlueString(bar),
		content,
		color.BlueString(bar))

	return true, nil
}

func (p *Prime) listMessages() (bool, error) {
	messages, err := p.session.ListMessages()
	if err != nil {
		return true, fmt.Errorf("failed to list messages: %v", err)
	}

	bar := strings.Repeat("─", contentWidth)
	fmt.Printf("\nMessage History\n%s\n", color.BlueString(bar))
	for _, msg := range messages {
		fmt.Println(msg)
	}
	fmt.Printf("%s\n", color.BlueString(bar))

	return true, nil
}

func (p *Prime) readMessage(arg string) (bool, error) {
	if arg == "" {
		return true, fmt.Errorf("usage: !read <message_number>")
	}

	msgNum, err := strconv.ParseUint(arg, 10, 64)
	if err != nil {
		return true, fmt.Errorf("invalid message number: %s", arg)
	}

	content, err := p.session.ReadMessage(uint(msgNum))
	if err != nil {
		return true, fmt.Errorf("failed to read message: %v", err)
	}

	bar := strings.Repeat("─", contentWidth)
	fmt.Printf("\nMessage #%d\n%s\n%s\n%s\n",
		msgNum,
		color.BlueString(bar),
		content,
		color.BlueString(bar))

	return true, nil
}

func (p *Prime) showHelp() (bool, error) {
	bar := strings.Repeat("─", contentWidth)
	fmt.Printf(`
%s
Prime Development Environment

Input:
  Enter code-related requests or questions

Commands:
  %-20s Exit session
  %-20s Show help
  %-20s Clear screen
  %-20s View memory state
  %-20s Message history
  %-20s View message
%s
`,
		color.BlueString(bar),
		color.GreenString("exit, !exit"),
		color.YellowString("!help"),
		color.YellowString("!clear"),
		color.YellowString("!memory [type]"),
		color.YellowString("!list"),
		color.YellowString("!read <num>"),
		color.BlueString(bar))

	return true, nil
}

func (p *Prime) processUserInput(input string) error {
	if err := p.session.AddUserMessage(input); err != nil {
		return fmt.Errorf("failed to save message: %v", err)
	}

	currentPrompt := input
	recursionDepth := 0
	const maxRecursionDepth = 3

	for {
		if recursionDepth >= maxRecursionDepth {
			return fmt.Errorf("maximum recursion depth reached")
		}

		fmt.Print("\nProcessing request...\n")

		llmResponse, err := p.session.GeneratePrimeResponse(currentPrompt, recursionDepth > 0)
		if err != nil {
			return fmt.Errorf("failed to generate response: %v", err)
		}

		bar := strings.Repeat("─", bannerWidth)
		fmt.Printf("\nResponse:\n%s\n", color.BlueString(bar))
		fmt.Print(p.highlightResponse(llmResponse))
		fmt.Printf("%s\n", color.BlueString(bar))

		results, err := p.session.ProcessCommands(llmResponse)
		if err != nil {
			return fmt.Errorf("command processing failed: %v", err)
		}

		if len(results) == 0 {
			return nil
		}

		var failedCommands strings.Builder
		allSucceeded := true

		for _, result := range results {
			if !result.Success {
				allSucceeded = false
				fmt.Fprintf(&failedCommands,
					"Command:\n```\n%s\n```\nFailed (exit code %d):\n```\n%s\n```\n\n",
					result.Command, result.ExitCode, result.Output)
			}
		}

		if allSucceeded {
			color.Green("Commands completed successfully")
			return nil
		}

		recursionDepth++
		color.Yellow("Attempting error recovery (try %d/%d)...",
			recursionDepth, maxRecursionDepth)

		currentPrompt = fmt.Sprintf(
			"Command execution failed. Please provide corrected commands.\n\n"+
				"Original request:\n%s\n\n"+
				"Failed commands:\n%s\n"+
				"Provide corrected commands or indicate if the task cannot be completed.",
			input, failedCommands.String())
	}
}

func (p *Prime) highlightResponse(response string) string {
	var result strings.Builder
	inCodeBlock := false
	scanner := bufio.NewScanner(strings.NewReader(response))

	for scanner.Scan() {
		line := scanner.Text()
		if strings.HasPrefix(line, "```") {
			inCodeBlock = !inCodeBlock
			result.WriteString(color.YellowString(line) + "\n")
		} else if inCodeBlock {
			result.WriteString(color.YellowString(line) + "\n")
		} else {
			result.WriteString(line + "\n")
		}
	}

	return result.String()
}
