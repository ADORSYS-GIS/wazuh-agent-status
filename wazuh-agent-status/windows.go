//go:build windows
// +build windows

package main

import (
	"log"
	"net"
	"os/exec"
	"time"
	"strings"

	"github.com/kardianos/service"
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
	cmd := exec.Command("powershell", "-Command", "Get-Service -Name WazuhSvc")
	output, err := cmd.CombinedOutput() // Use CombinedOutput to capture both stdout and stderr
	if err != nil {
		log.Printf("[%s] Error checking service status: %v\n", time.Now().Format(time.RFC3339), err)
		log.Printf("[%s] Service command error output:\n%s\n", time.Now().Format(time.RFC3339), string(output))
		return "Inactive", "Disconnected"
	}

	// Debugging the service output
	log.Printf("[%s] Service status output:\n%s\n", time.Now().Format(time.RFC3339), string(output))

	// Check if the service is running
	status := "Inactive"
	if strings.Contains(string(output), "Running") {
		status = "Active"
	}

	// Check connection status by reading the wazuh-agent.state file
	connCmd := exec.Command("powershell", "-Command", "Select-String -Path 'C:\\Program Files (x86)\\ossec-agent\\wazuh-agent.state' -Pattern '^status'")
	connOutput, connErr := connCmd.CombinedOutput()
	if connErr != nil {
		log.Printf("[%s] Error checking connection status: %v\n", time.Now().Format(time.RFC3339), connErr)
		log.Printf("[%s] Connection command error output:\n%s\n", time.Now().Format(time.RFC3339), string(connOutput))
		return status, "Disconnected"
	}

	// Debugging the connection status output
	log.Printf("[%s] Connection status output:\n%s\n", time.Now().Format(time.RFC3339), string(connOutput))

	// Clean the output and check if the status indicates "connected"
	connection := "Disconnected"
	if strings.Contains(string(connOutput), "status='connected'") {
		connection = "Connected"
	}

	return status, connection
}

// pauseAgent pauses the Wazuh agent on Windows
func pauseAgent() {
	log.Printf("[%s] Pausing Wazuh agent...\n", time.Now().Format(time.RFC3339))

	// Stop the service using sc stop command
	cmd := exec.Command("powershell", "-Command", "Stop-Service -Name WazuhSvc")
	err := cmd.Run()
	if err != nil {
		log.Printf("[%s] Failed to pause Wazuh agent: %v\n", time.Now().Format(time.RFC3339), err)
		return
	}
	log.Printf("[%s] Wazuh agent paused successfully\n", time.Now().Format(time.RFC3339))

	// Wait for a few seconds to allow the service to fully stop
	time.Sleep(5 * time.Second)
}

// restartAgent restarts the Wazuh agent on Windows
func restartAgent() {

	pauseAgent()

	log.Printf("[%s] Restarting Wazuh agent...\n", time.Now().Format(time.RFC3339))

	cmd := exec.Command("powershell", "-Command", "Start-Service -Name WazuhSvc")
	err := cmd.Run()
	if err != nil {
		log.Printf("[%s] Failed to restart Wazuh agent: %v\n", time.Now().Format(time.RFC3339), err)
	} else {
		log.Printf("[%s] Wazuh agent restarted successfully\n", time.Now().Format(time.RFC3339))
	}

	// Wait for a few seconds to allow the service to fully stop
	time.Sleep(5 * time.Second)
}

// updategent updates the Wazuh agent on macOS
func updateAgent() {
	log.Printf("[%s] Updating Wazuh agent...\n", time.Now().Format(time.RFC3339))

	cmd := exec.Command("powershell", "-Command", "C:\\Program Files (x86)\\ossec-agent\\adorsys-update.ps1")
	err := cmd.Run()
	if err != nil {
		log.Printf("[%s] Failed to update Wazuh agent: %v\n", time.Now().Format(time.RFC3339), err)
	} else {
		log.Printf("[%s] Wazuh agent updated successfully\n", time.Now().Format(time.RFC3339))
	}
	
	restartAgent()
	
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
