package api

import "github.com/puppetlabs/puppetdb-cli/api/client"

//Client is interface to the api client
type Client interface {
	GetClient() (*client.PuppetdbCli, error)
}
