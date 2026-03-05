package main

import (
	"bufio"
	"embed"
	"fmt"
	"io"
	"log"
	"net"
	"os"
	"path/filepath"
	"regexp"
	"runtime"
	"strings"
	"sync"
	"time"

	"fyne.io/systray"
	"gopkg.in/natefinch/lumberjack.v2"
)

const (
	backendAddress = "localhost:50506"
)

//go:embed assets/*
var embeddedFiles embed.FS

var (
	statusItem, connectionItem, updateItem, versionItem *systray.MenuItem
	enabledIcon, disabledIcon                           []byte

	// monitorOnce ensures we only start the monitoring routines once
	monitorOnce sync.Once

	// updateMutex prevents concurrent attempts to start the update stream
	updateMutex sync.Mutex

	// prereleaseShown tracks if we've already shown the prerelease notification
	prereleaseShown bool
	prereleaseMutex sync.Mutex

	// Prerelease notification menu items
	prereleaseItem, prereleaseUpdateItem *systray.MenuItem
	prereleaseNotificationActive         bool
)

var titleRegexp = regexp.MustCompile(`"(.*?)"`)
var versionRegex = regexp.MustCompile(`^v?\d+\.\d+\.\d+(-rc[.\d]*)?$`)

// Version is set at build time via ldflags
var Version = "dev"

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
	// Load icons and set up systray
	mainIcon, err := getEmbeddedFile(getIconPath())
	if err != nil {
		log.Fatalf("Failed to load main icon: %v", err)
	}
	systray.SetIcon(mainIcon)
	systray.SetTooltip("Wazuh Agent Status")

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

	// Start background monitoring routines (guarded to run only once)
	monitorOnce.Do(func() {
		go monitorStatusStream()
		go monitorVersion()
		go goroutineCounter()
	})

	// Handle menu item clicks
	go handleMenuActions()
}

// goroutineCounter logs goroutine count periodically to help detect leaks.
func goroutineCounter() {
	for {
		time.Sleep(1 * time.Minute)
		log.Printf("Debug: goroutines=%d", runtime.NumGoroutine())
	}
}

// establishConnection creates and initializes a connection to the backend
func establishConnection() (net.Conn, error) {
	conn, err := net.DialTimeout("tcp", backendAddress, 5*time.Second)
	if err != nil {
		return nil, err
	}

	log.Println("Status stream connected. Sending subscription...")
	fmt.Fprintln(conn, "subscribe-status")
	return conn, nil
}

// parseStatusResponse processes incoming status messages and updates UI accordingly
func parseStatusResponse(response string) (shouldContinue bool) {
	response = strings.TrimSpace(response)
	if strings.HasPrefix(response, "STATUS_UPDATE:") {
		parts := strings.SplitN(response, ": ", 2)
		if len(parts) > 1 {
			data := strings.SplitN(parts[1], ", ", 2)
			if len(data) == 2 {
				updateStatusItems(data[0], data[1])
				return true
			}
			log.Printf("Unexpected STATUS_UPDATE data format: %q", parts[1])
		} else {
			log.Printf("Unexpected STATUS_UPDATE format: %q", response)
		}
	} else if strings.HasPrefix(response, "ERROR:") {
		log.Printf("Error from status stream: %s", response)
		return false
	}
	return true
}

// monitorStatusStream establishes a persistent connection to receive status updates
func monitorStatusStream() {
	for {
		conn, err := establishConnection()
		if err != nil {
			log.Printf("Failed to connect to backend for status stream: %v. Retrying in 5s...", err)
			updateStatusItems("Unknown", "Unknown")
			time.Sleep(5 * time.Second)
			continue
		}

		if !processStatusStream(conn) {
			conn.Close()
		}
		time.Sleep(5 * time.Second)
	}
}

// processStatusStream handles the reading loop for an established connection
func processStatusStream(conn net.Conn) bool {
	reader := bufio.NewReader(conn)
	for {
		response, err := reader.ReadString('\n')
		if err != nil {
			log.Printf("Status stream closed by server or error: %v. Reconnecting...", err)
			return false
		}

		if !parseStatusResponse(response) {
			return false
		}
	}
}

