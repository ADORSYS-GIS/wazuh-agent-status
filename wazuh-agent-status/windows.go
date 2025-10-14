//go:build windows
// +build windows

package main

import (
	"fmt"
	"log"
	"net"
	"os/exec"
	"strings"

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
	connCmd := exec.Command(powershellExe, cmdFlag, "Select-String", "-Path", "C:\\Program Files (x86)\\ossec-agent\\wazuh-agent.state", "-Pattern", "^status")
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
	log.Printf("Launching Wazuh agent update in user session...\n")

	// Use PowerShell to launch the binary in the user's interactive session
	// The -WindowStyle Hidden hides the PowerShell window, but the launched exe will show its own UI
	psScript := `
		$updateExe = "C:\Program Files (x86)\ossec-agent\active-response\bin\adorsys-update.exe"
		Start-Process -FilePath $updateExe -Verb RunAs
	`

	updateCmd := exec.Command(powershellExe, cmdFlag, psScript)
	err := updateCmd.Start()
	if err != nil {
		logFilePath := "C:\\Program Files (x86)\\ossec-agent\\active-response\\active-responses.log"
		errorMessage := fmt.Sprintf("Failed to launch update: %v. For details check logs at %s", err, logFilePath)
		log.Printf("%s\n", errorMessage)
	} else {
		log.Printf("Wazuh agent update launcher started successfully in user session\n")
	}
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