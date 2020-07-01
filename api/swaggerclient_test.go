package api

import (
	"path/filepath"
	"testing"

	"github.com/puppetlabs/puppetdb-cli/testdata"
	"github.com/stretchr/testify/assert"
)

func TestGetClientFailsIfNoUrl(t *testing.T) {
	assert := assert.New(t)
	errorMessage := "Invalid scheme for "

	cacert := ""
	cert := ""
	key := ""
	url := ""
	token := ""
	client := NewClient(cacert, cert, key, url, token)
	_, receivedError := client.GetClient()
	assert.EqualError(receivedError, errorMessage)
}

func TestGetClientSuccessIfHTTP(t *testing.T) {
	assert := assert.New(t)

	cacert := ""
	cert := ""
	key := ""
	url := "http://random3751.com"
	token := ""
	client := NewClient(cacert, cert, key, url, token)

	_, receivedError := client.GetClient()
	assert.NoError(receivedError)
}

func TestGetClientFailsIfHTTPSNoToken(t *testing.T) {
	assert := assert.New(t)
	errorMessage := "ssl requires a token, please use `puppet access login` to retrieve a token (alternatively use 'cert' and 'key' for whitelist validation)"

	cacert := ""
	cert := ""
	key := ""
	url := "https://random3751.com"
	token := ""
	client := NewClient(cacert, cert, key, url, token)

	_, receivedError := client.GetClient()
	assert.EqualError(receivedError, errorMessage)
}

func TestGetClientSuccessIfHTTPSWithToken(t *testing.T) {
	assert := assert.New(t)

	cacert := ""
	cert := ""
	key := ""
	url := "https://random3751.com"
	token := filepath.Join(testdata.FixturePath(), "token")
	client := NewClient(cacert, cert, key, url, token)

	_, receivedError := client.GetClient()
	assert.NoError(receivedError)
}

func TestGetClientSuccessIfHTTPSWithCertAndKey(t *testing.T) {
	assert := assert.New(t)

	cacert := ""
	cert := filepath.Join(testdata.FixturePath(), "cert.crt")
	key := filepath.Join(testdata.FixturePath(), "private_key.key")
	url := "https://random3751.com"
	token := ""
	client := NewClient(cacert, cert, key, url, token)

	_, receivedError := client.GetClient()
	assert.NoError(receivedError)
}

func TestGetClientFailsIfCannotParse(t *testing.T) {
	assert := assert.New(t)

	cacert := ""
	cert := ""
	key := ""
	url := "§¶£¡:random.com"
	token := ""
	client := NewClient(cacert, cert, key, url, token)
	_, receivedError := client.GetClient()
	assert.Error(receivedError)
}
