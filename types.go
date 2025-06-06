package main

// CommandExecutionResult holds the result of a single command execution
type CommandExecutionResult struct {
	Command  string
	ExitCode int
	Output   string
	Success  bool
}

// Message represents a message in a Prime session
type Message struct {
	Number  uint
	Type    string
	Path    string
	Content string
}

// MemoryEntry represents a memory entry
type MemoryEntry struct {
	MemoryType string
	Category   string
	Content    string
}

// CommandProcessor is the interface for executing commands
type CommandProcessor interface {
	ExecuteCommand(command string) (int, string, error)
	ExecuteScript(scriptContent string) (int, string, error)
	IsDestructiveCommand(command string) bool
	ExecuteInDirectory(command string, directory string) (int, string, error)
}

// MemoryManager is the interface for managing memory
type MemoryManager interface {
	Initialize() error
	AddMemory(memoryType, category, content string) error
	ReadMemory(memoryType string) (string, error)
	ClearShortTermMemory() error
	SearchMemory(query string, memoryType string) ([]MemoryEntry, error)
	GetCategories(memoryType string) ([]string, error)
}
