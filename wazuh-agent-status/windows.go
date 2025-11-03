//go:build windows
// +build windows

package main

import (
	"fmt"
	"io"
	"log"
	"net"
	"os"
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

	connCmd := exec.Command(powershellExe, cmdFlag, "Select-String", "-Path", "C:\\Program Files (x86)\\ossec-agent\\wazuh-agent.state", "-Pattern", "^status")
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
func updateAgent(conn net.Conn) {
	// The caller (handleConnection) closes the dedicated update stream conn when this function returns
	defer conn.Close() 

	// Helper to write status updates directly to the connection
	writeUpdate := func(status string) {
		conn.Write([]byte(fmt.Sprintf("UPDATE_PROGRESS: %s\n", status)))
		log.Printf("Update progress: %s", status)
	}

	writeUpdate("Starting...")

	// Set the execution policy to RemoteSigned for the current user
	setPolicyCmd := exec.Command(powershellExe, cmdFlag, "Set-ExecutionPolicy", "-Scope", "CurrentUser", "-ExecutionPolicy", "RemoteSigned", "-Force")
	if output, err := setPolicyCmd.CombinedOutput(); err != nil {
		writeUpdate(fmt.Sprintf("ERROR: Failed to set execution policy: %v", string(output)))
		return
	}
	writeUpdate("Policy Set...")

	// Command to execute the update script
	cmd := exec.Command(powershellExe, cmdFlag, "&", "'C:\\Program Files (x86)\\ossec-agent\\active-response\\bin\\adorsys-update.ps1'")

	// Stream stdout and stderr to the log and to the client connection (Note: PowerShell streaming can be complex; this is a simplified approach)
	stdout, _ := cmd.StdoutPipe()
	stderr, _ := cmd.StderrPipe()

	if err := cmd.Start(); err != nil {
		writeUpdate(fmt.Sprintf("ERROR: Command failed to start: %v", err))
		return
	}

	writeUpdate("Executing script...")

	// Stream stdout and stderr to os.Stdout/os.Stderr (which goes to the server log)
	go io.Copy(os.Stdout, stdout)
	go io.Copy(os.Stderr, stderr)

	// Wait for the command to finish
	if err := cmd.Wait(); err != nil {
		logFilePath := "C:\\Program Files (x86)\\ossec-agent\\active-response\\active-responses.log"
		errorMessage := fmt.Sprintf("ERROR: Update failed: For details check logs at %s", logFilePath)
		writeUpdate("Error")
		log.Println(errorMessage)
	} else {
		writeUpdate("Complete")
		log.Println("Wazuh agent updated successfully")
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