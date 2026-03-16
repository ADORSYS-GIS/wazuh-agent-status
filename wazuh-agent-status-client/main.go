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

// String constants to avoid duplication
const (
	// backend address
	backendAddress = "localhost:50505"

	// Update state
	updateAvailableTitle     = "Update Available"
	upToDateStatus           = "Up to date"
	prereleaseAvailableTitle = "Prerelease available"

	// Default states
	defaultVersionTitle = "v---"
	defaultUpdateTitle  = "---"
	unknownString       = "Unknown"

	// Common strings
	errorPrefix                          = "ERROR:"
	updateAvailableTitleWithStableSuffix = updateAvailableTitle + " (Stable)"
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
	statusItem = systray.AddMenuItem("Agent: %s"+unknownString, "Wazuh Agent Status")
	statusItem.Disable()
	connectionItem = systray.AddMenuItem("Connection: %s"+unknownString, "Wazuh Agent Connection")
	connectionItem.Disable()
	systray.AddSeparator()
	updateItem = systray.AddMenuItem(defaultUpdateTitle, "Update the Wazuh Agent")
	updateItem.Disable() // Initially disabled
	systray.AddSeparator()
	versionItem = systray.AddMenuItem(defaultVersionTitle, "The version state of the wazuhbsetup")
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
	} else if strings.HasPrefix(response, errorPrefix) {
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
			updateStatusItems(unknownString, unknownString)
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
			if versionRegex.MatchString(currentVersion) || strings.HasPrefix(currentVersion, "Prerelease: ") || (strings.HasPrefix(currentVersion, "v") && !strings.HasPrefix(currentVersion, "v---")) {
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
		versionItem.SetTitle(unknownString)
		updateItem.SetTitle(defaultUpdateTitle)
		updateItem.Disable()
		return
	}

	response = strings.TrimPrefix(response, "VERSION_CHECK: ")
	isOutdated := strings.Contains(response, "Outdated")
	isPrerelease := strings.Contains(response, prereleaseAvailableTitle)
	isCombined := strings.Contains(response, "Outdated with "+prereleaseAvailableTitle)

	// --- Update Menu Items ---
	if isCombined {
		handleCombinedStatus(response, autoStart)
	} else if isOutdated {
		handleOutdatedVersion(response, autoStart)
	} else if isPrerelease {
		handlePrereleaseVersion(response)
	} else if strings.Contains(response, upToDateStatus) {
		handleUpToDateVersion(response)
	} else {
		versionItem.SetTitle(defaultVersionTitle)
		versionItem.SetTooltip(prereleaseAvailableTitle)
		updateItem.SetTitle(defaultUpdateTitle)
		updateItem.Disable()
	}
}

// handleCombinedStatus processes responses where both stable update and prerelease are available
func handleCombinedStatus(response string, autoStart bool) {
	re := regexp.MustCompile(`Outdated with Prerelease available: (.+?) \(stable: ([^,]+), prerelease: ([^\)]+)\)`)
	matches := re.FindStringSubmatch(response)

	if len(matches) == 4 {
		currentVersion := strings.TrimSpace(matches[1])
		stableVersion := matches[2]
		prereleaseVersion := matches[3]

		log.Printf("Combined status - Current: %s, Stable: %s, Prerelease: %s", currentVersion, stableVersion, prereleaseVersion)

		if !strings.HasPrefix(currentVersion, "v") && !strings.HasPrefix(currentVersion, "Prerelease:") {
			currentVersion = "v" + currentVersion
		}

		// Set version display to current version
		versionItem.SetTitle(currentVersion)
		versionItem.SetTooltip(fmt.Sprintf("%s: %s (stable), %s (prerelease)", updateAvailableTitle, stableVersion, prereleaseVersion))

		// Enable regular update button for stable version
		updateItem.SetTitle(updateAvailableTitleWithStableSuffix)
		updateItem.Enable()

		// Show prerelease notification
		prereleaseInfo := fmt.Sprintf("%s", prereleaseVersion)
		showPrereleaseNotification(prereleaseInfo)

		if autoStart {
			log.Println("Automatic update triggered by periodic check (stable version).")
			updateItem.Disable()
			go startUpdateStream()
		}
	} else {
		// Fallback parsing if regex fails
		log.Printf("Failed to parse combined status from: %q", response)
		versionItem.SetTitle(defaultVersionTitle)
		versionItem.SetTooltip("Update and " + prereleaseAvailableTitle)
		updateItem.SetTitle(updateAvailableTitle)
		updateItem.Enable()
	}
}

// handleOutdatedVersion processes outdated version response
func handleOutdatedVersion(response string, autoStart bool) {
	parts := strings.SplitN(response, ", ", 2)
	version := unknownString
	if len(parts) == 2 {
		version = parts[1]
	} else {
		log.Printf("Unexpected version response (Outdated): %q", response)
	}
	versionItem.SetTitle(version)
	updateItem.SetTitle(updateAvailableTitle)
	updateItem.Enable()

	if autoStart {
		log.Println("Automatic update triggered by periodic check.")
		updateItem.Disable()
		go startUpdateStream()
	}
}

