//go:build windows
// +build windows

package main

import (
	"fmt"
	"log"
	"net"
	"os/exec"
	"strings"
	"time"

	"github.com/kardianos/service"
)

// Define constants for commonly used literals
const (
	powershellExe = "C:\\Windows\\System32\\WindowsPowerShell\\v1.0\\powershell.exe"
	cmdFlag       = "-Command"
	taskName      = "WazuhAgentUpdate"
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
	output, err := cmd.CombinedOutput()
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

// createScheduledTask creates a Windows scheduled task that will run in the user's context
func createScheduledTask() error {
	// PowerShell script to create a scheduled task
	psScript := fmt.Sprintf(`
		$taskName = "%s"
		$updateExe = "C:\Program Files (x86)\ossec-agent\active-response\bin\adorsys-update.exe"
		
		# Check if task already exists
		$existingTask = Get-ScheduledTask -TaskName $taskName -ErrorAction SilentlyContinue
		if ($existingTask) {
			Unregister-ScheduledTask -TaskName $taskName -Confirm:$false
		}
		
		# Create the action
		$action = New-ScheduledTaskAction -Execute $updateExe
		
		# Create a trigger that runs immediately
		$trigger = New-ScheduledTaskTrigger -Once -At (Get-Date).AddSeconds(2)
		
		# Set to run with highest privileges in interactive mode
		$principal = New-ScheduledTaskPrincipal -UserId "INTERACTIVE" -LogonType Interactive -RunLevel Highest
		
		# Create settings
		$settings = New-ScheduledTaskSettingsSet -AllowStartIfOnBatteries -DontStopIfGoingOnBatteries -StartWhenAvailable
		
		# Register the task
		Register-ScheduledTask -TaskName $taskName -Action $action -Trigger $trigger -Principal $principal -Settings $settings -Force
		
		# Run the task immediately
		Start-ScheduledTask -TaskName $taskName
		
		# Wait a moment for task to start
		Start-Sleep -Seconds 2
		
		# Clean up the task after it runs
		Unregister-ScheduledTask -TaskName $taskName -Confirm:$false
	`, taskName)

	cmd := exec.Command(powershellExe, "-ExecutionPolicy", "Bypass", cmdFlag, psScript)
	output, err := cmd.CombinedOutput()
	if err != nil {
		log.Printf("Failed to create scheduled task: %v\nOutput: %s", err, string(output))
		return err
	}
	log.Printf("Scheduled task created and started successfully")
	return nil
}

// updateAgent updates the Wazuh agent on Windows using Task Scheduler
func updateAgent() {
	log.Printf("Launching Wazuh agent update via Task Scheduler...\n")
	
	err := createScheduledTask()
	if err != nil {
		// Fallback to WMI method if Task Scheduler fails
		log.Printf("Task Scheduler failed, trying WMI method...\n")
		updateAgentViaWMI()
	} else {
		log.Printf("Wazuh agent update launched successfully via Task Scheduler\n")
	}
}

// updateAgentViaWMI uses WMI to launch the update in the user's session (fallback method)
func updateAgentViaWMI() {
	psScript := `
		# Get the active user session
		$sessions = Get-CimInstance -ClassName Win32_ComputerSystem | Select-Object -ExpandProperty UserName
		if ($sessions) {
			# Get the session ID of the active user
			$sessionId = (Get-Process -IncludeUserName | Where-Object {$_.UserName -eq $sessions} | Select-Object -First 1).SessionId
			if ($sessionId) {
				# Use WMI to create process in user session
				$updateExe = "C:\Program Files (x86)\ossec-agent\active-response\bin\adorsys-update.exe"
				
				# Create process in the user's session
				$startInfo = ([wmiclass]"\\localhost\root\cimv2:Win32_ProcessStartup").CreateInstance()
				$startInfo.ShowWindow = 1  # Show window
				
				$result = ([wmiclass]"\\localhost\root\cimv2:Win32_Process").Create($updateExe, $null, $startInfo)
				
				if ($result.ReturnValue -eq 0) {
					Write-Output "Update process started successfully with PID: $($result.ProcessId)"
				} else {
					Write-Error "Failed to start update process. Return code: $($result.ReturnValue)"
				}
			} else {
				Write-Error "Could not find active user session ID"
			}
		} else {
			Write-Error "No active user session found"
		}
	`

	cmd := exec.Command(powershellExe, "-ExecutionPolicy", "Bypass", cmdFlag, psScript)
	output, err := cmd.CombinedOutput()
	if err != nil {
		logFilePath := "C:\\Program Files (x86)\\ossec-agent\\active-response\\active-responses.log"
		errorMessage := fmt.Sprintf("Failed to launch update via WMI: %v. Output: %s\nFor details check logs at %s", err, string(output), logFilePath)
		log.Printf("%s\n", errorMessage)
		
		// Last resort: try direct execution
		updateAgentDirect()
	} else {
		log.Printf("Wazuh agent update launched successfully via WMI\nOutput: %s\n", string(output))
	}
}

// updateAgentDirect attempts direct execution (last resort)
func updateAgentDirect() {
	log.Printf("Attempting direct execution as last resort...\n")
	
	psScript := `
		$updateExe = "C:\Program Files (x86)\ossec-agent\active-response\bin\adorsys-update.exe"
		Start-Process -FilePath $updateExe -Verb RunAs -WindowStyle Normal
	`

	cmd := exec.Command(powershellExe, "-ExecutionPolicy", "Bypass", cmdFlag, psScript)
	err := cmd.Start()
	if err != nil {
		log.Printf("Direct execution failed: %v\n", err)
		log.Printf("All update methods have failed. Manual intervention may be required.\n")
	} else {
		log.Printf("Direct execution initiated\n")
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