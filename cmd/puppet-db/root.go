package main

import (
	"bufio"
	"errors"
	"regexp"
	"strings"

	"github.com/spf13/cobra"
)

var (
	rootCmd = &cobra.Command{
		Use:   "puppet-db [flags] [options]",
		Short: "puppet-db.",
		PersistentPreRunE: func(cmd *cobra.Command, args []string) error {
			return validateGlobalFlags(cmd)
		},
	}
)

func init() {

	rootCmd.Flags().SortFlags = false
	rootCmd.SetHelpCommand(&cobra.Command{
		Use:    "no-help",
		Hidden: true,
	})

	rootCmd.Flags().BoolP("help", "h", false, "Show this screen.")
	rootCmd.Flags().BoolP("version", "v", false, "Show version.")
	setCmdFlags(rootCmd)
	customizeUsage(rootCmd)
}

func customizeUsage(cmd *cobra.Command) {
	cobra.AddTemplateFunc("enhanceFlagUsages", enhanceFlagUsages)

	usageTemplate := cmd.UsageTemplate()
	usageTemplate = strings.ReplaceAll(usageTemplate, ".FlagUsages", ".FlagUsages | enhanceFlagUsages")
	rootCmd.SetUsageTemplate(usageTemplate)
}

func enhanceFlagUsages(s string) string {
	s, changed := updateFlagValues(s)
	if changed {
		s = updateNoFlagValues(s)
		s = lowerValue(s)
	}
	return s
}

func updateFlagValues(s string) (string, bool) {
	reValues := regexp.MustCompile(`(--[^ ]+) ([^ ]+)   (.*)`)
	updated := false
	updatedUsage := ""
	updatedUsageSeparator := ""
	scanner := bufio.NewScanner(strings.NewReader(s))
	for scanner.Scan() {
		a := scanner.Text()
		if reValues.FindStringIndex(a) != nil {
			updated = true
			a = reValues.ReplaceAllString(a, "$1=<$2>   $3")
		}
		updatedUsage = updatedUsage + updatedUsageSeparator + a
		updatedUsageSeparator = "\n"
	}
	return updatedUsage, updated
}

func updateNoFlagValues(s string) string {
	reNoValues := regexp.MustCompile(`(--[^ =]+   )(.*)`)
	updatedUsage := ""
	updatedUsageSeparator := ""
	scanner := bufio.NewScanner(strings.NewReader(s))
	for scanner.Scan() {
		a := scanner.Text()
		a = reNoValues.ReplaceAllString(a, "$1  $2")
		updatedUsage = updatedUsage + updatedUsageSeparator + a
		updatedUsageSeparator = "\n"
	}
	return updatedUsage
}

func lowerValue(s string) string {
	reToLower := regexp.MustCompile(`--[^=]+=<([^ ]+)>`)
	updatedUsage := ""
	updatedUsageSeparator := ""
	scanner := bufio.NewScanner(strings.NewReader(s))
	for scanner.Scan() {
		a := scanner.Text()
		a = reToLower.ReplaceAllStringFunc(a, strings.ToLower)
		updatedUsage = updatedUsage + updatedUsageSeparator + a
		updatedUsageSeparator = "\n"
	}
	return updatedUsage
}

// Execute will start command line parsing
func Execute(version string) error {
	rootCmd.Version = version
	return rootCmd.Execute()
}

func setCmdFlags(cmd *cobra.Command) {
	cmd.PersistentFlags().StringP(
		"log-level",
		"l",
		"warn",
		"Set logging `level`. Supported levels are: none, trace, debug, info, warn, and error",
	)
	cmd.PersistentFlags().StringP(
		"anon",
		"",
		"none",
		"Archive anonymization `profile`",
	)
	cmd.PersistentFlags().StringP(
		"config",
		"c",
		getDefaultConfig(),
		"`Path` to CLI config, defaults to $HOME/.puppetlabs/client-tools/puppetdb.conf",
	)
	cmd.PersistentFlags().StringP(
		"urls",
		"u",
		getDefaultUrls(),
		"`Urls` to PuppetDB instances",
	)
	cmd.PersistentFlags().StringP(
		"cacert",
		"",
		getDefaultCacert(),
		"`Path` to CA certificate for auth",
	)
	cmd.PersistentFlags().StringP(
		"cert",
		"",
		getDefaultCert(),
		"`Path` to client certificate for auth",
	)
	cmd.PersistentFlags().StringP(
		"key",
		"",
		getDefaultKey(),
		"`Path` to client certificate for auth",
	)

	cmd.PersistentFlags().StringP(
		"token",
		"",
		getDefaultToken(),
		"`Path` to RBAC token for auth (PE Only)",
	)
}

func validateGlobalFlags(cmd *cobra.Command) error {
	return nil
}

func getDefaultConfig() string {
	return errors.New("not implemented").Error()
}

func getDefaultUrls() string {
	return errors.New("not implemented").Error()
}

func getDefaultCacert() string {
	return errors.New("not implemented").Error()
}

func getDefaultCert() string {
	return errors.New("not implemented").Error()
}

func getDefaultKey() string {
	return errors.New("not implemented").Error()
}

func getDefaultToken() string {
	return errors.New("not implemented").Error()
}