// updateStatusItems safely updates the systray menu items.
func updateStatusItems(status, connection string) {
	if status == "Active" {
		statusItem.SetTitle("Agent: Active")
		statusItem.SetIcon(enabledIcon)
	} else {
		statusItem.SetTitle("Agent: Inactive")
		statusItem.SetIcon(disabledIcon)
	}

	if connection == "Connected" {
		connectionItem.SetTitle("Connection: Connected")
		connectionItem.SetIcon(enabledIcon)
	} else {
		connectionItem.SetTitle("Connection: Disconnected")
		connectionItem.SetIcon(disabledIcon)
	}
}

// monitorVersion handles the startup check and the periodic check.
func monitorVersion() {
	for {
		// This inner loop handles the version check with retries.
		for {
			handleVersionCheck(true)
			currentVersion := getMenuItemTitle(versionItem.String())
			log.Println("Current Version:", currentVersion)

			// If the version matches the expected format, break the inner loop.
			if versionRegex.MatchString(currentVersion) || strings.HasPrefix(currentVersion, "Prerelease: ") {
				log.Println("Version check successful.")
				break
			}

			// If the version is in a default/error state, retry after a short delay.
			log.Println("Version is in default/error state, retrying in 5 seconds...")
			time.Sleep(5 * time.Second)
		}

		// Once a valid version is fetched, wait for the long polling interval.
		log.Println("Switching to 8-hour polling interval.")
		time.Sleep(8 * time.Hour)
	}
}

// handleVersionCheck communicates with the backend, updates the menu, and conditionally starts an update.
func handleVersionCheck(autoStart bool) {
	response, err := sendCommandAndReceive("get-version")
	if err != nil {
		log.Printf("Error fetching version: %v", err)
		versionItem.SetTitle("Unknown")
		updateItem.SetTitle("---")
		updateItem.Disable()
		return
	}

	response = strings.TrimPrefix(response, "VERSION_CHECK: ")
	isOutdated := strings.Contains(response, "Outdated")
	isPrerelease := strings.Contains(response, "Prerelease available")

	// --- Update Menu Items ---
	if isOutdated {
		// Defensive parsing: only split into two parts and validate
		isPrerelease = false // Ensure we don't treat an outdated response as a prerelease
		parts := strings.SplitN(response, ", ", 2)
		version := "Unknown"
		if len(parts) == 2 {
			version = parts[1]
		} else {
			log.Printf("Unexpected version response (Outdated): %q", response)
		}
		versionItem.SetTitle(version)
		updateItem.SetTitle("Update Available")
		updateItem.Enable()

		if autoStart {
			log.Println("Automatic update triggered by periodic check.")
			updateItem.Disable()
			go startUpdateStream()
		}

	} else if isPrerelease {
		// Parse prerelease version info
		// Expected format: "Prerelease available: 1.9.0-rc.1 (current: v1.8.0)"
		log.Printf("Prerelease response received: %q", response)
		if strings.HasPrefix(response, "Prerelease available:") {
			// Extract the prerelease version and current version
			prereleaseVersion := ""
			currentVersion := ""

			// Use regex to extract versions more reliably
			re := regexp.MustCompile(`Prerelease available: ([^\s]+) \(current: v([^\)]+)\)`)
			matches := re.FindStringSubmatch(response)
			log.Printf("Regex matches: %v (length: %d)", matches, len(matches))
			if len(matches) == 3 {
				prereleaseVersion = matches[1]
				currentVersion = matches[2]
				log.Printf("Extracted - Prerelease: %s, Current: %s", prereleaseVersion, currentVersion)

				versionItem.SetTitle(currentVersion)

				// Mark version item as "hidden" when prerelease is available
				versionItem.SetTooltip("Prerelease available")

				// Show prerelease notification with clean format
				prereleaseInfo := fmt.Sprintf("%s", prereleaseVersion)
				showPrereleaseNotification(prereleaseInfo)

				// Hide regular update button when prerelease is available
				updateItem.SetTitle("Up to date")
				updateItem.Disable()
			} else {
				// Fallback parsing if regex fails
				log.Printf("Failed to parse prerelease info from: %q", response)
				versionItem.SetTitle("v---")
				versionItem.SetTooltip("Prerelease available")
				updateItem.SetTitle("---")
				updateItem.Disable()
			}
		} else {
			versionItem.SetTitle(response)
			updateItem.SetTitle("---")
			updateItem.Disable()
		}
	} else if strings.Contains(response, "Up to date") {
		// Defensive parsing: only split into two parts and validate
		parts := strings.SplitN(response, ", ", 2)
		version := "Unknown"
		if len(parts) == 2 {
			version = parts[1]
		} else {
			log.Printf("Unexpected version response (Up to date): %q", response)
		}
		versionItem.SetTitle(version)
		versionItem.SetTooltip(fmt.Sprintf("Current version: %s", version))
		updateItem.SetTitle("Up to date")
		updateItem.Disable()

		// Hide prerelease button if current version is a prerelease and up to date
		if strings.Contains(version, "rc") {
			if prereleaseUpdateItem != nil {
				prereleaseUpdateItem.SetTitle("Up to date")
				prereleaseUpdateItem.Disable()
			}
			// Hide prerelease notification items if they exist
			if prereleaseItem != nil {
				prereleaseItem.Hide()
			}
			if prereleaseUpdateItem != nil {
				prereleaseUpdateItem.Hide()
			}
			prereleaseNotificationActive = false
		}
	} else {
		versionItem.SetTitle("v---")
		versionItem.SetTooltip("Prerelease available")
		updateItem.SetTitle("---")
		updateItem.Disable()
	}
}

