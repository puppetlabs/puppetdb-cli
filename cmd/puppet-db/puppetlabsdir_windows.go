package main

import (
	"path/filepath"

	"golang.org/x/sys/windows"
)

// PuppetLabsDir return puppetlabs dir
func PuppetLabsDir() (string, error) {
	dir, err := windows.KnownFolderPath(windows.FOLDERID_ProgramData, windows.KF_FLAG_DEFAULT)
	if err != nil {
		return dir, err
	}

	return filepath.Join(dir, "PuppetLabs"), nil
}
