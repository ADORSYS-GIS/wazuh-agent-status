package main

import (
	"encoding/json"
	"fmt"
	"io"
	"log"
	"net/http"
	"os"
	"os/exec"
	"path/filepath"
	"runtime"
	"strings"
)

func getSystemLogFilePath() string {
	var logDir string

	switch runtime.GOOS {
	case "linux", "darwin":
		logDir = "/var/log"
	case "windows":
		logDir = "C:\\ProgramData\\wazuh\\logs"
	default:
		logDir = "./logs"
	}

	if runtime.GOOS == "windows" {
		if err := os.MkdirAll(logDir, 0755); err != nil {
			log.Fatalf("failed to create log directory: %v", err)
		}
	}

	return filepath.Join(logDir, "wazuh-agent-status.log")
}

// Run a command as root using sudo
func runAsRoot(command string, args ...string) (string, error) {
	cmd := exec.Command(sudoCommand, append([]string{command}, args...)...)
	output, err := cmd.CombinedOutput()
	return string(output), err
}

// Read local version from embedded file
func getLocalVersion() string {
	if runtime.GOOS == "windows" {
		output, err := os.ReadFile(getVersionFilePath())
		if err != nil {
			log.Printf("Failed to read local version on Windows: %v", err)
			return "Unknown"
		}
		return strings.TrimSpace(string(output))
	} else {
		output, err := runAsRoot("cat", getVersionFilePath())
		if err != nil {
			log.Printf("Failed to read local version on Linux/macOS: %v", err)
			return "Unknown"
		}
		return strings.TrimSpace(output)
	}
}

func isVersionHigher(online, local string) bool {
	// Strip "v" prefix
	online = strings.TrimPrefix(online, "v")
	local = strings.TrimPrefix(local, "v")

	// Split by "-" to separate version from prerelease (e.g., "1.9.0-rc.1" -> ["1.9.0", "rc.1"])
	onlineBase := strings.Split(online, "-")[0]
	localBase := strings.Split(local, "-")[0]

	onlineParts := strings.Split(onlineBase, ".")
	localParts := strings.Split(localBase, ".")

	// Compare base version numbers
	for i := 0; i < len(onlineParts) && i < len(localParts); i++ {
		var onlineNum, localNum int
		fmt.Sscanf(onlineParts[i], "%d", &onlineNum)
		fmt.Sscanf(localParts[i], "%d", &localNum)

		if onlineNum > localNum {
			return true
		}
		if onlineNum < localNum {
			return false
		}
	}

	// If base versions are equal, check length
	if len(onlineParts) != len(localParts) {
		return len(onlineParts) > len(localParts)
	}

	// If base versions are identical, stable release (no "-") is higher than prerelease (has "-")
	// e.g., "1.9.0" > "1.9.0-rc.1"
	onlineIsPrerelease := strings.Contains(online, "-")
	localIsPrerelease := strings.Contains(local, "-")

	if onlineIsPrerelease && !localIsPrerelease {
		return false // online is prerelease, local is stable -> online is NOT higher
	}
	if !onlineIsPrerelease && localIsPrerelease {
		return true // online is stable, local is prerelease -> online IS higher
	}

	// Both are prerelease or both are stable with same base version
	return false
}

func fetchVersionInfo() *VersionInfo {
	resp, err := http.Get(versionURL)
	if err != nil {
		log.Printf("Failed to fetch version info: %v", err)
		return nil
	}
	defer resp.Body.Close()

	if resp.StatusCode != http.StatusOK {
		log.Printf("Failed to fetch version info: HTTP %d", resp.StatusCode)
		return nil
	}

	var versionInfo VersionInfo
	err = json.NewDecoder(resp.Body).Decode(&versionInfo)
	if err != nil {
		log.Printf("Failed to parse version info: %v", err)
		return nil
	}
	log.Printf("Fetched version info: Framework.Version=%s, Framework.PrereleaseVersion=%s, TestGroups=%v",
		versionInfo.Framework.Version, versionInfo.Framework.PrereleaseVersion, versionInfo.PrereleaseTestGroups)

	return &versionInfo
}

