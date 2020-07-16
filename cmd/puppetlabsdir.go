// +build darwin linux

package cmd

// PuppetLabsDir return puppetlabs dir
func PuppetLabsDir() (string, error) {
	return "/etc/puppetlabs", nil
}
