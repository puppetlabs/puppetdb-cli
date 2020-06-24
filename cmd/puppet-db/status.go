package main

import (
	"fmt"

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
	rootCmd.AddCommand(statusCmd)
}

func executeStatusCommand(cmd *cobra.Command, args []string) {
	fmt.Print("not implemented")
}
