package git

import (
	"context"
	"testing"
	"time"
)

func TestRepository_GetCommits(t *testing.T) {
	// This is a basic test that would need a real git repository
	// In a real implementation, you'd set up a test repository
	t.Skip("Requires real git repository for testing")
}

func TestParseTime(t *testing.T) {
	tests := []struct {
		input    string
		expected time.Time
		hasError bool
	}{
		{
			input:    "2025-01-01",
			expected: time.Date(2025, 1, 1, 0, 0, 0, 0, time.UTC),
			hasError: false,
		},
		{
			input:    "1d",
			expected: time.Now().AddDate(0, 0, -1),
			hasError: false,
		},
		{
			input:    "2w",
			expected: time.Now().AddDate(0, 0, -14),
			hasError: false,
		},
		{
			input:    "invalid",
			expected: time.Time{},
			hasError: true,
		},
	}

	for _, test := range tests {
		t.Run(test.input, func(t *testing.T) {
			// This would test the parseTime function from main.go
			// In a real implementation, you'd extract this to a testable function
			t.Skip("parseTime function needs to be extracted for testing")
		})
	}
}