// getAgentGroups extracts groups from merged.mg
func getAgentGroups() []string {
	mergedMgPath := getMergedMgPath()
	var output string
	var err error

	if runtime.GOOS == "windows" {
		data, readErr := os.ReadFile(mergedMgPath)
		if readErr != nil {
			log.Printf("Failed to read merged.mg on Windows: %v", readErr)
			return []string{}
		}
		output = string(data)
	} else {
		output, err = runAsRoot(grepCommand, sourceFileMarker, mergedMgPath)
		if err != nil {
			log.Printf("Failed to grep merged.mg on Linux/macOS: %v", err)
			// On error, try reading the file directly as a fallback
			data, readErr := os.ReadFile(mergedMgPath)
			if readErr != nil {
				return []string{}
			}
			output = string(data)
		}
	}

	return extractGroupsFromMergedMg(output)
}

// extractGroupsFromMergedMg parses the merged.mg content to extract groups
func extractGroupsFromMergedMg(content string) []string {
	var groups []string
	lines := strings.Split(content, "\n")
	for _, line := range lines {
		if strings.Contains(line, sourceFileMarker) {
			// Extract group from "<!-- Source file: <group>/agent.conf -->"
			parts := strings.Split(line, sourceFileMarker)
			if len(parts) > 1 {
				groupPart := strings.TrimSpace(parts[1])
				// Extract until "/agent.conf"
				if strings.Contains(groupPart, "/agent.conf") {
					group := strings.Split(groupPart, "/agent.conf")[0]
					groups = append(groups, strings.TrimSpace(group))
				}
			}
		}
	}
	return groups
}

// shouldShowPrerelease checks if prerelease should be shown based on agent groups
func shouldShowPrerelease(versionInfo *VersionInfo, agentGroups []string) bool {
	if len(versionInfo.PrereleaseTestGroups) == 0 || len(agentGroups) == 0 {
		return false
	}

	// Check if any agent group matches the test groups
	for _, agentGroup := range agentGroups {
		for _, testGroup := range versionInfo.PrereleaseTestGroups {
			if strings.EqualFold(agentGroup, testGroup) {
				return true
			}
		}
	}

	return false
}

func getBasePath() string {
	switch os := runtime.GOOS; os {
	case "linux":
		return "/var/ossec"
	case "darwin":
		return "/Library/Ossec"
	case "windows":
		return "C:\\Program Files (x86)\\ossec-agent"
	default:
		return unsupportedOSMessage
	}
}

// getMergedMgPath returns merged.mg path based on the OS
func getMergedMgPath() string {
	basePath := getBasePath()
	if basePath == unsupportedOSMessage {
		return unsupportedOSMessage
	}
	switch os := runtime.GOOS; os {
	case "windows":
		return filepath.Join(basePath, "shared", "merged.mg")
	default:
		return filepath.Join(basePath, "etc", "shared", "merged.mg")
	}
}

// getVersionFilePath returns version.txt path based on the OS
func getVersionFilePath() string {
	basePath := getBasePath()
	if basePath == unsupportedOSMessage {
		return unsupportedOSMessage
	}
	switch runtime.GOOS {
	case "windows":
		return filepath.Join(basePath, "version.txt")
	default:
		return filepath.Join(basePath, "etc", "version.txt")
	}
}

// createLogFile creates a secure log file for update operations
func createLogFile() (*os.File, error) {
	logFileHandle, err := os.CreateTemp("", "wazuh-update-*.log")
	if err != nil {
		log.Printf("ERROR: Failed to create log file: %v", err)
		return nil, err
	}
	logFile := logFileHandle.Name()
	log.Printf("Logging to: %s", logFile)
	return logFileHandle, nil
}

