//go:build linux
// +build linux

package main

import (
	"fmt"
	"log"
	"net"
	"os"
	"os/exec"
	"path/filepath"
	"strings"
)

// checkServiceStatus checks the status of Wazuh agent and its connection on Linux
func checkServiceStatus() (string, string) {
	cmd := exec.Command(sudoCommand, "/var/ossec/bin/wazuh-control", "status")
	output, err := cmd.CombinedOutput()
	if err != nil {
		log.Printf("Error checking Wazuh agent status: %v\nOutput: %s", err, string(output))
		return "Inactive", "Disconnected"
	}

	status := "Inactive"
	if strings.Contains(string(output), "wazuh-agentd is running") {
		status = "Active"
	}

	connCmd := exec.Command(sudoCommand, grepCommand, "^status", "/var/ossec/var/run/wazuh-agentd.state")
	connOutput, connErr := connCmd.CombinedOutput()
	connection := "Disconnected"
	if connErr != nil {
		log.Printf("Error checking Wazuh agent connection: %v\nOutput: %s", connErr, string(connOutput))
	} else if strings.Contains(string(connOutput), "status='connected'") {
		connection = "Connected"
	}

	return status, connection
}

// updateAgent updates the Wazuh agent on Linux and streams progress to the client
func updateAgent(conn net.Conn, isPrerelease bool) {
	// The caller (handleConnection) closes the dedicated update stream conn when this function returns
	defer conn.Close()

	// Helper to write status updates directly to the connection
	writeUpdate := func(status string) {
		conn.Write([]byte(fmt.Sprintf("UPDATE_PROGRESS: %s\n", status)))
		log.Printf("Update progress: %s", status)
	}

	writeUpdate("Starting...")

	// Create log file for troubleshooting
	logFile := "/tmp/wazuh-update.log"
	logFileHandle, err := os.Create(logFile)
	if err != nil {
		writeUpdate(fmt.Sprintf("ERROR: Failed to create log file: %v", err))
		return
	}
	defer logFileHandle.Close()

	logFileHandle.Sync() // Force write to disk

	writeUpdate(fmt.Sprintf("Logging to: %s", logFile))

	var cmd *exec.Cmd
	if isPrerelease {
		// For prerelease, download and execute setup script directly
		versionInfo := fetchVersionInfo()
		if versionInfo != nil && versionInfo.Framework.PrereleaseVersion != "" {
			prereleaseScriptURL := fmt.Sprintf("https://raw.githubusercontent.com/ADORSYS-GIS/wazuh-agent/refs/tags/v%s/scripts/setup-agent.sh", versionInfo.Framework.PrereleaseVersion)

			writeUpdate(fmt.Sprintf("prereleaseScriptURL: %s", prereleaseScriptURL))
			logFileHandle.WriteString(fmt.Sprintf("Prerelease Script URL: %s\n", prereleaseScriptURL))

			writeUpdate(fmt.Sprintf("Downloading prerelease script from: %s", versionInfo.Framework.PrereleaseVersion))

			// Create a temporary directory for the script
			tmpDir := "/tmp/wazuh-prerelease"
			if err := os.MkdirAll(tmpDir, 0755); err != nil {
				writeUpdate(fmt.Sprintf("ERROR: Failed to create temp directory: %v", err))
				logFileHandle.WriteString(fmt.Sprintf("ERROR: Failed to create temp directory: %v\n", err))
				logFileHandle.Sync()
				return
			}
			defer os.RemoveAll(tmpDir)

			// Download the prerelease setup script
			scriptPath := filepath.Join(tmpDir, "setup-agent.sh")
			if err := downloadFile(prereleaseScriptURL, scriptPath); err != nil {
				writeUpdate(fmt.Sprintf("ERROR: Failed to download script: %v", err))
				logFileHandle.WriteString(fmt.Sprintf("ERROR: Failed to download script: %v\n", err))
				logFileHandle.Sync()
				return
			}

			// Make script executable and run it
			if err := os.Chmod(scriptPath, 0755); err != nil {
				writeUpdate(fmt.Sprintf("ERROR: Failed to make script executable: %v", err))
				logFileHandle.WriteString(fmt.Sprintf("ERROR: Failed to make script executable: %v\n", err))
				logFileHandle.Sync()
				return
			}

			cmd = exec.Command(sudoCommand, "bash", scriptPath)
			logFileHandle.WriteString(fmt.Sprintf("Executing: %s %s %s\n", sudoCommand, "bash", scriptPath))
			logFileHandle.Sync()
		} else {
			writeUpdate(fmt.Sprintf("ERROR: Empty prerelease"))
			logFileHandle.WriteString(fmt.Sprintf("ERROR: Empty prerelease"))
			logFileHandle.Sync()
		}
	} else {
		cmd = exec.Command(sudoCommand, "/var/ossec/active-response/bin/adorsys-update.sh")
		logFileHandle.WriteString(fmt.Sprintf("Executing: %s %s\n", sudoCommand, "/var/ossec/active-response/bin/adorsys-update.sh"))
		logFileHandle.Sync()
	}

	// Stream stdout and stderr ONLY to the update log file
	cmd.Stdout = logFileHandle
	cmd.Stderr = logFileHandle

	if err := cmd.Start(); err != nil {
		writeUpdate(fmt.Sprintf("ERROR: Command failed to start: %v", err))
		logFileHandle.WriteString(fmt.Sprintf("ERROR: Command failed to start: %v\n", err))
		logFileHandle.Sync()
		return
	}

	writeUpdate("Executing script...")
	logFileHandle.WriteString("Executing script...\n")

	// Wait for the command to finish
	if err := cmd.Wait(); err != nil {
		logFilePath := "/var/ossec/logs/active-responses.log"
		errorMessage := fmt.Sprintf("ERROR: Update failed: %v. Check logs at %s", err, logFilePath)
		writeUpdate("Error")
		logFileHandle.WriteString(fmt.Sprintf("UPDATE FAILED: %v\n", err))
		logFileHandle.WriteString(fmt.Sprintf("Additional logs available at: %s\n", logFilePath))
		logFileHandle.Sync() // Force write before closing
		log.Println(errorMessage)
	} else {
		writeUpdate("Complete")
		logFileHandle.WriteString("UPDATE COMPLETED SUCCESSFULLY\n")
		logFileHandle.Sync() // Force write before closing
		log.Println("Wazuh agent updated successfully")
	}
}

func windowsMain() {
	// This function is intentionally left empty for Linux builds.
}
