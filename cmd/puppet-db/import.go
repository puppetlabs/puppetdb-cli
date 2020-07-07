package main

import (
	"errors"
	"fmt"
	"os"

	"github.com/puppetlabs/puppetdb-cli/api"
	app "github.com/puppetlabs/puppetdb-cli/app/puppet-db"
	"github.com/puppetlabs/puppetdb-cli/log"
	"github.com/spf13/cobra"
	"github.com/spf13/viper"
)

var importCmd = &cobra.Command{
	Use:   "import",
	Short: "import <path>",
	Args: func(cmd *cobra.Command, args []string) error {
		if len(args) != 1 {
			return errors.New("Must specify a single file to import")
		}
		return nil
	},
	Run: executeImportCommand,
}

func init() {
	rootCmd.AddCommand(importCmd)
}

func executeImportCommand(cmd *cobra.Command, args []string) {
	url := viper.GetStringSlice("urls")[0]
	filePath := args[0]

	if len(viper.GetStringSlice("urls")) > 1 {
		log.Debug(fmt.Sprintf("Multiple URLs passed, will only use the first one (%s)", url))
	}

	puppetDb := app.NewWithConfig(
		url,
		viper.GetString("cacert"),
		viper.GetString("cert"),
		viper.GetString("key"),
		viper.GetString("token"))

	resp, err := puppetDb.PostImportFile(filePath)

	if e, ok := err.(*api.ArgError); ok {
		log.Error(e.Error())
		os.Exit(1)
	}

	if err != nil {
		log.Error(fmt.Sprintf("Failed to import puppetdb data: %s", err.Error()))
		os.Exit(1)
	} else {
		if !resp.Payload.Ok {
			log.Warn(fmt.Sprintf("API returned 200, but got 'ok: %t' instead of true", resp.Payload.Ok))
		}
		log.Info(fmt.Sprintf("Successfully imported \"%s\"", filePath))
	}
}
