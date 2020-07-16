package main

import (
	"errors"
	"fmt"
	"os"

	"github.com/puppetlabs/puppetdb-cli/api"
	"github.com/puppetlabs/puppetdb-cli/app"
	"github.com/puppetlabs/puppetdb-cli/cmd"
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
	RunE: executeExportCommand,
}

func init() {
	cmd.RootCmd.AddCommand(exportCmd)
}

func executeExportCommand(cmd *cobra.Command, args []string) error {
	anonymizationProfile, _ := cmd.Flags().GetString("anon")

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

	if _, ok := err.(*api.ArgError); ok {
		return err
	}

	if err != nil {
		log.Error(fmt.Sprintf("Failed to export puppetdb data: %s", err.Error()))
		os.Exit(1)
	}
	fmt.Println("Wrote archive to \"", filePath, "\"")
	return nil
}
