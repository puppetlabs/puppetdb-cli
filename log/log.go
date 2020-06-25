package log

import (
	"fmt"
	"io"

	"github.com/puppetlabs/puppetdb-cli/log/golog"
	"github.com/puppetlabs/puppetdb-cli/log/loginterface"
	"github.com/puppetlabs/puppetdb-cli/log/loglevel"
)

var instance = golog.NewGolog()

//SetInstance sets logger instance
func SetInstance(log loginterface.Log) {
	instance = log
}

//Trace logs a trace message
func Trace(msg string) {
	instance.Trace(fmt.Sprintf("TRACE - %s", msg))
}

//Debug logs a debug message
func Debug(msg string) {
	instance.Debug(fmt.Sprintf("DEBUG - %s", msg))
}

//Info logs an info message
func Info(msg string) {
	instance.Info(fmt.Sprintf("INFO - %s", msg))
}

//Warn logs a warning message
func Warn(msg string) {
	instance.Warn(fmt.Sprintf("WARN - %s", msg))
}

//Error logs an error message
func Error(msg string) {
	instance.Error(fmt.Sprintf("ERROR - %s", msg))
}

//SetLogLevel sets the log level
func SetLogLevel(logLevel string) error {
	return instance.SetLogLevel(logLevel)
}

//GetLogLevel gets the log level
func GetLogLevel() loglevel.LogLevel {
	return instance.GetLogLevel()
}

//setOutput sets log output
func setOutput(w io.Writer) {
	instance.SetOutput(w)
}
