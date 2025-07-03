package main

import (
	"bufio"
	"embed"
	"fmt"
	"log"
	"net"

	"os"
	"path/filepath"
	"runtime"
	"strings"
	"time"

	"github.com/getlantern/systray"
	"gopkg.in/natefinch/lumberjack.v2"
)

//go:embed assets/*
var embeddedFiles embed.FS

const (
	backendAddr = "localhost:50505"
	statusActive = "Active"
	statusInactive = "Inactive"
	connectionConnected = "Connected"
	connectionDisconnected = "Disconnected"
	updateDisable = "Disable"
	versionUpToDate = "Up to date"
	versionOutdated = "Outdated"

	errConnectBackend = "Failed to connect to backend: %v"
	errReadResponse   = "Failed to read response: %v"
)

var (
	statusItem, connectionItem, updateItem, versionItem *systray.MenuItem
	enabledIcon, disabledIcon                           []byte
	isMonitoringUpdate                                  bool
)

func getUserLogFilePath() string {
	var logDir string
	home := os.Getenv("HOME")
	switch runtime.GOOS {
	case "linux":
		logDir = filepath.Join(home, ".wazuh")
	case "darwin":
		logDir = filepath.Join(home, "Library", "Logs", "wazuh")
	case "windows":
		logDir = filepath.Join(os.Getenv("APPDATA"), "wazuh", "logs")
	default:
		logDir = "./logs"
	}
	if err := os.MkdirAll(logDir, 0755); err != nil {
		log.Fatalf("failed to create log directory: %v", err)
	}
	return filepath.Join(logDir, "wazuh-agent-status-client.log")
}

// Set up log rotation using lumberjack
func init() {
	logFilePath := getUserLogFilePath()
	log.SetOutput(&lumberjack.Logger{
		Filename:   logFilePath,
		MaxSize:    10,
		MaxBackups: 3,
		MaxAge:     30,
		Compress:   true,
	})
	log.SetFlags(log.LstdFlags | log.Lshortfile)
}

// Main entry point
func main() {
	log.Println("Starting frontend...")
	systray.Run(onReady, onExit)
}

// onReady sets up the tray icon
func onReady() {
	defer recoverGoroutine("onReady")
	// Load main icon
	mainIcon, err := getEmbeddedFile(getIconPath())
	if err != nil {
		log.Fatalf("Failed to load main icon: %v", err)
	}
	systray.SetIcon(mainIcon)
	systray.SetTooltip("Wazuh Agent Status")

	// Load status icons
	enabledIcon, err = getEmbeddedFile("assets/green-dot.png")
	if err != nil {
		log.Printf("Failed to load enabled icon: %v", err)
	}
	disabledIcon, err = getEmbeddedFile("assets/gray-dot.png")
	if err != nil {
		log.Printf("Failed to load disabled icon: %v", err)
	}

	// Create menu items
	statusItem = systray.AddMenuItem("Agent: Unknown", "Wazuh Agent Status")
	connectionItem = systray.AddMenuItem("Connection: Unknown", "Wazuh Agent Connection")
	systray.AddSeparator()
	updateItem = systray.AddMenuItem("---", "Update the Wazuh Agent")
	systray.AddSeparator()
	versionItem = systray.AddMenuItem("v---", "The version state of the wazuh setup")

	statusItem.Disable()
	connectionItem.Disable()
	updateItem.Disable()

	// Start background status update
	go monitorStatus()
	// Handle menu item clicks
	go handleMenuActions()
}

// monitorStatus continuously fetches and updates the agent status
func monitorStatus() {
	go func() {
		defer recoverGoroutine("monitorStatus:status")
		statusTicker := time.NewTicker(5 * time.Second)
		versionTicker := time.NewTicker(6 * time.Hour)
		defer statusTicker.Stop()
		defer versionTicker.Stop()

		// Initial check
		status, connection := fetchStatus()
		setStatusMenu(status)
		setConnectionMenu(connection)
		checkVersion()

		for {
			select {
			case <-statusTicker.C:
				status, connection := fetchStatus()
				setStatusMenu(status)
				setConnectionMenu(connection)
			case <-versionTicker.C:
				checkVersion()
			}
		}
	}()
}

func setStatusMenu(status string) {
	if status == statusActive {
		statusItem.SetTitle("Agent: Active")
		statusItem.SetIcon(enabledIcon)
	} else {
		statusItem.SetTitle("Agent: Inactive")
		statusItem.SetIcon(disabledIcon)
	}
}

func setConnectionMenu(connection string) {
	if connection == connectionConnected {
		connectionItem.SetTitle("Connection: Connected")
		connectionItem.SetIcon(enabledIcon)
	} else {
		connectionItem.SetTitle("Connection: Disconnected")
		connectionItem.SetIcon(disabledIcon)
	}
}

func checkVersion() {
	versionStatus, version := fetchVersionStatus()
	switch {
	case strings.HasPrefix(versionStatus, versionUpToDate):
		versionItem.SetTitle(version)
		versionItem.Disable()
		updateItem.SetTitle(versionUpToDate)
		updateItem.Disable()
	case strings.HasPrefix(versionStatus, versionOutdated):
		versionItem.SetTitle(version)
		versionItem.Disable()
		updateItem.SetTitle("Update")
		updateItem.Enable()
		log.Println("Update launched")
		startUpdateMonitor()
	default:
		versionItem.SetTitle("Version: Unknown")
		versionItem.Disable()
		updateItem.Disable()
	}
}

