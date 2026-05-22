//go:build darwin
// +build darwin

package main

func getBasePath() (string, error) {
	return "/Library/Ossec", nil
}
