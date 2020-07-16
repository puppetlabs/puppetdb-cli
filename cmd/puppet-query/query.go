package main

import (
	"errors"
	"fmt"
	"os"

	"github.com/puppetlabs/puppetdb-cli/app"
	"github.com/puppetlabs/puppetdb-cli/cmd"
	"github.com/puppetlabs/puppetdb-cli/json"
	"github.com/puppetlabs/puppetdb-cli/log"
	"github.com/spf13/cobra"
	"github.com/spf13/viper"
)

func init() {
	cmd.RootCmd.RunE = executeQueryCommand
}

func executeQueryCommand(cmd *cobra.Command, args []string) error {
	url := viper.GetStringSlice("urls")[0]

	if len(viper.GetStringSlice("urls")) > 1 {
		log.Debug(fmt.Sprintf("Multiple URLs passed, will only use the first one (%s)", url))
	}

	log.Debug(fmt.Sprintf("args(%v) len is %v", args, len(args)))
	if len(args) != 1 {
		err := errors.New("One query argument must be provided")
		return err
	}

	puppetQuery := app.NewWithConfig(
		url,
		viper.GetString("cacert"),
		viper.GetString("cert"),
		viper.GetString("key"),
		viper.GetString("token"))

	resp, err := puppetQuery.QueryWithErrorDetails(args[0])
	if err != nil {
		log.Error(err.Error())
		os.Exit(1)
	}

	json.WritePayload(os.Stdout, resp.GetPayload())
	return nil
}