// downloadFile downloads a file from URL to the specified path
func downloadAndSaveFile(url string, filePath string, mod os.FileMode) error {
	resp, err := http.Get(url)
	if err != nil {
		return err
	}
	defer resp.Body.Close()

	if resp.StatusCode != http.StatusOK {
		return fmt.Errorf("HTTP %d: %s", resp.StatusCode, resp.Status)
	}

	out, err := os.Create(filePath)
	if err != nil {
		return err
	}
	defer out.Close()

	// Only apply chmod on non-Windows systems
	if runtime.GOOS != "windows" {
		if err := os.Chmod(filePath, mod); err != nil {
			return err
		}
	}

	_, err = io.Copy(out, resp.Body)
	return err
}

// handlePrereleaseUpdate handles the prerelease update process
func handlePrereleaseUpdate(writeUpdate func(string), logFileHandle *os.File) {
	versionInfo := fetchVersionInfo()
	if versionInfo == nil || versionInfo.Framework.PrereleaseVersion == "" {
		writeUpdate(fmt.Sprintf("ERROR: Empty prerelease"))
		logFileHandle.WriteString(fmt.Sprintf("ERROR: Empty prerelease"))
		return
	}

	var prereleaseScriptURL string
	var tempFilePattern string
	switch runtime.GOOS {
	case "windows":
		prereleaseScriptURL = fmt.Sprintf("https://raw.githubusercontent.com/ADORSYS-GIS/wazuh-agent/refs/tags/v%s/scripts/setup-agent.ps1", versionInfo.Framework.PrereleaseVersion)
		tempFilePattern = "wazuh-prerelease-*.ps1"
	default:
		prereleaseScriptURL = fmt.Sprintf("https://raw.githubusercontent.com/ADORSYS-GIS/wazuh-agent/refs/tags/v%s/scripts/setup-agent.sh", versionInfo.Framework.PrereleaseVersion)
		tempFilePattern = "wazuh-prerelease-*.sh"
	}

	log.Printf("Prerelease script URL: %s", prereleaseScriptURL)

	tempFile, err := os.CreateTemp("", tempFilePattern)
	if err != nil {
		logFileHandle.WriteString(fmt.Sprintf("ERROR: Failed to create temp log file: %v\n", err))
		return
	}
	tempFile.Close() // We just need the name, will write to it later

	if err := downloadAndSaveFile(prereleaseScriptURL, tempFile.Name(), 0750); err != nil {
		logFileHandle.WriteString(fmt.Sprintf("ERROR: Failed to download prerelease script: %v\n", err))
		return
	}
	defer os.Remove(tempFile.Name()) // Clean up temp file

	var cmd *exec.Cmd
	switch runtime.GOOS {
	case "windows":
		// On Windows, use PowerShell to execute the PowerShell script
		cmd = exec.Command(powershellExe, executionPolicyFlag, "Bypass", "-File", tempFile.Name(), "-Prerelease")
	default:
		// On Unix-like systems, execute the shell script directly
		cmd = exec.Command(tempFile.Name())
	}

	executePrereleaseScript(cmd, writeUpdate, logFileHandle)
}

// executePrereleaseScript executes the prerelease script and waits for completion
func executePrereleaseScript(cmd *exec.Cmd, writeUpdate func(string), logFileHandle *os.File) {
	// Stream stdout and stderr ONLY to the update log file
	cmd.Stdout = logFileHandle
	cmd.Stderr = logFileHandle

	// Execute the prerelease script
	if err := cmd.Start(); err != nil {
		writeUpdate(fmt.Sprintf("ERROR: Command failed to start: %v", err))
		logFileHandle.WriteString(fmt.Sprintf("ERROR: Command failed to start: %v\n", err))
		return
	}

	writeUpdate("Executing script...")
	logFileHandle.WriteString("Executing script...\n")

	// Wait for the command to finish
	if err := cmd.Wait(); err != nil {
		writeUpdate("Error")
		logFileHandle.WriteString(fmt.Sprintf("UPDATE FAILED: %v\n", err))
		log.Println(fmt.Sprintf("ERROR: Update failed: %v", err))
	} else {
		writeUpdate("Complete")
		logFileHandle.WriteString("UPDATE COMPLETED SUCCESSFULLY\n")
		log.Println("Wazuh agent updated successfully")
	}
}
