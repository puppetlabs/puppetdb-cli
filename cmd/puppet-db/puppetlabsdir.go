// +build darwin linux

package main

// PuppetLabsDir return puppetlabs dir
func PuppetLabsDir() (string, error) {
	return "/etc/puppetlabs", nil
}
