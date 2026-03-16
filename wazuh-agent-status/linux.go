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

	writeUpdate := func(status string) {
		conn.Write([]byte(fmt.Sprintf("UPDATE_PROGRESS: %s\n", status)))
		log.Printf("Update progress: %s", status)
	}

	var adorsysUpdatePath, updatePathErr = getAdorsysUpdatePath()
	if updatePathErr != nil {
		writeUpdate("Failed")
		log.Printf("Error getting adorsys update path: %v", updatePathErr)
		return
	}

	writeUpdate("Starting...")

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
		if err := handleRegularUpdate(adorsysUpdatePath); err != nil {
			log.Printf("Error handling regular update: %v", err)
			writeUpdate("Error")
			return
		}
	}

	writeUpdate("Complete")
}

// handleRegularUpdate handles the regular update process
func handleRegularUpdate(adorsysUpdatePath string) error {
	var cmd *exec.Cmd
	cmd = exec.Command(sudoCommand, adorsysUpdatePath)
	if err := cmd.Start(); err != nil {
		return fmt.Errorf("error starting update command: %w", err)
	}

	// Wait for the command to finish
	if err := cmd.Wait(); err != nil {
		return fmt.Errorf("update failed: %w", err)
	}

	return nil
}

func getWazuhControlPath() (string, error) {
	basePath, err := getBasePath()
	if err != nil {
		return "", err
	}

	return filepath.Join(basePath, "bin", "wazuh-control"), nil
}

// handlePrereleaseUpdate handles the prerelease update process for Linux
func handlePrereleaseUpdate(logFileHandle *os.File) error {
	versionInfo := fetchVersionInfo()
	if versionInfo == nil || versionInfo.Framework.PrereleaseVersion == "" {
		return fmt.Errorf("empty prerelease")
	}

	prereleaseScriptURL := fmt.Sprintf("https://raw.githubusercontent.com/ADORSYS-GIS/wazuh-agent/refs/tags/v%s/scripts/setup-agent.sh", versionInfo.Framework.PrereleaseVersion)
	tempFilePattern := "wazuh-prerelease-*.sh"

	tempFile, err := os.CreateTemp("", tempFilePattern)
	if err != nil {
		return fmt.Errorf("failed to create temp log file: %w", err)
	}
	tempFile.Close()

	if err := os.Chmod(tempFile.Name(), 0750); err != nil {
		return fmt.Errorf("failed to set permissions on temp file: %w", err)
	}

	if err := downloadAndSaveFile(prereleaseScriptURL, tempFile.Name(), 0750); err != nil {
		return fmt.Errorf("failed to download prerelease script: %w", err)
	}
	defer os.Remove(tempFile.Name())

	cmd := exec.Command(tempFile.Name())

	// Stream stdout and stderr ONLY to the update log file
	cmd.Stdout = logFileHandle
	cmd.Stderr = logFileHandle

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

// Linux-specific helper functions

func getSystemLogFilePath() (string, error) {
	logDir := "/var/log"
	return filepath.Join(logDir, "wazuh-agent-status.log"), nil
}

func runAsRoot(command string, args ...string) (string, error) {
	cmd := exec.Command(sudoCommand, append([]string{command}, args...)...)
	output, err := cmd.CombinedOutput()
	return string(output), err
}

func getLocalVersion() string {
	versionPath, err := getVersionFilePath()
	if err != nil {
		log.Printf("Failed to get version file path on Linux: %v", err)
		return "Unknown"
	}
	output, err := runAsRoot("cat", versionPath)
	if err != nil {
		log.Printf("Failed to read local version on Linux: %v", err)
		return "Unknown"
	}
	return strings.TrimSpace(output)
}

func getBasePath() (string, error) {
	return "/var/ossec", nil
}

func getMergedMgPath() (string, error) {
	basePath, err := getBasePath()
	if err != nil {
		return "", err
	}
	return filepath.Join(basePath, "etc", "shared", "merged.mg"), nil
}

func getVersionFilePath() (string, error) {
	basePath, err := getBasePath()
	if err != nil {
		return "", err
	}
	return filepath.Join(basePath, "etc", "version.txt"), nil
}

func getAdorsysUpdatePath() (string, error) {
	basePath, err := getBasePath()
	if err != nil {
		return "", err
	}
	return filepath.Join(basePath, "active-response", "bin", "adorsys-update.sh"), nil
}

func getWazuhStatePath() (string, error) {
	basePath, err := getBasePath()
	if err != nil {
		return "", err
	}
	return filepath.Join(basePath, "var", "run", "wazuh-agentd.state"), nil
}

func windowsMain() {
	// This function is intentionally left empty for Linux builds.
}
