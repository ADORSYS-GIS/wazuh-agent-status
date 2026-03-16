package main

import (
	"bufio"
	"encoding/json"
	"fmt"
	"io"
	"log"
	"net/http"
	"os"
	"runtime"
	"strings"
	"time"
)

func isVersionHigher(online, local string) bool {
	// Strip "v" prefix
	online = strings.TrimPrefix(online, "v")
	local = strings.TrimPrefix(local, "v")

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

	onlineIsPrerelease := strings.Contains(online, "-")
	localIsPrerelease := strings.Contains(local, "-")

	if onlineIsPrerelease && !localIsPrerelease {
		return false
	}
	if !onlineIsPrerelease && localIsPrerelease {
		return true
	}

	// Both are prerelease or both are stable with same base version
	return false
}

func fetchVersionInfo() *VersionInfo {
	client := &http.Client{
		Timeout: 10 * time.Second,
	}

	resp, err := client.Get(versionURL)
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
	log.Printf("Fetched version info: Framework.Version=%s, Framework.PrereleaseVersion=%s",
		versionInfo.Framework.Version, versionInfo.Framework.PrereleaseVersion)

	return &versionInfo
}

// extractFirstCommentGroup extracts the first commented line as a group
func extractFirstCommentGroup(line string) (string, bool) {
	if !strings.HasPrefix(line, "#") || strings.Contains(line, sourceFileMarker) {
		return "", false
	}

	candidate := strings.TrimSpace(strings.TrimPrefix(line, "#"))
	return candidate, candidate != ""
}

// extractSourceFileGroup extracts group from source file marker
func extractSourceFileGroup(line string) (string, bool) {
	if !strings.Contains(line, sourceFileMarker) {
		return "", false
	}

	parts := strings.SplitN(line, sourceFileMarker, 2)
	if len(parts) != 2 {
		return "", false
	}

	pathPart := strings.TrimSpace(parts[1])
	pathPart, found := strings.CutSuffix(pathPart, "-->")
	if !found {
		return "", false
	}

	pathPart = strings.TrimSpace(pathPart)

	if idx := strings.Index(pathPart, "/"); idx > 0 {
		group := pathPart[:idx]
		return group, group != ""
	}

	return "", false
}

// getAgentGroups extracts groups from merged.mg file
func getAgentGroups() ([]string, error) {
	mergedMgPath, err := getMergedMgPath()
	if err != nil {
		log.Printf("Failed to get merged.mg path: %v", err)
		return []string{}, err
	}

	f, err := os.Open(mergedMgPath)
	if err != nil {
		return nil, fmt.Errorf("open merged.mg: %w", err)
	}
	defer f.Close()

	var groups []string
	scanner := bufio.NewScanner(f)
	firstCommentAdded := false

	for scanner.Scan() {
		line := strings.TrimSpace(scanner.Text())
		if line == "" {
			continue
		}

		if !firstCommentAdded {
			if group, ok := extractFirstCommentGroup(line); ok {
				groups = append(groups, group)
				firstCommentAdded = true
				continue
			}
		}

		if group, ok := extractSourceFileGroup(line); ok {
			groups = append(groups, group)
		}
	}

	log.Printf("Extracted groups: %s", groups)
	return groups, scanner.Err()
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

	if runtime.GOOS != "windows" {
		if err := os.Chmod(filePath, mod); err != nil {
			return err
		}
	}

	_, err = io.Copy(out, resp.Body)
	return err
}
