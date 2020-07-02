package app

import (
	"github.com/puppetlabs/puppetdb-cli/api"
	"github.com/puppetlabs/puppetdb-cli/token"
	"github.com/puppetlabs/puppetdb-cli/token/filetoken"
)

// PuppetDb interface
type PuppetDb struct {
	URL     string
	Token   token.Token
	Cacert  string
	Cert    string
	Key     string
	Version string
	Client  api.Client
}

// NewWithConfig creates a puppet code application with configuration
func NewWithConfig(url, cacert, cert, key, tokenFile string) *PuppetDb {
	return &PuppetDb{
		URL:    url,
		Cacert: cacert,
		Cert:   cert,
		Key:    key,
		Token:  filetoken.NewFileToken(tokenFile),

		Client: api.NewClient(cacert, cert, key, url, tokenFile),
	}
}

// NewPuppetDbApp FIXME
func NewPuppetDbApp(version string) *PuppetDb {
	return &PuppetDb{
		Version: version,
	}
}

// New creates an unconfigured puppet-db application
func New() *PuppetDb {
	return &PuppetDb{
		Token:  filetoken.NewFileToken(""),
		Client: api.NewClient("", "", "", "", ""),
	}
}
