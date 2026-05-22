//go:build linux
// +build linux

package main

func getBasePath() (string, error) {
	return "/var/ossec", nil
}
