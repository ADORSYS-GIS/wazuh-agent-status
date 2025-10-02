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
	"sync"
	"sync/atomic"
	"time"

	"github.com/getlantern/systray"
	"gopkg.in/natefinch/lumberjack.v2"
)

//go:embed assets/*
var embeddedFiles embed.FS

var (
	statusItem, connectionItem, updateItem, versionItem *systray.MenuItem
	enabledIcon, disabledIcon                           []byte

	// isMonitoringUpdate is an atomic flag (0/1) to indicate a running update monitor
	isMonitoringUpdate int32

	// monitorOnce ensures we only start monitorStatus once
	monitorOnce sync.Once
)

// Version is set at build time via ldflags
var Version = "v0.3.4-user-rc1"

func getUserLogFilePath() string {
	var logDir string

	switch runtime.GOOS {
	case "linux", "darwin":
		logDir = filepath.Join(os.Getenv("HOME"), ".wazuh")
	case "windows":
		logDir = filepath.Join(os.Getenv("APPDATA"), "wazuh", "logs")
	default:
		logDir = "./logs"
	}

	// Ensure the directory exists
	if err := os.MkdirAll(logDir, 0755); err != nil {
		log.Fatalf("failed to create log directory: %v", err)
	}

	return filepath.Join(logDir, "wazuh-agent-status-client.log")
}

func init() {
	logFilePath := getUserLogFilePath()

	log.SetOutput(&lumberjack.Logger{
		Filename:   logFilePath,
		MaxSize:    10,
		MaxBackups: 3,
		MaxAge:     30,
		Compress:   true,
	})
}

// Main entry point
func main() {
	for _, arg := range os.Args[1:] {
		if arg == "--version" || arg == "-v" {
			fmt.Println(Version)
			return
		}
	}
	log.Printf("Starting frontend... (version: %s)", Version)
	systray.Run(onReady, onExit)
}

// onReady sets up the tray icon
func onReady() {
	// Load main icon
	mainIcon, err := getEmbeddedFile(getIconPath())
	if err != nil {
		log.Fatalf("Failed to load main icon: %v", err)
	}
	systray.SetIcon(mainIcon)
	systray.SetTooltip("Wazuh Agent Status")

	// Load status icons.
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
	statusItem.Disable()
	connectionItem = systray.AddMenuItem("Connection: Unknown", "Wazuh Agent Connection")
	connectionItem.Disable()
	systray.AddSeparator()
	updateItem = systray.AddMenuItem("---", "Update the Wazuh Agent")
	updateItem.Disable() // Initially disabled
	systray.AddSeparator()
	versionItem = systray.AddMenuItem("v---", "The version state of the wazuhbsetup")
	versionItem.Disable() // Initially disabled

	// Start background status update (guarded to run only once)
	monitorOnce.Do(func() {
		go monitorStatus()
		// optional goroutine counter for debug
		go goroutineCounter()
	})

	// Handle menu item clicks
	go handleMenuActions()
}

// goroutineCounter logs goroutine count periodically to help detect leaks.
// You can remove this if it's too noisy.
func goroutineCounter() {
	for {
		time.Sleep(1 * time.Minute)
		log.Printf("Debug: goroutines=%d", runtime.NumGoroutine())
	}
}

// monitorStatus continuously fetches and updates the agent status
func monitorStatus() {
	// Status polling loop
	go func() {
		for {
			status, connection := fetchStatus()

			// Update status menu item
			if status == "Active" {
				statusItem.SetTitle("Agent: Active")
				statusItem.SetIcon(enabledIcon)
			} else {
				statusItem.SetTitle("Agent: Inactive")
				statusItem.SetIcon(disabledIcon)
			}

			// Update connection menu item
			if connection == "Connected" {
				connectionItem.SetTitle("Connection: Connected")
				connectionItem.SetIcon(enabledIcon)
			} else {
				connectionItem.SetTitle("Connection: Disconnected")
				connectionItem.SetIcon(disabledIcon)
			}

			// If version hasn't been set yet, ensure it's checked
			v := versionItem.String()
			if v == "v---" || v == "Version: Unknown" || v == "vUnknown" {
				checkVersionAfterUpdate()
			}

			time.Sleep(5 * time.Second)
		}
	}()

	// Long interval version check
	go func() {
		for {
			checkVersion()
			time.Sleep(4 * time.Hour)
		}
	}()
}

