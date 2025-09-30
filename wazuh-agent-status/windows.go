//go:build windows
// +build windows

package main

import (
	"log"
	"net"
	"os"
	"os/exec"
	"path/filepath"
	"strings"
	"syscall"

	"golang.org/x/sys/windows"

	"github.com/kardianos/service"
)

// Define constants for commonly used literals
const (
	powershellExe = "C:\\Windows\\System32\\WindowsPowerShell\\v1.0\\powershell.exe"
	cmdFlag       = "-Command"
)

// Define the program structure for the service
type program struct {
	// Any fields you want to manage for your service, such as the listener
	listener net.Listener
}

// Start will be called when the service is started
func (p *program) Start(s service.Service) error {
	log.Println("Starting wazuh-agent-status server...")

	// Start the listener in a goroutine
	go p.run()
	return nil
}

// The actual server logic in the background
func (p *program) run() {
	listener, err := net.Listen("tcp", ":50505")
	if err != nil {
		log.Fatalf("Failed to start server: %v", err)
		return
	}
	p.listener = listener
	defer listener.Close()
	log.Println("wazuh-agent-status server listening on port 50505")

	// Handle incoming connections
	for {
		conn, err := listener.Accept()
		if err != nil {
			log.Printf("Failed to accept connection: %v", err)
			continue
		}
		go handleConnection(conn)
	}
}

// Stop will be called when the service is stopped
func (p *program) Stop(s service.Service) error {
	log.Println("Stopping wazuh-agent-status server...")
	if p.listener != nil {
		p.listener.Close()
	}
	return nil
}

// checkServiceStatus checks the status of Wazuh agent and its connection on Windows
func checkServiceStatus() (string, string) {
	// Check if the Wazuh service is running
	cmd := exec.Command(powershellExe, cmdFlag, "Get-Service", "-Name", "WazuhSvc")
	output, err := cmd.CombinedOutput() // Use CombinedOutput to capture both stdout and stderr
	if err != nil {
		log.Printf("Error checking service status: %v\n", err)
		log.Printf("Service command error output:\n%s\n", string(output))
		return "Inactive", "Disconnected"
	}

	// Check if the service is running
	status := "Inactive"
	if strings.Contains(string(output), "Running") {
		status = "Active"
	}

	// Check connection status by reading the wazuh-agent.state file
	statePath := `"C:\\Program Files (x86)\\ossec-agent\\wazuh-agent.state"`
	connCmd := exec.Command(powershellExe, cmdFlag, "Select-String", "-Path", statePath, "-Pattern", "^status")
	connOutput, connErr := connCmd.CombinedOutput()
	if connErr != nil {
		log.Printf("Error checking connection status: %v\n", connErr)
		log.Printf("Connection command error output:\n%s\n", string(connOutput))
		return status, "Disconnected"
	}

	// Clean the output and check if the status indicates "connected"
	connection := "Disconnected"
	if strings.Contains(string(connOutput), "status='connected'") {
		connection = "Connected"
	}
	return status, connection
}

// updategent updates the Wazuh agent on windows
func updateAgent() {
	log.Printf("Updating Wazuh agent...\n")
	programFiles := os.Getenv("ProgramFiles(x86)")
	scriptPath := filepath.Join(programFiles, "ossec-agent", "active-response", "bin", "adorsys-update.ps1")
	logFilePath := filepath.Join(programFiles, "ossec-agent", "active-response", "active-responses.log")

	bgCmd := exec.Command(powershellExe, "-File", scriptPath)

	// Ensure detached process
	bgCmd.SysProcAttr = &syscall.SysProcAttr{
		CreationFlags: windows.CREATE_NEW_PROCESS_GROUP | windows.DETACHED_PROCESS | windows.CREATE_NO_WINDOW,
	}

	// Open the log file for child process
	logFile, err := os.OpenFile(logFilePath, os.O_APPEND|os.O_CREATE|os.O_WRONLY, 0666)
	if err != nil {
		log.Printf("Failed to open log file: %v", err)
		return
	}
	bgCmd.Stdout = logFile
	bgCmd.Stderr = logFile

	if err := bgCmd.Start(); err != nil {
		log.Printf("Failed to trigger background update: %v. See %s", err, logFilePath)
		return
	}

	// Don’t wait for completion; process runs independently
	log.Printf("Wazuh agent update triggered in background\n")
}

// Main function that sets up the service
func windowsMain() {
	// Define the service config
	serviceConfig := &service.Config{
		Name:        "GoWazuhService",
		DisplayName: "Go Wazuh Service",
		Description: "A Go application to manage Wazuh service.",
	}

	// Create the program object
	prg := &program{}

	// Create a new service object
	s, err := service.New(prg, serviceConfig)
	if err != nil {
		log.Fatalf("Failed to create service: %v", err)
	}

	// Run the service
	err = s.Run()
	if err != nil {
		log.Fatalf("Failed to run service: %v", err)
	}
}