func checkVersionAfterUpdate() {
	versionStatus, version := fetchVersionStatus()
	switch {
	case strings.HasPrefix(versionStatus, versionUpToDate):
		versionItem.SetTitle(version)
		versionItem.Disable()
		updateItem.SetTitle(versionUpToDate)
		updateItem.Disable()
	case strings.HasPrefix(versionStatus, versionOutdated):
		versionItem.SetTitle(version)
		versionItem.Disable()
		updateItem.SetTitle("Update")
		updateItem.Enable()
	default:
		versionItem.SetTitle("Version: Unknown")
		versionItem.Disable()
		updateItem.Disable()
	}
}

func fetchVersionStatus() (string, string) {
	conn, err := net.Dial("tcp", backendAddr)
	if err != nil {
		log.Printf(errConnectBackend, err)
		return "Unknown", "Unknown"
	}
	defer conn.Close()

	fmt.Fprintln(conn, "check-version")
	reader := bufio.NewReader(conn)
	response, err := reader.ReadString('\n')
	if err != nil {
		log.Printf(errReadResponse, err)
		return "Unknown", "Unknown"
	}

	response = strings.TrimSuffix(response, "\n")
	parts := strings.Split(response, ": ")
	if len(parts) < 2 {
		return "Unknown", "Unknown"
	}

	parts = strings.Split(parts[1], ", ")
	if len(parts) < 2 {
		return "Unknown", "Unknown"
	}
	return parts[0], parts[1]
}

// startUpdateMonitor starts the update status monitoring if not already active
func startUpdateMonitor() {
	if isMonitoringUpdate {
		log.Println("Update monitoring is already running.")
		return
	}
	isMonitoringUpdate = true
	sendCommand("update")
	go monitorUpdateStatus()
}

// monitorUpdateStatus continuously fetches and updates the update status
func monitorUpdateStatus() {
	defer recoverGoroutine("monitorUpdateStatus")
	for isMonitoringUpdate {
		updateStatus := fetchUpdateStatus()
		if updateStatus == updateDisable {
			log.Println("Update status is disabled. Stopping monitoring.")
			isMonitoringUpdate = false
			checkVersionAfterUpdate()
		} else {
			log.Printf("Current update status: %v", updateStatus)
			updateItem.SetTitle("Updating...")
			updateItem.Disable()
		}
		time.Sleep(5 * time.Second)
	}
	isMonitoringUpdate = false
}

// handleMenuActions listens for menu item clicks and performs actions
func handleMenuActions() {
	defer recoverGoroutine("handleMenuActions")
	for range updateItem.ClickedCh {
		log.Println("Update clicked")
		startUpdateMonitor()
	}
}

// fetchStatus retrieves the agent status and connection state from the backend
func fetchStatus() (string, string) {
	conn, err := net.Dial("tcp", backendAddr)
	if err != nil {
		log.Printf(errConnectBackend, err)
		return "Unknown", "Unknown"
	}
	defer conn.Close()

	fmt.Fprintln(conn, "status")
	reader := bufio.NewReader(conn)
	response, err := reader.ReadString('\n')
	if err != nil {
		log.Printf(errReadResponse, err)
		return "Unknown", "Unknown"
	}
	response = strings.TrimSuffix(response, "\n")
	parts := strings.Split(response, ", ")
	if len(parts) < 2 {
		return "Unknown", "Unknown"
	}
	status, ok1 := strings.CutPrefix(parts[0], "Status: ")
	connection, ok2 := strings.CutPrefix(parts[1], "Connection: ")
	if !ok1 || !ok2 {
		return "Unknown", "Unknown"
	}
	return status, connection
}

// fetchUpdateStatus retrieves the update status from the backend
func fetchUpdateStatus() string {
	conn, err := net.Dial("tcp", backendAddr)
	if err != nil {
		log.Printf(errConnectBackend, err)
		return "Unknown"
	}
	defer conn.Close()

	fmt.Fprintln(conn, "update-status")
	reader := bufio.NewReader(conn)
	response, err := reader.ReadString('\n')
	if err != nil {
		log.Printf(errReadResponse, err)
		return "Unknown"
	}
	response = strings.TrimSuffix(response, "\n")
	log.Printf("Update: %v", response)
	parts := strings.SplitN(response, ": ", 2)
	if len(parts) < 2 {
		return "Unknown"
	}
	return parts[1]
}

// sendCommand sends a command (e.g., update) to the backend
func sendCommand(command string) {
	conn, err := net.Dial("tcp", backendAddr)
	if err != nil {
		log.Printf(errConnectBackend, err)
		return
	}
	defer conn.Close()
	fmt.Fprintln(conn, command)
}
// recoverGoroutine logs panics in goroutines for easier debugging
func recoverGoroutine(context string) {
	if r := recover(); r != nil {
		log.Printf("Panic in goroutine [%s]: %v", context, r)
	}
}

// getEmbeddedFile reads a file from the embedded file system
func getEmbeddedFile(path string) ([]byte, error) {
	return embeddedFiles.ReadFile(path)
}

// getIconPath returns iconpath based on the OS
func getIconPath() string {
	switch os := runtime.GOOS; os {
	case "windows":
		return "assets/wazuh-logo.ico" // Path to the ICO icon for Windows
	default:
		return "assets/wazuh-logo.png" // Default icon path
	}
}

// onExit is called when the application is terminated
func onExit() {
	log.Println("Frontend application stopped")
}
