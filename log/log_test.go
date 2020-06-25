package log

import (
	"bytes"
	"fmt"
	"os"
	"strings"
	"testing"

	"github.com/stretchr/testify/assert"
)

var buf bytes.Buffer

func TestMain(m *testing.M) {
	setOutput(&buf)
	os.Exit(m.Run())
}

func TestWarnMessage(t *testing.T) {
	buf.Reset()

	expectedMsg := "WARN - this is a warning message"

	Warn(expectedMsg)
	assert.True(t, strings.Contains(buf.String(), expectedMsg))
}

func TestLowerLogLevelAreNotShown(t *testing.T) {
	buf.Reset()

	msg := "this should not be shown"

	SetLogLevel("error")
	Warn(msg)
	Info(msg)
	Debug(msg)
	Trace(msg)
	assert.Equal(t, buf.String(), "")
}

func TestHigherLogLevelsAreShown(t *testing.T) {
	buf.Reset()

	msg := "test message"

	SetLogLevel("info")
	Error(msg)
	Warn(msg)
	Info(msg)
	Debug(msg)
	Trace(msg)

	logCount := len(strings.Split(strings.TrimSuffix(buf.String(), "\n"), "\n"))

	// Should show only info, warn and error
	assert.Equal(t, 3, logCount)
}

func TestInvalidLogLevelErrors(t *testing.T) {
	buf.Reset()

	logLevel := "salam"
	expectedError := fmt.Errorf("Invalid log level provided: %s", logLevel)

	err := SetLogLevel(logLevel)
	assert.EqualError(t, expectedError, err.Error())
}
