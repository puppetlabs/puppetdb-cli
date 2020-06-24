package main

import (
	"fmt"

	"github.com/spf13/cobra"
)

var exportCmd = &cobra.Command{
	Use:   "export",
	Short: "export <path> [--anon=<profile>]",
	Run:   executeExportCommand,
}

func init() {
	rootCmd.AddCommand(exportCmd)
}

func executeExportCommand(cmd *cobra.Command, args []string) {
	fmt.Print("not implemented")
}
