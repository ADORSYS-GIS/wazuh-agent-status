//go:build windows
// +build windows

package main

import (
	"fmt"
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
	notifierPath := filepath.Join(programFiles, "ossec-agent", "active-response", "bin", "update-notifier.ps1")

	if _, err := os.Stat(scriptPath); err != nil {
		log.Printf("Script file not found: %v", err)
		return
	}

	notifierStarted := false
	if _, err := os.Stat(notifierPath); err != nil {
		log.Printf("Notifier script not found: %v", err)
		// Fallback to running update directly
	} else {
		// Create a temporary Windows service to run the notifier which launches and monitors the update
		// The notifier will delete the service on completion
		serviceName := "wazuh-update-notifier"
		// Ensure any existing service is removed before creating
		_ = exec.Command("sc.exe", "stop", serviceName).Run()
		_ = exec.Command("sc.exe", "delete", serviceName).Run()

		// Build the binPath value - sc.exe expects the entire path (including arguments)
		// to be wrapped in quotes when there are spaces
		binPathValue := fmt.Sprintf("\"%s\" -NoProfile -ExecutionPolicy Bypass -File \"%s\" -UpdateScriptPath \"%s\" -ServiceName \"%s\"",
			powershellExe, notifierPath, scriptPath, serviceName)

		// Create service using separate arguments (more reliable than cmd /c)
		createCmd := exec.Command("sc.exe", "create", serviceName,
			"binPath="+binPathValue,
			"start=", "demand",
			"DisplayName=", "Wazuh Update Notifier",
			"obj=", "LocalSystem")

		if out, err := createCmd.CombinedOutput(); err != nil {
			log.Printf("Failed to create notifier service: %v, output: %s", err, string(out))
		} else {
			if out, err := exec.Command("sc.exe", "start", serviceName).CombinedOutput(); err != nil {
				log.Printf("Failed to start notifier service: %v, output: %s", err, string(out))
			} else {
				log.Printf("Started notifier service '%s' to monitor update", serviceName)
				notifierStarted = true
			}
		}
	}

	if notifierStarted {
		// Notifier launched and will execute the update itself; return.
		return
	}

	bgCmd := exec.Command(powershellExe, "-ExecutionPolicy", "RemoteSigned", "-File", scriptPath)

	bgCmd.SysProcAttr = &syscall.SysProcAttr{
		CreationFlags: windows.CREATE_NEW_PROCESS_GROUP | windows.CREATE_NO_WINDOW,
	}

	// Ensure no inherited handles that could tie it to the parent
	bgCmd.Stdin = nil
	bgCmd.Stdout = nil
	bgCmd.Stderr = nil

	if err := bgCmd.Start(); err != nil {
		log.Printf("Failed to trigger background update: %v", err)
		return
	}

	// Release immediately
	bgCmd.Process.Release()

	log.Printf("Wazuh agent update triggered in background (PID: %d)\n", bgCmd.Process.Pid)
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
