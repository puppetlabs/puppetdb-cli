package testdata

import (
	"go/build"
	"os"
	"path/filepath"
)

func FixturePath() string {
	gopath := os.Getenv("GOPATH")
	if gopath == "" {
		gopath = build.Default.GOPATH
	}
	return filepath.Join(gopath, "src/github.com/puppetlabs/puppetdb-cli/testdata/fixtures")
}
