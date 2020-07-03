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

var exportCmd = &cobra.Command{
	Use:   "export",
	Short: "export <path> [--anon=<profile>]",
	Args: func(cmd *cobra.Command, args []string) error {
		if len(args) != 1 {
			return errors.New("Must specify a single file to write to")
		}
		return nil
	},
	Run: executeExportCommand,
}

func init() {
	rootCmd.AddCommand(exportCmd)
}

func executeExportCommand(cmd *cobra.Command, args []string) {
	anonymizationProfile := viper.GetString("anon")
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

	_, err := puppetDb.GetExportFile(filePath, anonymizationProfile)

	if e, ok := err.(*api.ArgError); ok {
		log.Error(e.Error())
		os.Exit(1)
	}

	if err != nil {
		log.Error(fmt.Sprintf("Failed to export puppetdb data: %s", err.Error()))
	} else {
		log.Info(fmt.Sprintf("Wrote archive to \"%s\"", filePath))
	}

}
