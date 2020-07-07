package main

import (
	"path/filepath"
	"testing"

	"github.com/puppetlabs/puppetdb-cli/testdata"
	"github.com/spf13/viper"
	"github.com/stretchr/testify/assert"
)

func TestGlobalConfigFileAbsent(t *testing.T) {
	assert := assert.New(t)
	err := readGlobalConfigFile()
	assert.NoError(err)
}

func TestDefaultConfigFileAbsent(t *testing.T) {
	assert := assert.New(t)
	err := readConfigFile(getDefaultConfig())
	assert.NoError(err)
}

func TestCLIConfigFileAbsent(t *testing.T) {
	assert := assert.New(t)
	err := readConfigFile("/path/to/absent/config")
	assert.Error(err)
}

func TestCanReadAndAliasConfigParameters(t *testing.T) {
	assert := assert.New(t)
	initConfig(filepath.Join(testdata.FixturePath(), "puppetdb.conf"))
	registerConfigAliases()

	assert.Equal([]string{"https://127.0.0.1:8080", "https://127.0.0.1:8081"}, viper.GetStringSlice("urls"))
	assert.Equal("/path/to/cacert", viper.GetString("cacert"))
	assert.Equal("/path/to/cert", viper.GetString("cert"))
	assert.Equal("/path/to/private_key", viper.GetString("key"))
	assert.Equal("/path/to/token", viper.GetString("token"))
}
