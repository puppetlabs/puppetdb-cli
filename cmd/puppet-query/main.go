package main

import (
	"os"

	"github.com/puppetlabs/puppetdb-cli/cmd"
	"github.com/puppetlabs/puppetdb-cli/log"
)

// Version of the application, to be overwritten from build command line
var Version = "0.0.0"

func init() {
	cmd.RootCmd.Use = "puppet-query [flags] <query>"
	cmd.RootCmd.Short = "puppet-query."
}

func main() {
	os.Setenv("GODEBUG", "x509ignoreCN=0")

	if err := cmd.Execute(Version); err != nil {
		log.Error(err.Error())
		os.Exit(1)
	}
}
