//go:build windows
// +build windows

package main

import (
	"log"
	"os/exec"
	"strings"
	"time"
)

// checkServiceStatus checks the status of Wazuh agent and its connection on Windows
func checkServiceStatus() (string, string) {
	cmd := exec.Command("cmd", "/C", `C:\Program Files (x86)\ossec\bin\wazuh-control status`)
	output, err := cmd.Output()
	if err != nil {
		return "Inactive", "Disconnected"
	}

	status := "Inactive"
	if strings.Contains(string(output), "wazuh-agentd is running") {
		status = "Active"
	}

	// Check connection status
	connCmd := exec.Command("cmd", "/C", `powershell -Command "Select-String -Path 'C:\Program Files (x86)\ossec-agent\wazuh-agent.state' -Pattern '^status'"`)
	connOutput, connErr := connCmd.Output()
	connection := "Disconnected"
	if connErr == nil && strings.Contains(string(connOutput), "status='connected'") {
		connection = "Connected"
	}

	return status, connection
}

// pauseAgent pauses the Wazuh agent on Windows
func pauseAgent() {
	log.Printf("[%s] Pausing Wazuh agent...\n", time.Now().Format(time.RFC3339))
	err := exec.Command("net", "stop", "wazuh-agent").Run()
	if err != nil {
		log.Printf("[%s] Failed to pause Wazuh agent: %v\n", time.Now().Format(time.RFC3339), err)
	} else {
		log.Printf("[%s] Wazuh agent paused successfully\n", time.Now().Format(time.RFC3339))
	}
}

// restartAgent restarts the Wazuh agent on Windows
func restartAgent() {
	log.Printf("[%s] Restarting Wazuh agent...\n", time.Now().Format(time.RFC3339))
	err := exec.Command("net", "start", "wazuh-agent").Run()
	if err != nil {
		log.Printf("[%s] Failed to restart Wazuh agent: %v\n", time.Now().Format(time.RFC3339), err)
	} else {
		log.Printf("[%s] Wazuh agent restarted successfully\n", time.Now().Format(time.RFC3339))
	}
}
