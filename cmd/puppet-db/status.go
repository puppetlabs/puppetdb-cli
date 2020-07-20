package main

import (
	"os"

	"github.com/puppetlabs/puppetdb-cli/api"
	"github.com/puppetlabs/puppetdb-cli/app"
	"github.com/puppetlabs/puppetdb-cli/cmd"
	"github.com/puppetlabs/puppetdb-cli/json"
	"github.com/puppetlabs/puppetdb-cli/log"
	"github.com/spf13/cobra"
	"github.com/spf13/viper"
)

var statusViper = viper.New()

var statusCmd = &cobra.Command{
	Use:   "status",
	Short: "status",
	Long:  "status",
	Run:   executeStatusCommand,
}

func init() {
	cmd.RootCmd.AddCommand(statusCmd)
}

func executeStatusCommand(cmd *cobra.Command, args []string) {
	result := make(map[string]interface{})

	for _, url := range viper.GetStringSlice("urls") {
		puppetDb := app.NewWithConfig(
			url,
			viper.GetString("cacert"),
			viper.GetString("cert"),
			viper.GetString("key"),
			viper.GetString("token"))

		resp, err := puppetDb.GetStatus()

		if _, ok := err.(*api.ArgError); ok {
			log.Error(err.Error())
			os.Exit(1)
		}

		if err != nil {
			resp := make(map[string]string)
			resp["error"] = err.Error()
			result[url] = resp
		} else {
			result[url] = resp.GetPayload()
		}
	}
	//FIXME if all are errors, we should return 1
	json.WritePayload(os.Stdout, result)
}
