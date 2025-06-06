package main

import (
	"fmt"
	"os"
	"os/exec"
	"runtime"
	"strings"
)

// CommandProcessorImpl handles command execution for Prime
type CommandProcessorImpl struct {
	shellCommand string
	shellArgs    []string
}

// NewCommandProcessor creates a new command processor
func NewCommandProcessor() *CommandProcessorImpl {
	var shellCommand string
	var shellArgs []string

	if runtime.GOOS == "windows" {
		shellCommand = "powershell"
		shellArgs = []string{"-Command"}
	} else {
		shellCommand = "sh"
		shellArgs = []string{"-c"}
	}

	return &CommandProcessorImpl{
		shellCommand: shellCommand,
		shellArgs:    shellArgs,
	}
}

// ExecuteCommand executes a shell command and returns its output
func (cp *CommandProcessorImpl) ExecuteCommand(command string) (int, string, error) {
	fmt.Printf("Executing: %s\n", command)

	// Create args by cloning shellArgs and adding the command
	args := make([]string, len(cp.shellArgs), len(cp.shellArgs)+1)
	copy(args, cp.shellArgs)
	args = append(args, command)

	// Execute the command
	cmd := exec.Command(cp.shellCommand, args...)
	output, err := cmd.CombinedOutput()

	// Get exit code
	exitCode := 0
	if err != nil {
		if exitErr, ok := err.(*exec.ExitError); ok {
			exitCode = exitErr.ExitCode()
		} else {
			return -1, "", fmt.Errorf("failed to execute command: %v", err)
		}
	}

	// Convert output to string
	outputStr := string(output)

	// Print a short summary of the result
	fmt.Printf("Command completed with exit code: %d\n", exitCode)

	// Print output with appropriate feedback
	lines := strings.Split(strings.TrimSpace(outputStr), "\n")
	if len(lines) > 0 {
		if len(lines) > 5 {
			fmt.Println("\nOutput preview (first 5 lines):")
			fmt.Println(strings.Join(lines[:5], "\n"))
			fmt.Printf("\n... (%d more lines, full output saved in conversation)\n", len(lines)-5)
		} else {
			fmt.Println("\nOutput:")
			fmt.Println(outputStr)
		}
	} else {
		fmt.Println("\nCommand produced no output")
	}

	return exitCode, outputStr, nil
}

// ExecuteScript executes a script file
func (cp *CommandProcessorImpl) ExecuteScript(scriptContent string) (int, string, error) {
	// Create a temporary script file
	var extension string
	if runtime.GOOS == "windows" {
		extension = ".ps1"
	} else {
		extension = ".sh"
	}

	tempFile, err := os.CreateTemp("", "prime_script_*"+extension)
	if err != nil {
		return -1, "", fmt.Errorf("failed to create temporary script file: %v", err)
	}
	defer os.Remove(tempFile.Name())

	// Write script content
	if err := os.WriteFile(tempFile.Name(), []byte(scriptContent), 0644); err != nil {
		return -1, "", fmt.Errorf("failed to write script content: %v", err)
	}

	// Make the script executable on Unix-like systems
	if runtime.GOOS != "windows" {
		if err := os.Chmod(tempFile.Name(), 0755); err != nil {
			return -1, "", fmt.Errorf("failed to make script executable: %v", err)
		}
	}

	// Execute the script
	scriptPath := tempFile.Name()
	if runtime.GOOS == "windows" {
		// For Windows, wrap the script path in single quotes and use &
		return cp.ExecuteCommand(fmt.Sprintf("& '%s'", scriptPath))
	}
	return cp.ExecuteCommand(scriptPath)
}

// IsDestructiveCommand checks if a command is potentially destructive
func (cp *CommandProcessorImpl) IsDestructiveCommand(command string) bool {
	command = strings.ToLower(strings.TrimSpace(command))

	dangerousPatterns := []string{}
	if runtime.GOOS == "windows" {
		dangerousPatterns = []string{
			"remove-item -recurse",
			"rmdir /s",
			"del /s",
			"format",
			"fdisk",
			"clear-disk",
			"initialize-disk",
			"remove-partition",
			"diskpart",
		}
	} else {
		dangerousPatterns = []string{
			"rm -rf",
			"rm -r",
			"rmdir",
			"mkfs",
			"fdisk",
			"format",
			"dd if=",
			"shred",
			":(){:|:&};:",
			"chmod -R 777",
			"mv /* /dev/null",
		}
	}

	for _, pattern := range dangerousPatterns {
		if strings.Contains(command, pattern) {
			return true
		}
	}

	return false
}

// ExecuteInDirectory executes a command within a specific directory
func (cp *CommandProcessorImpl) ExecuteInDirectory(command string, directory string) (int, string, error) {
	fmt.Printf("Executing in %s: %s\n", directory, command)

	// Create args by cloning shellArgs and adding the command
	args := make([]string, len(cp.shellArgs), len(cp.shellArgs)+1)
	copy(args, cp.shellArgs)
	args = append(args, command)

	// Execute command in specified directory
	cmd := exec.Command(cp.shellCommand, args...)
	cmd.Dir = directory
	output, err := cmd.CombinedOutput()

	// Get exit code
	exitCode := 0
	if err != nil {
		if exitErr, ok := err.(*exec.ExitError); ok {
			exitCode = exitErr.ExitCode()
		} else {
			return -1, "", fmt.Errorf("failed to execute command in %s: %v", directory, err)
		}
	}

	// Convert output to string
	outputStr := string(output)

	fmt.Printf("Command completed with exit code: %d\n", exitCode)

	return exitCode, outputStr, nil
}
