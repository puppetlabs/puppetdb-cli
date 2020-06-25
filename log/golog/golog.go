package golog

import (
	"fmt"
	"io"
	gologger "log"

	"github.com/puppetlabs/puppetdb-cli/log/loginterface"
	"github.com/puppetlabs/puppetdb-cli/log/loglevel"
)

type golog struct {
	logLevel loglevel.LogLevel
}

//NewGolog constructs a golog instance
func NewGolog() loginterface.Log {
	return &golog{
		logLevel: loglevel.Info,
	}
}

func (l *golog) Trace(msg string) {
	if l.logLevel <= loglevel.Trace {
		gologger.Println(msg)
	}
}

func (l *golog) Debug(msg string) {
	if l.logLevel <= loglevel.Debug {
		gologger.Println(msg)
	}
}

func (l *golog) Info(msg string) {
	if l.logLevel <= loglevel.Info {
		gologger.Println(msg)
	}
}

func (l *golog) Warn(msg string) {
	if l.logLevel <= loglevel.Warn {
		gologger.Println(msg)
	}
}

func (l *golog) Error(msg string) {
	if l.logLevel <= loglevel.Error {
		gologger.Println(msg)
	}
}

func (l *golog) SetOutput(w io.Writer) {
	gologger.SetOutput(w)
}

func (l *golog) SetLogLevel(level string) error {
	l.logLevel = loglevel.Enumify(level)
	if l.logLevel == loglevel.Unknown {
		return fmt.Errorf("Invalid log level provided: %s", level)
	}
	return nil
}

func (l *golog) GetLogLevel() loglevel.LogLevel {
	return l.logLevel
}
