package main

import (
	"os"

	"github.com/puppetlabs/puppetdb-cli/log"
)

// Version of the application, to be overwritten from build command line
var Version = "0.0.0"

func main() {
	if err := Execute(Version); err != nil {
		log.Error(err.Error())
		os.Exit(1)
	}
}
