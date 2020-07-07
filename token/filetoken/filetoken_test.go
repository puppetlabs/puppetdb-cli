package filetoken

import (
	"os"
	"path/filepath"
	"testing"

	"github.com/stretchr/testify/assert"
)

func TestReadOK(t *testing.T) {
	//arrange
	test := assert.New(t)
	root, _ := os.Getwd()
	path := filepath.Join(root, "../../testdata/token")
	fileToken := NewFileToken(path)

	//act
	token, err := fileToken.Read()

	//assert
	test.Equal(nil, err)
	test.Equal("eyJhbGciOiJIUzI1NiIsInR5cCI6IkpXVCJ9.eyJzdWIiOiIxMjM0NTY3ODkwIiwibmFtZSI6IkpvaG4gRG9lIiwiaWF0IjoxNTE2MjM5MDIyfQ.SflKxwRJSMeKKF2QT4fwpMeJf36POk6yJV_adQssw5cmytoken", token)
}

func TestIsValidJWTOK(t *testing.T) {
	//arrange
	test := assert.New(t)
	token := "eyJhbGciOiJIUzI1NiIsInR5cCI6IkpXVCJ9.eyJsb2dnZWRJbkFzIjoiYWRtaW4iLCJpYXQiOjE0MjI3Nzk2Mzh9.gzSraSYS8EXBxLN_oWnFSRgCzcmJmMjLiuyu5CSpyHI"

	//act
	valid := isValid(token)

	//assert
	test.True(valid)
}

func TestIsValidToken(t *testing.T) {
	//arrange
	test := assert.New(t)
	token := "this.is.a.valid.token"

	//act
	valid := isValid(token)

	//assert
	test.True(valid)
}

func TestIsInvalid(t *testing.T) {
	//arrange
	test := assert.New(t)
	invalidToken := ":[]"

	//act
	valid := isValid(invalidToken)

	//assert
	test.False(valid)
}

func TestAllowEmptyPath(t *testing.T) {
	//arrange
	test := assert.New(t)
	path := ""

	//act
	tokenPath, err := getPath(&path)

	//asset
	test.Nil(err)
	test.NotEqual("", tokenPath)
}

func TestInvalidPath(t *testing.T) {
	//arrange
	test := assert.New(t)
	path := "invalidpath"
	fileToken := NewFileToken(path)

	//act
	_, err := fileToken.Read()

	//asset
	test.Error(err)
}

func TestDefaultPath(t *testing.T) {
	//arrange
	test := assert.New(t)

	//act
	_, err := defaultPath()

	//assert
	test.Nil(err)

}
