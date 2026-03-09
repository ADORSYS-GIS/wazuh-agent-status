//go:build windows
// +build windows

package main

import (
	"fmt"
	"log"
	"net"
	"os"
	"os/exec"
	"strings"

	"github.com/kardianos/service"
)

// Define constants for commonly used literals
const (
	cmdFlag      = "-Command"
	taskName     = "WazuhAgentUpdate"
	updateFlag   = "-Update"
	updateScript = "C:\\Program Files (x86)\\ossec-agent\\active-response\\bin\\adorsys-update.ps1"
)

// Define the program structure for the service
type program struct {
	listener net.Listener
}

// Start will be called when the service is started
func (p *program) Start(s service.Service) error {
	log.Println("Starting wazuh-agent-status server...")
	go p.run()
	return nil
}

// The actual server logic in the background
func (p *program) run() {
	listener, err := net.Listen("tcp", ":"+backendPort)
	if err != nil {
		log.Fatalf("Failed to start server: %v", err)
		return
	}
	p.listener = listener
	defer listener.Close()
	log.Println("wazuh-agent-status server listening on port " + backendPort)

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
	cmd := exec.Command(powershellExe, cmdFlag, "Get-Service", "-Name", "WazuhSvc")
	output, err := cmd.CombinedOutput()
	if err != nil {
		log.Printf("Error checking service status: %v\n", err)
		log.Printf("Service command error output:\n%s\n", string(output))
		return "Inactive", "Disconnected"
	}

	status := "Inactive"
	if strings.Contains(string(output), "Running") {
		status = "Active"
	}

	connCmd := exec.Command(powershellExe, cmdFlag, "Select-String -Path 'C:\\Program Files (x86)\\ossec-agent\\wazuh-agent.state' -Pattern '^status'")
	connOutput, connErr := connCmd.CombinedOutput()
	if connErr != nil {
		log.Printf("Error checking connection status: %v\n", connErr)
		log.Printf("Connection command error output:\n%s\n", string(connOutput))
		return status, "Disconnected"
	}

	connection := "Disconnected"
	if strings.Contains(string(connOutput), "status='connected'") {
		connection = "Connected"
	}

	return status, connection
}

// updateAgent updates the Wazuh agent on Windows and streams progress to the client
func updateAgent(conn net.Conn, isPrerelease bool) {
	// The caller (handleConnection) closes the dedicated update stream conn when this function returns
	defer conn.Close()

	// Helper to write status updates directly to the connection
	writeUpdate := func(status string) {
		conn.Write([]byte(fmt.Sprintf("UPDATE_PROGRESS: %s\n", status)))
		log.Printf("Update progress: %s", status)
	}

	writeUpdate("Starting...")

	logFileHandle, err := createLogFile()
	if err != nil {
		return
	}
	defer logFileHandle.Close()

	if isPrerelease {
		handlePrereleaseUpdate(writeUpdate, logFileHandle)
	} else {
		handleRegularUpdate(writeUpdate, logFileHandle)
	}
}

// handleRegularUpdate handles the regular update process
func handleRegularUpdate(writeUpdate func(string), logFileHandle *os.File) {
	writeUpdate("Using regular update method")
	logFileHandle.WriteString("Using regular update method\n")
	psScript := fmt.Sprintf(`
		$updateScript = %s
		Start-Process -FilePath $updateScript -ArgumentList "%s" -Verb RunAs -WindowStyle Normal
	`, updateScript, updateFlag)

	cmd := exec.Command(powershellExe, executionPolicyFlag, "Bypass", cmdFlag, psScript)
	err := cmd.Start()
	if err != nil {
		writeUpdate("Update failed")
		logFileHandle.WriteString("Update failed\n")
	} else {
		writeUpdate("Update initiated")
		logFileHandle.WriteString("Update initiated\n")
	}
}

// Main function that sets up the service
func windowsMain() {
	serviceConfig := &service.Config{
		Name:        "GoWazuhService",
		DisplayName: "Go Wazuh Service",
		Description: "A Go application to manage Wazuh service.",
	}

	prg := &program{}

	s, err := service.New(prg, serviceConfig)
	if err != nil {
		log.Fatalf("Failed to create service: %v", err)
	}

	err = s.Run()
	if err != nil {
		log.Fatalf("Failed to run service: %v", err)
	}
}
