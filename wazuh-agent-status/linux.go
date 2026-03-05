//go:build linux
// +build linux

package main

import (
	"fmt"
	"io"
	"log"
	"net"
	"os"
	"os/exec"
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

	var cmd *exec.Cmd
	if isPrerelease {
		cmd = exec.Command(sudoCommand, "/var/ossec/active-response/bin/adorsys-update.sh", "--no-confirm")
	} else {
		cmd = exec.Command(sudoCommand, "/var/ossec/active-response/bin/adorsys-update.sh")
	}

	// Stream stdout and stderr to the log and to the client connection
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
		logFilePath := "/var/ossec/logs/active-responses.log"
		errorMessage := fmt.Sprintf("ERROR: Update failed: %v. Check logs at %s", err, logFilePath)
		writeUpdate("Error")
		log.Println(errorMessage)
	} else {
		writeUpdate("Complete")
		log.Println("Wazuh agent updated successfully")
	}
}

func windowsMain() {
	// This function is intentionally left empty for Linux builds.
}
