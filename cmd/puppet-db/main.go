package main

import (
	"os"
)

// Version of the application, to be overwritten from build command line
var Version = "0.0.0"

func main() {
	if err := Execute(Version); err != nil {
		os.Exit(1)
	}
}
