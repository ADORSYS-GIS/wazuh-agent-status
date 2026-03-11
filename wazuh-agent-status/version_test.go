package main

import "testing"

func TestIsVersionHigher(t *testing.T) {
	tests := []struct {
		name     string
		online   string
		local    string
		expected bool
	}{
		// Basic version comparisons
		{"newer major", "v2.0.0", "v1.9.0", true},
		{"older major", "v1.0.0", "v2.0.0", false},
		{"newer minor", "v1.9.0", "v1.8.0", true},
		{"older minor", "v1.8.0", "v1.9.0", false},
		{"newer patch", "v1.8.1", "v1.8.0", true},
		{"older patch", "v1.8.0", "v1.8.1", false},
		{"equal versions", "v1.8.0", "v1.8.0", false},

		// Release candidate comparisons
		{"stable vs rc - stable is higher", "v1.9.0", "v1.9.0-rc.1", true},
		{"rc vs stable - rc is lower", "v1.9.0-rc.1", "v1.9.0", false},
		{"rc vs rc same version", "v1.9.0-rc.1", "v1.9.0-rc.1", false},
		{"newer version rc vs older stable", "v1.9.0-rc.1", "v1.8.0", true},
		{"older version rc vs newer stable", "v1.8.0-rc.1", "v1.9.0", false},

		// Without 'v' prefix
		{"no prefix newer", "1.9.0", "1.8.0", true},
		{"no prefix rc vs stable", "1.9.0-rc.1", "1.9.0", false},
	}

	for _, tt := range tests {
		t.Run(tt.name, func(t *testing.T) {
			result := isVersionHigher(tt.online, tt.local)
			if result != tt.expected {
				t.Errorf("isVersionHigher(%q, %q) = %v, want %v",
					tt.online, tt.local, result, tt.expected)
			}
		})
	}
}