// handleMenuActions listens for menu item clicks and performs actions
func handleMenuActions() {
	for range updateItem.ClickedCh {
		log.Println("Update clicked. Starting stream...")
		updateItem.SetTitle("Starting Update...")
		updateItem.Disable()
		go startUpdateStream()
	}
}

// startUpdateStream initiates the update on the server and streams the progress back. (FIXED LOGIC - UNCHANGED)
func startUpdateStream() {
	// Use mutex to prevent multiple update streams from starting simultaneously
	if !updateMutex.TryLock() {
		log.Println("Update already in progress. Skipping.")
		return
	}
	defer updateMutex.Unlock()

	conn, err := net.DialTimeout("tcp", backendAddress, 5*time.Second)
	if err != nil {
		log.Printf("Failed to connect to backend for update stream: %v", err)
		updateItem.SetTitle("Update Failed (Connect)")
		updateItem.Enable()
		return
	}
	defer conn.Close()

	fmt.Fprintln(conn, "update")

	reader := bufio.NewReader(conn)

	// Read and process the streaming response from the server
	for {
		response, err := reader.ReadString('\n')
		if err != nil {
			if err != io.EOF {
				log.Printf("Update stream error: %v", err)
				updateItem.SetTitle("Update Failed (Stream)")
			} else {
				log.Println("Update stream finished (EOF).")
				// Set a temporary status before the synchronous check
				updateItem.SetTitle("Checking Version Status...")
			}
			break
		}

		trimmed := strings.TrimSpace(response)
		log.Printf("Update Stream: %s", trimmed)

		// Display status on the menu item
		if strings.HasPrefix(trimmed, "UPDATE_PROGRESS:") {
			status := strings.TrimPrefix(trimmed, "UPDATE_PROGRESS: ")
			if status == "Complete" || status == "Error" {
				break
			}
			updateItem.SetTitle(fmt.Sprintf("Updating: %s", status))
		}
	}

	// After the stream ends, perform a synchronous version check to update the menu accurately
	handleVersionCheck(false)
}

// sendCommandAndReceive is a general utility for single request/response commands. (UNCHANGED)
func sendCommandAndReceive(command string) (string, error) {
	conn, err := net.DialTimeout("tcp", backendAddress, 3*time.Second)
	if err != nil {
		return "", fmt.Errorf("failed to dial: %w", err)
	}
	defer conn.Close()

	_ = conn.SetReadDeadline(time.Now().Add(5 * time.Second))

	_, err = fmt.Fprintln(conn, command)
	if err != nil {
		return "", fmt.Errorf("failed to send command: %w", err)
	}

	reader := bufio.NewReader(conn)
	response, err := reader.ReadString('\n')
	if err != nil {
		return "", fmt.Errorf("failed to read response: %w", err)
	}

	trimmed := strings.TrimSpace(response)
	if strings.HasPrefix(trimmed, "ERROR:") {
		return "", fmt.Errorf("server error: %s", trimmed)
	}

	return trimmed, nil
}

// getEmbeddedFile reads a file from the embedded file system (UNCHANGED)
func getEmbeddedFile(path string) ([]byte, error) {
	return embeddedFiles.ReadFile(path)
}