func checkVersion() {
	versionStatus, version := fetchVersionStatus()

	if strings.HasPrefix(versionStatus, "Up to date") {
		versionItem.SetTitle(version)
		versionItem.Disable()
		updateItem.SetTitle("Up to date")
		updateItem.Disable()
	} else if strings.HasPrefix(versionStatus, "Outdated") {
		versionItem.SetTitle(version)
		versionItem.Disable()
		updateItem.SetTitle("Update")
		updateItem.Enable()
		log.Println("Version is outdated, starting update monitor...")
		startUpdateMonitor()
	} else {
		versionItem.SetTitle("Version: Unknown")
		versionItem.Disable()
		updateItem.Disable()
	}
}

func checkVersionAfterUpdate() {
	versionStatus, version := fetchVersionStatus()

	if strings.HasPrefix(versionStatus, "Up to date") {
		versionItem.SetTitle(version)
		versionItem.Disable()
		updateItem.SetTitle("Up to date")
		updateItem.Disable()
	} else if strings.HasPrefix(versionStatus, "Outdated") {
		versionItem.SetTitle(version)
		versionItem.Disable()
		updateItem.SetTitle("Update")
		updateItem.Enable()
	} else {
		versionItem.SetTitle("Version: Unknown")
		versionItem.Disable()
		updateItem.Disable()
	}
}

// fetchVersionStatus performs a single request/response and closes the connection immediately.
func fetchVersionStatus() (string, string) {
	conn, err := net.DialTimeout("tcp", "localhost:50505", 3*time.Second)
	if err != nil {
		log.Printf("Failed to connect to backend: %v", err)
		return "Unknown", "Unknown"
	}

	// Ensure we always close the conn explicitly in this function.
	// do not use defer in functions that may be called in tight loops if you expect the function to return quickly -
	// but here it's fine because we close before returning in all branches.
	// however prefer explicit close at the end of the operation:
	// set a read deadline so ReadString doesn't block forever
	_ = conn.SetReadDeadline(time.Now().Add(5 * time.Second))

	_, err = fmt.Fprintln(conn, "check-version")
	if err != nil {
		log.Printf("Failed to send check-version: %v", err)
		conn.Close()
		return "Unknown", "Unknown"
	}

	reader := bufio.NewReader(conn)
	response, err := reader.ReadString('\n')
	// close as soon as read completes or fails
	conn.Close()
	if err != nil {
		log.Printf("Failed to read response: %v", err)
		return "Unknown", "Unknown"
	}

	response = strings.TrimSpace(response)
	// defensive parsing
	parts := strings.SplitN(response, ": ", 2)
	if len(parts) < 2 {
		return "Unknown", "Unknown"
	}

	after := parts[1]
	parts2 := strings.SplitN(after, ", ", 2)
	if len(parts2) < 2 {
		// malformed but return what we have
		if len(parts2) == 1 {
			return parts2[0], "Unknown"
		}
		return "Unknown", "Unknown"
	}
	return parts2[0], parts2[1]
}

// startUpdateMonitor starts the update status monitoring if not already active
func startUpdateMonitor() {
	// Atomically set from 0 -> 1, return if already 1
	if !atomic.CompareAndSwapInt32(&isMonitoringUpdate, 0, 1) {
		log.Println("Update monitoring is already running.")
		return
	}

	// send update command once
	if err := sendCommandAndClose("update"); err != nil {
		log.Printf("Failed to send update command: %v", err)
		atomic.StoreInt32(&isMonitoringUpdate, 0)
		return
	}

	// run monitor in background
	go monitorUpdateStatus()
}

// monitorUpdateStatus continuously fetches and updates the update status
func monitorUpdateStatus() {
	defer atomic.StoreInt32(&isMonitoringUpdate, 0)

	for atomic.LoadInt32(&isMonitoringUpdate) == 1 {
		updateStatus := fetchUpdateStatus()

		// If the update status is "Disable", stop monitoring
		if strings.EqualFold(updateStatus, "Disable") {
			log.Println("Update status is disabled. Stopping monitoring.")
			atomic.StoreInt32(&isMonitoringUpdate, 0)
			checkVersionAfterUpdate()
			break
		} else if updateStatus == "Unknown" {
			// keep the monitoring alive but show updating state
			updateItem.SetTitle("Updating... (unknown)")
			updateItem.Disable()
		} else {
			log.Printf("Current update status: %v", updateStatus)
			// Update the icon or text based on the update status
			updateItem.SetTitle("Updating...")
			updateItem.Disable()
		}

		// Sleep for a short period before checking again
		time.Sleep(5 * time.Second)
	}
}

