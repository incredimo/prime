package main

import (
	"fmt"
	"os"
	"path/filepath"
	"strings"
	"time"
)

// MemoryManagerImpl manages Prime's memory files
type MemoryManagerImpl struct {
	memoryDir     string
	longTermFile  string
	shortTermFile string
}

// NewMemoryManager creates a new memory manager
func NewMemoryManager(memoryDir string) *MemoryManagerImpl {
	return &MemoryManagerImpl{
		memoryDir:     memoryDir,
		longTermFile:  filepath.Join(memoryDir, "long_term.md"),
		shortTermFile: filepath.Join(memoryDir, "short_term.md"),
	}
}

// Initialize initializes memory files if they don't exist
func (m *MemoryManagerImpl) Initialize() error {
	// Create memory directory if it doesn't exist
	if err := os.MkdirAll(m.memoryDir, 0755); err != nil {
		return fmt.Errorf("failed to create memory directory: %v", err)
	}

	// Initialize long-term memory file if it doesn't exist
	if err := m.initializeFile(m.longTermFile, "Long-term"); err != nil {
		return err
	}

	// Initialize short-term memory file if it doesn't exist
	if err := m.initializeFile(m.shortTermFile, "Short-term"); err != nil {
		return err
	}

	return nil
}

func (m *MemoryManagerImpl) initializeFile(path string, memoryType string) error {
	if _, err := os.Stat(path); os.IsNotExist(err) {
		content := fmt.Sprintf("# Prime %s Memory\n\n", memoryType)
		if err := os.WriteFile(path, []byte(content), 0644); err != nil {
			return fmt.Errorf("failed to initialize %s memory file: %v", strings.ToLower(memoryType), err)
		}
	}
	return nil
}

// AddMemory adds a memory entry to the specified memory type
func (m *MemoryManagerImpl) AddMemory(memoryType, category, content string) error {
	var memoryFile string
	switch strings.ToLower(memoryType) {
	case "long", "long_term":
		memoryFile = m.longTermFile
	case "short", "short_term":
		memoryFile = m.shortTermFile
	default:
		return fmt.Errorf("invalid memory type: %s", memoryType)
	}

	// Ensure memory directory and files exist
	if err := m.Initialize(); err != nil {
		return err
	}

	// Read existing memory content
	memoryContent, err := os.ReadFile(memoryFile)
	if err != nil {
		return fmt.Errorf("failed to read memory file: %v", err)
	}

	lines := strings.Split(string(memoryContent), "\n")
	var updatedContent strings.Builder
	categoryFound := false
	categoryProcessed := false

	// Write header and find category location
	for i, line := range lines {
		if strings.HasPrefix(line, fmt.Sprintf("## %s", category)) {
			categoryFound = true
			updatedContent.WriteString(line + "\n")
			timestamp := time.Now().Format("2006-01-02 15:04:05")
			updatedContent.WriteString(fmt.Sprintf("- %s (added: %s)\n", content, timestamp))
			categoryProcessed = true
		} else if categoryProcessed && strings.HasPrefix(line, "## ") {
			// We've found the next category after processing our target
			categoryProcessed = false
			updatedContent.WriteString(line + "\n")
		} else {
			updatedContent.WriteString(line)
			if i < len(lines)-1 {
				updatedContent.WriteString("\n")
			}
		}
	}

	// If category wasn't found, add it at the end
	if !categoryFound {
		if !strings.HasSuffix(updatedContent.String(), "\n") {
			updatedContent.WriteString("\n")
		}
		updatedContent.WriteString(fmt.Sprintf("## %s\n", category))
		timestamp := time.Now().Format("2006-01-02 15:04:05")
		updatedContent.WriteString(fmt.Sprintf("- %s (added: %s)\n", content, timestamp))
	}

	// Write updated content back to file
	if err := os.WriteFile(memoryFile, []byte(updatedContent.String()), 0644); err != nil {
		return fmt.Errorf("failed to write updated memory file: %v", err)
	}

	return nil
}

// ReadMemory reads memory content
func (m *MemoryManagerImpl) ReadMemory(memoryType string) (string, error) {
	// Ensure memory files exist
	if err := m.Initialize(); err != nil {
		return "", err
	}

	switch strings.ToLower(memoryType) {
	case "long", "long_term":
		content, err := os.ReadFile(m.longTermFile)
		if err != nil {
			return "", fmt.Errorf("failed to read long-term memory file: %v", err)
		}
		return string(content), nil

	case "short", "short_term":
		content, err := os.ReadFile(m.shortTermFile)
		if err != nil {
			return "", fmt.Errorf("failed to read short-term memory file: %v", err)
		}
		return string(content), nil

	case "", "all":
		// Combine both memory types
		shortTerm, err := os.ReadFile(m.shortTermFile)
		if err != nil {
			return "", fmt.Errorf("failed to read short-term memory file: %v", err)
		}

		longTerm, err := os.ReadFile(m.longTermFile)
		if err != nil {
			return "", fmt.Errorf("failed to read long-term memory file: %v", err)
		}

		return fmt.Sprintf("%s\n\n%s", string(shortTerm), string(longTerm)), nil

	default:
		return "", fmt.Errorf("invalid memory type: %s", memoryType)
	}
}

// ClearShortTermMemory clears short-term memory
func (m *MemoryManagerImpl) ClearShortTermMemory() error {
	content := "# Prime Short-term Memory\n\n"
	if err := os.WriteFile(m.shortTermFile, []byte(content), 0644); err != nil {
		return fmt.Errorf("failed to clear short-term memory file: %v", err)
	}
	return nil
}

// SearchMemory searches memory for content
func (m *MemoryManagerImpl) SearchMemory(query string, memoryType string) ([]MemoryEntry, error) {
	memoryContent, err := m.ReadMemory(memoryType)
	if err != nil {
		return nil, err
	}

	queryLower := strings.ToLower(query)
	var results []MemoryEntry

	currentCategory := ""
	currentType := memoryType
	if currentType == "" {
		currentType = "all"
	}

	for _, line := range strings.Split(memoryContent, "\n") {
		if strings.HasPrefix(line, "# Prime ") {
			// Memory type header
			if strings.Contains(line, "Long-term") {
				currentType = "long"
			} else if strings.Contains(line, "Short-term") {
				currentType = "short"
			}
		} else if strings.HasPrefix(line, "## ") {
			// Category header
			currentCategory = strings.TrimPrefix(line, "## ")
		} else if strings.HasPrefix(line, "- ") && strings.Contains(strings.ToLower(line), queryLower) {
			// Entry that matches query
			results = append(results, MemoryEntry{
				MemoryType: currentType,
				Category:   currentCategory,
				Content:    strings.TrimPrefix(line, "- "),
			})
		}
	}

	return results, nil
}

// GetCategories gets all categories from memory
func (m *MemoryManagerImpl) GetCategories(memoryType string) ([]string, error) {
	memoryContent, err := m.ReadMemory(memoryType)
	if err != nil {
		return nil, err
	}

	var categories []string
	for _, line := range strings.Split(memoryContent, "\n") {
		if strings.HasPrefix(line, "## ") {
			categories = append(categories, strings.TrimPrefix(line, "## "))
		}
	}

	return categories, nil
}
