//go:build windows
// +build windows

package main

import (
	"encoding/json"
	"fmt"
	"log"
	"os"
	"path/filepath"
	"time"

	"github.com/go-toast/toast"
	"github.com/kardianos/service"
	"gopkg.in/natefinch/lumberjack.v2"
)

// Version is set at build time via ldflags
var Version = "dev"

// UpdateStatus represents the status file structure
type UpdateStatus struct {
	Status    string    `json:"status"`
	Message   string    `json:"message,omitempty"`
	Timestamp time.Time `json:"timestamp"`
}

const (
	statusFilePath = `C:\ProgramData\WazuhAgent\update_status.json`
	iconPath       = `C:\Program Files\wazuh-agent-status\wazuh-logo.png`
	appID          = "ADORSYS.WazuhUpdateNotifier"
)

// program implements the service.Interface
type program struct {
	logger *log.Logger
}

func main() {
	// Handle version flag
	for _, arg := range os.Args[1:] {
		if arg == "--version" || arg == "-v" {
			fmt.Println(Version)
			return
		}
	}

	// Set up logging
	logDir := filepath.Join(os.Getenv("APPDATA"), "wazuh", "logs")
	if err := os.MkdirAll(logDir, 0755); err != nil {
		log.Fatalf("Failed to create log directory: %v", err)
	}

	logFile := filepath.Join(logDir, "wazuh-update-notifier.log")
	logger := log.New(&lumberjack.Logger{
		Filename:   logFile,
		MaxSize:    10, // MB
		MaxBackups: 3,
		MaxAge:     30, // days
		Compress:   true,
	}, "", log.LstdFlags)

	// Define service configuration
	svcConfig := &service.Config{
		Name:        "WazuhUpdateNotifier",
		DisplayName: "Wazuh Update Notifier",
		Description: "Displays toast notifications for Wazuh agent updates",
	}

	prg := &program{
		logger: logger,
	}

	// Create service
	s, err := service.New(prg, svcConfig)
	if err != nil {
		logger.Fatalf("Failed to create service: %v", err)
	}

	// Handle service control commands
	if len(os.Args) > 1 {
		switch os.Args[1] {
		case "install":
			err = s.Install()
			if err != nil {
				logger.Fatalf("Failed to install service: %v", err)
			}
			logger.Println("Service installed successfully")
			return
		case "uninstall":
			err = s.Uninstall()
			if err != nil {
				logger.Fatalf("Failed to uninstall service: %v", err)
			}
			logger.Println("Service uninstalled successfully")
			return
		case "start":
			err = s.Start()
			if err != nil {
				logger.Fatalf("Failed to start service: %v", err)
			}
			logger.Println("Service started successfully")
			return
		case "stop":
			err = s.Stop()
			if err != nil {
				logger.Fatalf("Failed to stop service: %v", err)
			}
			logger.Println("Service stopped successfully")
			return
		}
	}

	// Run the service
	err = s.Run()
	if err != nil {
		logger.Fatalf("Service run failed: %v", err)
	}
}

// Start implements service.Interface
func (p *program) Start(s service.Service) error {
	p.logger.Println("Wazuh Update Notifier service starting...")
	go p.run()
	return nil
}

// run contains the main service logic
func (p *program) run() {
	p.logger.Println("Wazuh Update Notifier service started")

	// Monitor the status file
	lastStatus := ""
	checkInterval := 2 * time.Second
	maxRunTime := 10 * time.Minute // Service will stop after 10 minutes of monitoring

	startTime := time.Now()

	for {
		// Check if we've exceeded max run time
		if time.Since(startTime) > maxRunTime {
			p.logger.Println("Max run time exceeded, service will stop")
			break
		}

		// Check if status file exists
		if _, err := os.Stat(statusFilePath); os.IsNotExist(err) {
			time.Sleep(checkInterval)
			continue
		}

		// Read status file
		status, err := p.readStatusFile()
		if err != nil {
			p.logger.Printf("Error reading status file: %v", err)
			time.Sleep(checkInterval)
			continue
		}

		// Only process if status has changed
		if status.Status != lastStatus {
			p.logger.Printf("Status changed: %s -> %s", lastStatus, status.Status)
			lastStatus = status.Status

			// Show notification based on status
			if err := p.showNotification(status); err != nil {
				p.logger.Printf("Error showing notification: %v", err)
			}

			// If status is success or error, we can stop monitoring
			if status.Status == "success" || status.Status == "error" {
				p.logger.Printf("Update completed with status: %s", status.Status)
				time.Sleep(5 * time.Second) // Give time for notification to be seen
				break
			}
		}

		time.Sleep(checkInterval)
	}

	p.logger.Println("Monitoring completed, service will stop")
}

// Stop implements service.Interface
func (p *program) Stop(s service.Service) error {
	p.logger.Println("Wazuh Update Notifier service stopping...")
	return nil
}

// readStatusFile reads and parses the status file
func (p *program) readStatusFile() (*UpdateStatus, error) {
	data, err := os.ReadFile(statusFilePath)
	if err != nil {
		return nil, fmt.Errorf("failed to read status file: %w", err)
	}

	var status UpdateStatus
	if err := json.Unmarshal(data, &status); err != nil {
		return nil, fmt.Errorf("failed to parse status file: %w", err)
	}

	return &status, nil
}

// showNotification displays a Windows toast notification
func (p *program) showNotification(status *UpdateStatus) error {
	var title, message string

	switch status.Status {
	case "started":
		title = "Wazuh Update"
		message = "Wazuh agent update has started..."
	case "downloading":
		title = "Wazuh Update"
		message = "Downloading Wazuh agent update..."
	case "installing":
		title = "Wazuh Update"
		message = "Installing Wazuh agent update..."
	case "success":
		title = "Wazuh Update Complete"
		message = "Wazuh agent has been updated successfully!"
	case "error":
		title = "Wazuh Update Failed"
		if status.Message != "" {
			message = fmt.Sprintf("Update failed: %s", status.Message)
		} else {
			message = "Wazuh agent update failed. Check logs for details."
		}
	default:
		title = "Wazuh Update"
		message = fmt.Sprintf("Update status: %s", status.Status)
	}

	// Log the notification
	p.logger.Printf("Showing notification - Title: %s, Message: %s", title, message)

	// Create toast notification
	notification := toast.Notification{
		AppID:   appID,
		Title:   title,
		Message: message,
	}

	// Add icon if it exists
	if _, err := os.Stat(iconPath); err == nil {
		notification.Icon = iconPath
	} else {
		p.logger.Printf("Warning: Icon file not found at %s", iconPath)
	}

	// Show the notification
	if err := notification.Push(); err != nil {
		return fmt.Errorf("failed to push notification: %w", err)
	}

	// Log to file
	p.logEvent(status)

	return nil
}

// logEvent logs the update event to a file
func (p *program) logEvent(status *UpdateStatus) {
	logEntry := fmt.Sprintf("[%s] Status: %s", status.Timestamp.Format(time.RFC3339), status.Status)
	if status.Message != "" {
		logEntry += fmt.Sprintf(", Message: %s", status.Message)
	}
	p.logger.Println(logEntry)
}
