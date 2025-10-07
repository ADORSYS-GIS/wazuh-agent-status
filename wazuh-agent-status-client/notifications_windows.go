//go:build windows
// +build windows

package main

import (
	"log"
	"os/exec"
)

// showWindowsNotification displays a native Windows notification using msg.exe
func showWindowsNotification(title, message string) {
	// Use msg.exe to display a simple message box to the current session
	// The * sends to all sessions, but it will only appear in active desktop sessions
	cmd := exec.Command("cmd", "/C", "msg", "*", "/TIME:30", title+" - "+message)
	err := cmd.Run()
	if err != nil {
		log.Printf("Failed to show Windows notification: %v", err)
	}
}
