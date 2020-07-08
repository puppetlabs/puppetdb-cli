package main

import (
	"bufio"
	"fmt"
	"os"
	"os/user"
	"path/filepath"
	"regexp"
	"strings"

	"github.com/puppetlabs/puppetdb-cli/log"
	"github.com/spf13/cobra"
	"github.com/spf13/viper"
)

var (
	urls    []string
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
	registerConfigAliases()
	bindConfigFlags(rootCmd)
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
		"`Path` to CLI config",
	)
	cmd.PersistentFlags().StringSliceVarP(
		&urls,
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
		"",
		"`Path` to client certificate for auth",
	)
	cmd.PersistentFlags().StringP(
		"key",
		"",
		"",
		"`Path` to client certificate for auth",
	)

	cmd.PersistentFlags().StringP(
		"token",
		"",
		getDefaultToken(),
		"`Path` to RBAC token for auth (PE Only)",
	)
}

func initConfig(cfgFile string) {
	viper.SetConfigType("json")
	err := readConfigFile(cfgFile)
	if err != nil {
		log.Error(err.Error())
		os.Exit(1)
	}
}

func readConfigFile(cfgFile string) error {
	err := readGlobalConfigFile()
	if err != nil {
		return err
	}

	return mergeUserConfigFile(cfgFile)
}

func getGlobalConfigFile() (string, error) {
	puppetLabsDir, err := PuppetLabsDir()
	if err != nil {
		return puppetLabsDir, err
	}

	globalConfigFile := filepath.Join(puppetLabsDir, "client-tools", "puppetdb.conf")
	return globalConfigFile, nil
}

func readGlobalConfigFile() error {
	globalConfigFile, err := getGlobalConfigFile()
	if err != nil {
		return err
	}

	_, err = os.Stat(globalConfigFile)
	if err != nil {
		log.Debug(fmt.Sprintf("Failed reading global config file: %s", err.Error()))
		return nil
	}
	viper.SetConfigFile(globalConfigFile)
	return viper.ReadInConfig()
}

func getDefaultConfig() string {
	usr, err := user.Current()
	if err != nil {
		log.Error(err.Error())
		return ""
	}

	configFile := filepath.Join(usr.HomeDir, ".puppetlabs", "client-tools", "puppetdb.conf")
	return configFile
}

func mergeUserConfigFile(cfgFile string) error {
	_, err := os.Stat(cfgFile)
	if err != nil {
		if cfgFile == getDefaultConfig() {
			log.Debug(fmt.Sprintf("Failed reading default config file: %s", err.Error()))
			return nil
		}
		log.Error(fmt.Sprintf("Failed reading CLI config file: %s", err.Error()))
		return err
	}
	viper.SetConfigFile(cfgFile)
	return viper.MergeInConfig()
}

func validateGlobalFlags(cmd *cobra.Command) error {
	logLevel, err := cmd.Flags().GetString("log-level")
	if err != nil {
		return err
	}
	if err := log.SetLogLevel(strings.ToLower(logLevel)); err != nil {
		return err
	}
	log.Debug(fmt.Sprintf("Log level changed to: %s", logLevel))

	tokenFile, err := cmd.Flags().GetString("token")
	if err != nil {
		return err
	}

	if tokenFile == getDefaultToken() {
		if _, err = os.Stat(tokenFile); err != nil {
			cmd.Flags().Set("token", "")
		}
	}

	cfgFile, err := cmd.Flags().GetString("config")
	if err != nil {
		return err
	}
	initConfig(cfgFile)
	return nil
}

func registerConfigAliases() {
	viper.RegisterAlias("urls", "puppetdb.server_urls")
	viper.RegisterAlias("cacert", "puppetdb.cacert")
	viper.RegisterAlias("cert", "puppetdb.cert")
	viper.RegisterAlias("key", "puppetdb.key")
	viper.RegisterAlias("token", "puppetdb.token-file")
}

func bindConfigFlags(cmd *cobra.Command) {
	viper.BindPFlag("puppetdb.server_urls", cmd.PersistentFlags().Lookup("urls"))
	viper.BindPFlag("puppetdb.cacert", cmd.PersistentFlags().Lookup("cacert"))
	viper.BindPFlag("puppetdb.cert", cmd.PersistentFlags().Lookup("cert"))
	viper.BindPFlag("puppetdb.key", cmd.PersistentFlags().Lookup("key"))
	viper.BindPFlag("puppetdb.token-file", cmd.PersistentFlags().Lookup("token"))
}

func getDefaultUrls() []string {
	return []string{"http://127.0.0.1:8080"}
}

func getDefaultCacert() string {
	puppetLabsDir, err := PuppetLabsDir()
	if err != nil {
		log.Error(err.Error())
		return ""
	}

	return filepath.Join(puppetLabsDir, "puppet", "ssl", "certs", "ca.pem")
}

func getDefaultToken() string {
	usr, err := user.Current()
	if err != nil {
		return ""
	}

	return filepath.Join(usr.HomeDir, ".puppetlabs", "token")
}
