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

	"github.com/kardianos/service"
)

// Define constants for commonly used literals
const (
	taskName   = "WazuhAgentUpdate"
	updateFlag = "-Update"
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

	var wazuhStatePath, statePathErr = getWazuhStatePath()
	if statePathErr != nil {
		log.Printf("Error getting Wazuh state path: %v", statePathErr)
		return "Inactive", "Disconnected"
	}

	connCmd := exec.Command(powershellExe, cmdFlag, fmt.Sprintf("Select-String -Path '%s' -Pattern '^status'", wazuhStatePath))
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

// createScheduledTask creates a Windows scheduled task that will run in the user's context
func createScheduledTask(updateExe string) error {
	// PowerShell script to create a scheduled task
	psScript := fmt.Sprintf(`
		$taskName = "%s"
		$updateExe = "%s"

		# Check if task already exists
		$existingTask = Get-ScheduledTask -TaskName $taskName -ErrorAction SilentlyContinue
		if ($existingTask) {
			Unregister-ScheduledTask -TaskName $taskName -Confirm:$false
		}

		# Create the action to run PowerShell with the script
		$action = New-ScheduledTaskAction -Execute $updateExe -Argument "%s"

		# Create a trigger that runs immediately
		$trigger = New-ScheduledTaskTrigger -Once -At (Get-Date).AddSeconds(2)

		# Set to run as BUILTIN\Administrators group with highest privileges
		$principal = New-ScheduledTaskPrincipal -GroupId "S-1-5-32-544" -RunLevel Highest

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
	`, taskName, updateExe, updateFlag)

	cmd := exec.Command(powershellExe, executionPolicyFlag, "Bypass", cmdFlag, psScript)
	output, err := cmd.CombinedOutput()
	if err != nil {
		log.Printf("Failed to create scheduled task: %v\nOutput: %s", err, string(output))
		return err
	}
	log.Printf("Scheduled task created and started successfully")
	return nil
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

	// Get the adorsys-update script path
	updateExe, err := getAdorsysUpdatePath()
	if err != nil {
		writeUpdate(fmt.Sprintf("ERROR: Failed to get update script path: %v", err))
		return
	}

	if isPrerelease {
		logFileHandle, err := createLogFile()
		if err != nil {
			writeUpdate(fmt.Sprintf("ERROR: Failed to create log file: %v", err))
			return
		}
		defer logFileHandle.Close()

		writeUpdate("Updating to prerelease")
		logFileHandle.WriteString("Using prerelease update method\n")
		if err := handlePrereleaseUpdate(logFileHandle); err != nil {
			log.Printf("Error handling prerelease update: %v", err)
			logFileHandle.WriteString(fmt.Sprintf("Error handling prerelease update: %v\n", err))
			writeUpdate("Error")
			return
		}
	} else {
		writeUpdate("Updating to stable")
		if err := handleRegularUpdate(updateExe); err != nil {
			log.Printf("Error handling regular update: %v", err)
			writeUpdate("Error")
			return
		}
	}

	log.Println("Wazuh agent updated successfully")
	writeUpdate("Complete")
}

// handleRegularUpdate handles the regular update process
func handleRegularUpdate(updateExe string) error {
	err := createScheduledTask(updateExe)
	if err != nil {
		return updateAgentViaWMI(updateExe)
	} else {
		return nil
	}
}

// updateAgentViaWMI uses WMI to launch the update in the user's session (fallback method)
func updateAgentViaWMI(updateExe string) error {
	psScript := fmt.Sprintf(`
		# Get the active user session
		$sessions = Get-CimInstance -ClassName Win32_ComputerSystem | Select-Object -ExpandProperty UserName
		if ($sessions) {
			# Get the session ID of the active user
			$sessionId = (Get-Process -IncludeUserName | Where-Object {$_.UserName -eq $sessions} | Select-Object -First 1).SessionId
			if ($sessionId) {
				# Use WMI to create process in user session
				$updateExe = %s
				
				# Create process in the user's session
				$startInfo = ([wmiclass]"\\localhost\root\cimv2:Win32_ProcessStartup").CreateInstance()
				$startInfo.ShowWindow = 1  # Show window
				
				$result = ([wmiclass]"\\localhost\root\cimv2:Win32_Process").Create("$updateExe %s", $null, $startInfo)
				
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
	`, updateExe, updateFlag)

	cmd := exec.Command(powershellExe, executionPolicyFlag, "Bypass", cmdFlag, psScript)
	output, err := cmd.CombinedOutput()
	if err != nil {
		logFilePath := "C:\\Program Files (x86)\\ossec-agent\\active-response\\active-responses.log"
		errorMessage := fmt.Sprintf("Failed to launch update via WMI: %v. Output: %s\nFor details check logs at %s", err, string(output), logFilePath)
		log.Printf("%s\n", errorMessage)

		// Last resort: try direct execution
		return updateAgentDirect(updateExe)
	} else {
		log.Printf("Wazuh agent update launched successfully via WMI\nOutput: %s\n", string(output))
		return nil
	}
}

// updateAgentDirect attempts direct execution (last resort)
func updateAgentDirect(updateExe string) error {
	log.Printf("Attempting direct execution as last resort...\n")
	psScript := fmt.Sprintf(`
		$updateExe = %s
		Start-Process -FilePath $updateExe -ArgumentList "%s" -Verb RunAs -WindowStyle Normal
	`, updateExe, updateFlag)

	cmd := exec.Command(powershellExe, executionPolicyFlag, "Bypass", cmdFlag, psScript)
	err := cmd.Start()
	if err != nil {
		log.Printf("Direct execution failed: %v\n", err)
		log.Printf("All update methods have failed. Manual intervention may be required.\n")
		return fmt.Errorf("all update methods failed: %w", err)
	} else {
		log.Printf("Direct execution initiated\n")
		return nil
	}
}

// handlePrereleaseUpdate handles the prerelease update process for Windows
func handlePrereleaseUpdate(logFileHandle *os.File) error {
	versionInfo := fetchVersionInfo()
	if versionInfo == nil || versionInfo.Framework.PrereleaseVersion == "" {
		return fmt.Errorf("empty prerelease")
	}

	// Get the adorsys-update.bat path
	updateScript, err := getAdorsysUpdatePath()
	if err != nil {
		return fmt.Errorf("failed to get update script path: %w", err)
	}

	// Execute the batch file with -Prerelease flag
	cmd := exec.Command(updateScript, "-Prerelease")
	return executePrereleaseScript(cmd, logFileHandle)
}

// executePrereleaseScript executes the prerelease script and waits for completion
func executePrereleaseScript(cmd *exec.Cmd, logFileHandle *os.File) error {
	// Stream stdout and stderr ONLY to the update log file
	cmd.Stdout = logFileHandle
	cmd.Stderr = logFileHandle

	// Execute the prerelease script
	if err := cmd.Start(); err != nil {
		return fmt.Errorf("command failed to start: %w", err)
	}

	logFileHandle.WriteString("Executing script...\n")

	// Wait for the command to finish
	if err := cmd.Wait(); err != nil {
		return fmt.Errorf("update failed: %w", err)
	} else {
		logFileHandle.WriteString("UPDATE COMPLETED SUCCESSFULLY\n")
		return nil
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

// Windows-specific helper functions

func getSystemLogFilePath() (string, error) {
	logDir := "C:\\ProgramData\\wazuh\\logs"

	if err := os.MkdirAll(logDir, 0755); err != nil {
		return "", fmt.Errorf("failed to create log directory: %v", err)
	}

	return filepath.Join(logDir, "wazuh-agent-status.log"), nil
}

func getLocalVersion() string {
	versionPath, err := getVersionFilePath()
	if err != nil {
		log.Printf("Failed to get version file path on Windows: %v", err)
		return "Unknown"
	}
	output, err := os.ReadFile(versionPath)
	if err != nil {
		log.Printf("Failed to read local version on Windows: %v", err)
		return "Unknown"
	}
	return strings.TrimSpace(string(output))
}

func getBasePath() (string, error) {
	return "C:\\Program Files (x86)\\ossec-agent", nil
}

func getMergedMgPath() (string, error) {
	basePath, err := getBasePath()
	if err != nil {
		return "", err
	}
	return filepath.Join(basePath, "shared", "merged.mg"), nil
}

func getVersionFilePath() (string, error) {
	basePath, err := getBasePath()
	if err != nil {
		return "", err
	}
	return filepath.Join(basePath, "version.txt"), nil
}

func getAdorsysUpdatePath() (string, error) {
	basePath, err := getBasePath()
	if err != nil {
		return "", err
	}
	return filepath.Join(basePath, "active-response", "bin", "adorsys-update.bat"), nil
}

func getWazuhStatePath() (string, error) {
	basePath, err := getBasePath()
	if err != nil {
		return "", err
	}
	return filepath.Join(basePath, "wazuh-agent.state"), nil
}
