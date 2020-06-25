package main

import (
	"os"
	"path/filepath"
	"testing"

	"github.com/spf13/viper"
	"github.com/stretchr/testify/assert"
)

func fixturePath() string {
	path, err := os.Getwd()
	if err != nil {
		panic(err)
	}
	return filepath.Join(filepath.Dir(filepath.Dir(path)), "testdata")
}

func TestCanReadAndAliasConfigParameters(t *testing.T) {
	assert := assert.New(t)
	initConfig(filepath.Join(fixturePath(), "puppetdb.conf"))
	registerConfigAliases()

	assert.Equal([]string{"https://127.0.0.1:8080", "https://127.0.0.1:8081"}, viper.GetStringSlice("urls"))
	assert.Equal("/path/to/cacert", viper.GetString("cacert"))
	assert.Equal("/path/to/cert", viper.GetString("cert"))
	assert.Equal("/path/to/private_key", viper.GetString("key"))
	assert.Equal("/path/to/token", viper.GetString("token"))
}
