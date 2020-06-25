package loginterface

import (
	"io"

	"github.com/puppetlabs/puppetdb-cli/log/loglevel"
)

//Log interface to logging service
type Log interface {
	Trace(msg string)
	Debug(msg string)
	Info(msg string)
	Warn(msg string)
	Error(msg string)
	SetOutput(w io.Writer)
	SetLogLevel(logLevel string) error
	GetLogLevel() loglevel.LogLevel
}
