//go:build linux || darwin
// +build linux darwin

package main

import (
	"fmt"
	"log"
	"net"
	"os"
	"os/exec"
	"path/filepath"
	"runtime"
	"strings"
)

// checkServiceStatus checks the status of Wazuh agent and its connection on Linux
func checkServiceStatus() (string, string) {
	var wazuhControlPath, controlPathErr = getWazuhControlPath()
	if controlPathErr != nil {
		log.Printf("Error getting Wazuh control path: %v", controlPathErr)
		return "Inactive", "Disconnected"
	}

	var wazuhStatePath, statePathErr = getWazuhStatePath()
	if statePathErr != nil {
		log.Printf("Error getting Wazuh state path: %v", statePathErr)
		return "Inactive", "Disconnected"
	}

	cmd := exec.Command(sudoCommand, wazuhControlPath, "status")
	output, err := cmd.CombinedOutput()
	if err != nil {
		log.Printf("Error checking Wazuh agent status: %v\nOutput: %s", err, string(output))
		return "Inactive", "Disconnected"
	}

	status := "Inactive"
	if strings.Contains(string(output), "wazuh-agentd is running") {
		status = "Active"
	}

	connCmd := exec.Command(sudoCommand, grepCommand, "^status", wazuhStatePath)
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

	var adorsysUpdatePath, updatePathErr = getAdorsysUpdatePath()
	if updatePathErr != nil {
		log.Printf("Error getting adorsys update path: %v", updatePathErr)
		return
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
		handleRegularUpdate(writeUpdate, logFileHandle, adorsysUpdatePath)
	}
}

// handleRegularUpdate handles the regular update process
func handleRegularUpdate(writeUpdate func(string), logFileHandle *os.File, adorsysUpdatePath string) {
	var cmd *exec.Cmd
	cmd = exec.Command(sudoCommand, adorsysUpdatePath)
	if err := cmd.Start(); err != nil {
		log.Printf("Error starting update command: %v", err)
		return
	}

	writeUpdate("Executing script...")

	// Wait for the command to finish
	if err := cmd.Wait(); err != nil {
		errorMessage := fmt.Sprintf("ERROR: Update failed: %v", err)
		writeUpdate("Error")
		logFileHandle.WriteString(errorMessage + "\n")
		log.Println(errorMessage)
	} else {
		writeUpdate("Complete")
		log.Println("Wazuh agent updated successfully")
	}
}

func getWazuhControlPath() (string, error) {
	basePath, err := getBasePath()
	if err != nil {
		return "", err
	}

	switch runtime.GOOS {
	case "linux", "darwin":
		return filepath.Join(basePath, "bin", "wazuh-control"), nil
	default:
		return "", fmt.Errorf("unsupported OS: %s", runtime.GOOS)
	}
}

func windowsMain() {
	// This function is intentionally left empty for Linux builds.
}
