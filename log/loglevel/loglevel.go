package loglevel

//LogLevel integer values
type LogLevel int

//Debug and all supported log levels
const (
	Trace LogLevel = iota
	Debug
	Info
	Warn
	Error
	None
	Unknown
)

// Enumify converts a log level passed as string to its corresponding enum value
func Enumify(str string) (level LogLevel) {
	switch str {
	case "":
		level = None
	case "trace":
		level = Trace
	case "debug":
		level = Debug
	case "info":
		level = Info
	case "warn":
		level = Warn
	case "error":
		level = Error
	default:
		level = Unknown
	}
	return
}
