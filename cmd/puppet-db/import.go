package main

import (
	"fmt"

	"github.com/spf13/cobra"
)

var importCmd = &cobra.Command{
	Use:   "import",
	Short: "import <path>",
	Run:   executeImportCommand,
}

func init() {
	rootCmd.AddCommand(importCmd)
}

func executeImportCommand(cmd *cobra.Command, args []string) {
	fmt.Print("not implemented")
}
