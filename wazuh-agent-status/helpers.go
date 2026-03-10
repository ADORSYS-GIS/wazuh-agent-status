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

func getSystemLogFilePath() (string, error) {
	var logDir string

	switch runtime.GOOS {
	case "linux", "darwin":
		logDir = "/var/log"
	case "windows":
		logDir = "C:\\ProgramData\\wazuh\\logs"
	default:
		return "", fmt.Errorf("unsupported OS: %s", runtime.GOOS)
	}

	if runtime.GOOS == "windows" {
		if err := os.MkdirAll(logDir, 0755); err != nil {
			return "", fmt.Errorf("failed to create log directory: %v", err)
		}
	}

	return filepath.Join(logDir, "wazuh-agent-status.log"), nil
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
		versionPath, err := getVersionFilePath()
		if err != nil {
			log.Printf("Failed to get version file path on Windows: %v", err)
			return "Unknown"
		}
		output, err := os.ReadFile(versionPath)
		if err != nil {
			log.Printf("Failed to read local version on Windows: %v", err)
			return "Unknown"
		}
		return strings.TrimSpace(string(output))
	} else {
		versionPath, err := getVersionFilePath()
		if err != nil {
			log.Printf("Failed to get version file path on Linux/macOS: %v", err)
			return "Unknown"
		}
		output, err := runAsRoot("cat", versionPath)
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
	mergedMgPath, err := getMergedMgPath()
	if err != nil {
		log.Printf("Failed to get merged.mg path: %v", err)
		return []string{}
	}
	var output string
	var readErr error

	if runtime.GOOS == "windows" {
		data, readErr := os.ReadFile(mergedMgPath)
		if readErr != nil {
			log.Printf("Failed to read merged.mg on Windows: %v", readErr)
			return []string{}
		}
		output = string(data)
	} else {
		output, readErr = runAsRoot(grepCommand, sourceFileMarker, mergedMgPath)
		if readErr != nil {
			log.Printf("Failed to grep merged.mg on Linux/macOS: %v", readErr)
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

func getBasePath() (string, error) {
	switch runtime.GOOS {
	case "linux":
		return "/var/ossec", nil
	case "darwin":
		return "/Library/Ossec", nil
	case "windows":
		return "C:\\Program Files (x86)\\ossec-agent", nil
	default:
		return "", fmt.Errorf("unsupported OS: %s", runtime.GOOS)
	}
}

// getMergedMgPath returns merged.mg path based on the OS
func getMergedMgPath() (string, error) {
	basePath, err := getBasePath()
	if err != nil {
		return "", err
	}
	switch os := runtime.GOOS; os {
	case "windows":
		return filepath.Join(basePath, "shared", "merged.mg"), nil
	default:
		return filepath.Join(basePath, "etc", "shared", "merged.mg"), nil
	}
}

// getVersionFilePath returns version.txt path based on the OS
func getVersionFilePath() (string, error) {
	basePath, err := getBasePath()
	if err != nil {
		return "", err
	}
	switch runtime.GOOS {
	case "windows":
		return filepath.Join(basePath, "version.txt"), nil
	default:
		return filepath.Join(basePath, "etc", "version.txt"), nil
	}
}

// getAdorsysUpdatePath returns adorsys-update script path based on the OS
func getAdorsysUpdatePath() (string, error) {
	basePath, err := getBasePath()
	if err != nil {
		return "", err
	}
	switch runtime.GOOS {
	case "windows":
		return filepath.Join(basePath, "active-response", "bin", "adorsys-update.bat"), nil
	default:
		return filepath.Join(basePath, "active-response", "bin", "adorsys-update.sh"), nil
	}
}

// getVersionFilePath returns state file path based on the OS
func getWazuhStatePath() (string, error) {
	basePath, err := getBasePath()
	if err != nil {
		return "", err
	}
	switch runtime.GOOS {
	case "windows":
		return filepath.Join(basePath, "wazuh-agent.state"), nil
	default:
		return filepath.Join(basePath, "var", "run", "wazuh-agentd.state"), nil
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