// handlePrereleaseVersion processes prerelease version response
func handlePrereleaseVersion(response string) {
	log.Printf("Prerelease response received: %q", response)
	if !strings.HasPrefix(response, prereleaseAvailableTitle+":") {
		versionItem.SetTitle(response)
		updateItem.SetTitle(defaultUpdateTitle)
		updateItem.Disable()
		return
	}

	re := regexp.MustCompile(`Prerelease available: ([^\s]+) \(current: (v[^\)]+|Prerelease: v[^\)]+)\)`)
	matches := re.FindStringSubmatch(response)
	log.Printf("Regex matches: %v (length: %d)", matches, len(matches))

	if len(matches) == 3 {
		prereleaseVersion := matches[1]
		currentVersion := matches[2]
		log.Printf("Extracted - Prerelease: %s, Current: %s", prereleaseVersion, currentVersion)

		versionItem.SetTitle(currentVersion)
		versionItem.SetTooltip(prereleaseAvailableTitle)

		// Show prerelease notification with clean format
		prereleaseInfo := fmt.Sprintf("%s", prereleaseVersion)
		showPrereleaseNotification(prereleaseInfo)

		// Only hide regular update button if this is not a combined status
		// (combined status is handled by handleCombinedStatus)
		if !strings.Contains(response, "Outdated with") {
			updateItem.SetTitle(upToDateStatus)
			updateItem.Disable()
		}
	} else {
		// Fallback parsing if regex fails
		log.Printf("Failed to parse prerelease info from: %q", response)
		versionItem.SetTitle(defaultVersionTitle)
		versionItem.SetTooltip(prereleaseAvailableTitle)
		updateItem.SetTitle(defaultUpdateTitle)
		updateItem.Disable()
	}
}

// handleUpToDateVersion processes up-to-date version response
func handleUpToDateVersion(response string) {
	parts := strings.SplitN(response, ", ", 2)
	version := unknownString
	if len(parts) == 2 {
		version = parts[1]
	} else {
		log.Printf("Unexpected version response (%s): %q", upToDateStatus, response)
	}
	versionItem.SetTitle(version)
	versionItem.SetTooltip("Current version: %s" + version)
	updateItem.SetTitle(upToDateStatus)
	updateItem.Disable()

	// Hide prerelease button if current version is a prerelease and up to date
	if strings.Contains(version, "rc") {
		if prereleaseUpdateItem != nil {
			prereleaseUpdateItem.SetTitle(upToDateStatus)
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
			updateItem.SetTitle("Updating: " + status)
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
	if strings.HasPrefix(trimmed, errorPrefix) {
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
	prereleaseItem = systray.AddMenuItem(prereleaseAvailableTitle+": "+prereleaseInfo, "Prerelease version available for testing")
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
		updateItem.Hide()
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

	updateItem.Hide()
	prereleaseUpdateItem.SetTitle("Starting Prerelease Update...")
	prereleaseUpdateItem.Disable()

	conn, err := net.DialTimeout("tcp", backendAddress, 5*time.Second)
	if err != nil {
		log.Printf("Failed to connect to backend for prerelease update stream: %v", err)
		prereleaseUpdateItem.SetTitle("Prerelease Update Failed (Connect)")
		prereleaseUpdateItem.Enable()
		// Restore regular update button state
		updateItem.SetTitle(updateAvailableTitleWithStableSuffix)
		updateItem.Enable()
		return
	}
	defer conn.Close()

	fmt.Fprintln(conn, "update-prerelease")

	reader := bufio.NewReader(conn)
	readUpdateStream(reader, prereleaseUpdateItem)

	// After the stream ends, perform a synchronous version check to update the menu accurately
	handleVersionCheck(false)
}

// processUpdateStreamResponse processes a single line from the update stream
func processUpdateStreamResponse(trimmed string, prereleaseUpdateItem *systray.MenuItem) bool {
	log.Printf("Prerelease Update Stream: %s", trimmed)

	// Display status on the menu item
	if !strings.HasPrefix(trimmed, "UPDATE_PROGRESS:") {
		return true
	}

	status := strings.TrimPrefix(trimmed, "UPDATE_PROGRESS: ")
	if status == "Complete" || status == "Error" || strings.HasPrefix(status, errorPrefix) {
		if status == "Error" || strings.HasPrefix(status, errorPrefix) {
			prereleaseUpdateItem.SetTitle("Prerelease Update Failed")
			prereleaseUpdateItem.Enable()
			// Restore regular update button state
			updateItem.SetTitle(updateAvailableTitleWithStableSuffix)
			updateItem.Enable()
		}
		return false
	}
	prereleaseUpdateItem.SetTitle("Updating: " + status)
	return true
}

// readUpdateStream reads and processes the update stream
func readUpdateStream(reader *bufio.Reader, prereleaseUpdateItem *systray.MenuItem) {
	for {
		response, err := reader.ReadString('\n')
		if err != nil {
			if err != io.EOF {
				log.Printf("Prerelease update stream error: %v", err)
				prereleaseUpdateItem.SetTitle("Prerelease Update Failed (Stream)")
				prereleaseUpdateItem.Enable()
				// Restore regular update button state
				updateItem.SetTitle(updateAvailableTitleWithStableSuffix)
				updateItem.Enable()
			} else {
				log.Println("Prerelease update stream finished (EOF).")
				prereleaseUpdateItem.SetTitle("Checking Version Status...")
			}
			break
		}

		trimmed := strings.TrimSpace(response)
		if !processUpdateStreamResponse(trimmed, prereleaseUpdateItem) {
			break
		}
	}
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
