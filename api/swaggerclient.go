package api

import (
	"fmt"
	"net/http"
	"net/url"
	"strings"

	openapihttptransport "github.com/go-openapi/runtime/client"
	"github.com/go-openapi/strfmt"
	"github.com/puppetlabs/puppetdb-cli/api/client"
	"github.com/puppetlabs/puppetdb-cli/log"
	"github.com/puppetlabs/puppetdb-cli/log/loglevel"
)

//SwaggerClient represents a puppetdb-cli swagger client
type SwaggerClient struct {
	cacert, cert, key, url, tokenFile string
}

//NewClient creates a new SwaggerClient
func NewClient(cacert, cert, key, url, tokenFile string) Client {
	sc := SwaggerClient{
		cacert:    cacert,
		cert:      cert,
		key:       key,
		url:       url,
		tokenFile: tokenFile,
	}
	return &sc
}

//ArgError represents an argument error
type ArgError struct {
	msg string
}

func (e *ArgError) Error() string {
	return e.msg
}

func supportedScheme(urlScheme string) bool {
	switch urlScheme {
	case "http", "https":
		return true
	default:
		return false
	}
}

func (sc *SwaggerClient) validateSchemeParameters(urlScheme string) error {
	if urlScheme == "https" && (sc.tokenFile == "" && (sc.cert == "" || sc.key == "")) {
		return &ArgError{"ssl requires a token, please use `puppet access login` to retrieve a token (alternatively use 'cert' and 'key' for whitelist validation)"}
	}
	return nil
}

//GetClient configures and creates a swagger generated client
func (sc *SwaggerClient) GetClient() (*client.PuppetdbCli, error) {
	url, err := url.Parse(sc.url)
	if err != nil {
		return nil, err
	}
	if !supportedScheme(url.Scheme) {
		err = fmt.Errorf("Invalid scheme for %v", strings.Title(url.Scheme))
		return nil, err
	}

	if err := sc.validateSchemeParameters(url.Scheme); err != nil {
		return nil, err
	}

	httpclient, err := getHTTPClient(sc.cacert, sc.cert, sc.key)
	if err != nil {
		return nil, err
	}

	openapitransport := newOpenAPITransport(*url, httpclient)
	openapitransport.SetDebug(log.GetLogLevel() == loglevel.Debug)

	return client.New(openapitransport, strfmt.Default), nil
}

func getHTTPClient(cacert, cert, key string) (*http.Client, error) {
	tlsClientOptions := openapihttptransport.TLSClientOptions{
		CA:          cacert,
		Certificate: cert,
		Key:         key,
	}
	return openapihttptransport.TLSClient(tlsClientOptions)
}

func newOpenAPITransport(url url.URL, httpclient *http.Client) *openapihttptransport.Runtime {
	schemes := []string{url.Scheme}

	return openapihttptransport.NewWithClient(
		fmt.Sprintf("%s:%s", url.Hostname(), url.Port()),
		fmt.Sprintf("%s", url.Path),
		schemes, httpclient)
}
