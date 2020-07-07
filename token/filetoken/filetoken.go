package filetoken

import (
	"fmt"
	"io/ioutil"
	"os/user"
	"path/filepath"
	"regexp"
	"strings"

	"github.com/puppetlabs/puppetdb-cli/log"
	"github.com/puppetlabs/puppetdb-cli/token"
)

// fileToken struct
type fileToken struct {
	path string
}

// NewFileToken constructs a new filetoken
func NewFileToken(path string) token.Token {
	fileToken := fileToken{path: path}
	return &fileToken
}

// Read reads the token from a file and returns a valid token.
// If the token is not valid an error will be return
func (ft *fileToken) Read() (string, error) {
	var err error

	tokenPath, err := getPath(&ft.path)
	if err != nil {
		return "", err
	}

	data, err := ioutil.ReadFile(*tokenPath)
	if err != nil {
		return "", err
	}

	token := strings.TrimRight(string(data), "\r\n")

	valid := isValid(token)
	if !valid {
		err = fmt.Errorf("Token %s is invalid", token)
		return "", err
	}

	return token, nil
}

func getPath(path *string) (*string, error) {
	var err error
	tokenPath := path
	if *path == "" {
		tokenPath, err = defaultPath()
		if err != nil {
			return nil, err
		}
	}
	return tokenPath, nil
}

func isValid(content string) bool {

	jwtExpresion := `([A-Za-z0-9_-]{4,})\.([A-Za-z0-9_-]{4,})\.([A-Za-z0-9_-]{4,})`
	tokenExpr := `([A-Za-z0-9_-]+)`

	jwtMatched, _ := regexp.MatchString(jwtExpresion, content)
	if jwtMatched {
		log.Debug("Token is in JWT format")
		return true
	}

	exprMatch, _ := regexp.MatchString(tokenExpr, content)
	if exprMatch {
		log.Debug("Token format is valid")
		return true
	}

	return false
}

//defaultPath returns token default path
func defaultPath() (*string, error) {
	usr, err := user.Current()
	if err != nil {
		return nil, err
	}

	result := filepath.Join(usr.HomeDir, ".puppetlabs", "token")
	return &result, nil
}