// handleMenuActions listens for menu item clicks and performs actions
func handleMenuActions() {
	for range updateItem.ClickedCh {
		log.Println("Update clicked")
		startUpdateMonitor()
	}
}

// fetchStatus retrieves the agent status and connection state from the backend
func fetchStatus() (string, string) {
	conn, err := net.DialTimeout("tcp", "localhost:50505", 3*time.Second)
	if err != nil {
		log.Printf("Failed to connect to backend: %v", err)
		return "Unknown", "Unknown"
	}
	_ = conn.SetReadDeadline(time.Now().Add(5 * time.Second))

	_, err = fmt.Fprintln(conn, "status")
	if err != nil {
		log.Printf("Failed to send status request: %v", err)
		conn.Close()
		return "Unknown", "Unknown"
	}

	reader := bufio.NewReader(conn)
	response, err := reader.ReadString('\n')
	// close immediately after ReadString
	conn.Close()
	if err != nil {
		log.Printf("Failed to read response: %v", err)
		return "Unknown", "Unknown"
	}

	response = strings.TrimSpace(response)
	// defensive splitting
	parts := strings.Split(response, ", ")
	if len(parts) < 2 {
		return "Unknown", "Unknown"
	}

	// Extract the values safely
	var statusVal, connVal string
	if len(parts) >= 1 {
		p := strings.SplitN(parts[0], ": ", 2)
		if len(p) == 2 {
			statusVal = p[1]
		}
	}
	if len(parts) >= 2 {
		p := strings.SplitN(parts[1], ": ", 2)
		if len(p) == 2 {
			connVal = p[1]
		}
	}

	if statusVal == "" {
		statusVal = "Unknown"
	}
	if connVal == "" {
		connVal = "Unknown"
	}

	return statusVal, connVal
}

// fetchUpdateStatus retrieves the update status from the backend
func fetchUpdateStatus() string {
	conn, err := net.DialTimeout("tcp", "localhost:50505", 3*time.Second)
	if err != nil {
		log.Printf("Failed to connect to backend: %v", err)
		return "Unknown"
	}
	_ = conn.SetReadDeadline(time.Now().Add(5 * time.Second))

	_, err = fmt.Fprintln(conn, "update-status")
	if err != nil {
		log.Printf("Failed to send update-status: %v", err)
		conn.Close()
		return "Unknown"
	}

	reader := bufio.NewReader(conn)
	response, err := reader.ReadString('\n')
	// close immediately after read
	conn.Close()
	if err != nil {
		log.Printf("Failed to read response: %v", err)
		return "Unknown"
	}

	response = strings.TrimSpace(response)
	log.Printf("Update: %v", response)

	parts := strings.SplitN(response, ": ", 2)
	if len(parts) < 2 {
		return "Unknown"
	}
	return parts[1]
}

// sendCommandAndClose sends a command and closes the connection explicitly.
func sendCommandAndClose(command string) error {
	conn, err := net.DialTimeout("tcp", "localhost:50505", 3*time.Second)
	if err != nil {
		return err
	}
	// set a short write deadline to avoid hanging
	_ = conn.SetWriteDeadline(time.Now().Add(3 * time.Second))
	_, err = fmt.Fprintln(conn, command)
	// close immediately after send
	_ = conn.Close()
	return err
}

// getEmbeddedFile reads a file from the embedded file system
func getEmbeddedFile(path string) ([]byte, error) {
	return embeddedFiles.ReadFile(path)
}

// getIconPath returns iconpath based on the OS
func getIconPath() string {
	switch os := runtime.GOOS; os {
	case "windows":
		_ = os // silence unused variable in switch
		return "assets/wazuh-logo.ico" // Path to the ICO icon for Windows
	default:
		return "assets/wazuh-logo.png" // Default icon path
	}
}

// onExit is called when the application is terminated
func onExit() {
	log.Println("Frontend application stopped")
}