// getIconPath returns iconpath based on the OS (UNCHANGED)
func getIconPath() string {
	switch os := runtime.GOOS; os {
	case "windows":
		_ = os
		return "assets/wazuh-logo.ico"
	default:
		return "assets/wazuh-logo.png"
	}
}

// showPrereleaseNotification displays a systray notification for prerelease versions
func showPrereleaseNotification(prereleaseInfo string) {
	prereleaseMutex.Lock()
	defer prereleaseMutex.Unlock()

	// Only show the notification once per session
	if prereleaseShown {
		return
	}

	prereleaseShown = true
	prereleaseNotificationActive = true
	log.Printf("Showing prerelease notification: %s", prereleaseInfo)

	// Add prerelease notification menu items
	systray.AddSeparator()
	log.Printf("Creating prerelease menu items with info: %s", prereleaseInfo)
	prereleaseItem = systray.AddMenuItem("Prerelease Available: "+prereleaseInfo, "Prerelease version available for testing")
	prereleaseItem.Disable()
	prereleaseUpdateItem = systray.AddMenuItem("Update to Prerelease", "Install prerelease version")
	log.Printf("Prerelease menu items created successfully")

	// Handle prerelease menu actions
	go handlePrereleaseActions()
}

// handlePrereleaseActions handles clicks on prerelease notification menu items
func handlePrereleaseActions() {
	for range prereleaseUpdateItem.ClickedCh {
		log.Println("Prerelease update clicked. Starting update...")
		prereleaseUpdateItem.SetTitle("Starting Prerelease Update...")
		prereleaseUpdateItem.Disable()
		go startPrereleaseUpdateStream()
		return
	}
}

// startPrereleaseUpdateStream starts the update process for prerelease versions
func startPrereleaseUpdateStream() {
	// Use a specialized update stream for prerelease versions
	if !updateMutex.TryLock() {
		log.Println("Update already in progress. Skipping.")
		prereleaseUpdateItem.SetTitle("Update to Prerelease")
		prereleaseUpdateItem.Enable()
		return
	}
	defer updateMutex.Unlock()

	conn, err := net.DialTimeout("tcp", backendAddress, 5*time.Second)
	if err != nil {
		log.Printf("Failed to connect to backend for prerelease update stream: %v", err)
		prereleaseUpdateItem.SetTitle("Prerelease Update Failed (Connect)")
		prereleaseUpdateItem.Enable()
		return
	}
	defer conn.Close()

	fmt.Fprintln(conn, "update-prerelease")

	reader := bufio.NewReader(conn)

	// Read and process the streaming response from the server
	for {
		response, err := reader.ReadString('\n')
		if err != nil {
			if err != io.EOF {
				log.Printf("Prerelease update stream error: %v", err)
				prereleaseUpdateItem.SetTitle("Prerelease Update Failed (Stream)")
				prereleaseUpdateItem.Enable()
			} else {
				log.Println("Prerelease update stream finished (EOF).")
				// Set a temporary status before the synchronous check
				prereleaseUpdateItem.SetTitle("Checking Version Status...")
			}
			break
		}

		trimmed := strings.TrimSpace(response)
		log.Printf("Prerelease Update Stream: %s", trimmed)

		// Display status on the menu item
		if strings.HasPrefix(trimmed, "UPDATE_PROGRESS:") {
			status := strings.TrimPrefix(trimmed, "UPDATE_PROGRESS: ")
			if status == "Complete" || status == "Error" || strings.HasPrefix(status, "ERROR:") {
				if status == "Error" || strings.HasPrefix(status, "ERROR:") {
					prereleaseUpdateItem.SetTitle("Prerelease Update Failed")
					prereleaseUpdateItem.Enable()
				}
				break
			}
			prereleaseUpdateItem.SetTitle(fmt.Sprintf("Updating: %s", status))
		}
	}

	// After the stream ends, perform a synchronous version check to update the menu accurately
	handleVersionCheck(false)
}

// getMenuItemTitle extracts the Title from MenuItem.String()
// formatted like "MenuItem[N, "Title"]".
func getMenuItemTitle(input string) string {
	// Find the text between the quotes
	matches := titleRegexp.FindStringSubmatch(input)
	if len(matches) > 1 {
		// matches[1] is the content between the quotes
		return matches[1]
	}
	return ""
}

// onExit is called when the application is terminated (UNCHANGED)
func onExit() {
	log.Println("Frontend application stopped")
}
