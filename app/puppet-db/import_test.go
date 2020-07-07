package app

import (
	"context"
	"errors"
	"testing"

	"github.com/golang/mock/gomock"
	"github.com/spf13/afero"

	"github.com/puppetlabs/puppetdb-cli/api/client"
	"github.com/puppetlabs/puppetdb-cli/api/client/operations"
	mock_operations "github.com/puppetlabs/puppetdb-cli/api/client/operations/testing"
	"github.com/puppetlabs/puppetdb-cli/api/models"
	mock_api "github.com/puppetlabs/puppetdb-cli/api/testing"
	match "github.com/puppetlabs/puppetdb-cli/app/puppet-db/testing"
	mock_token "github.com/puppetlabs/puppetdb-cli/token/testing"

	"github.com/stretchr/testify/assert"
)

func TestRunImportFailsIfNoClient(t *testing.T) {
	assert := assert.New(t)
	errorMessage := "No client"

	ctrl := gomock.NewController(t)
	defer ctrl.Finish()

	token := mock_token.NewMockToken(ctrl)
	api := mock_api.NewMockClient(ctrl)

	token.EXPECT().Read().Return("my token", nil)
	api.EXPECT().GetClient().Return(nil, errors.New(errorMessage))

	puppetDb := New()
	puppetDb.Token = token
	puppetDb.Client = api

	_, receivedError := puppetDb.PostImportFile("import.tar.gz")
	assert.EqualError(receivedError, errorMessage)
}

func TestRunImportFailsIfFileIsAbsent(t *testing.T) {
	appFS = afero.NewMemMapFs()
	assert := assert.New(t)
	errorMessage := "open import.tar.gz: file does not exist"

	ctrl := gomock.NewController(t)
	defer ctrl.Finish()

	token := mock_token.NewMockToken(ctrl)
	api := mock_api.NewMockClient(ctrl)
	operationsMock := mock_operations.NewMockClientService(ctrl)
	client := &client.PuppetdbCli{
		Operations: operationsMock,
	}

	api.EXPECT().GetClient().Return(client, nil)
	token.EXPECT().Read().Return("my token", nil)

	puppetDb := New()
	puppetDb.Token = token
	puppetDb.Client = api

	_, receivedError := puppetDb.PostImportFile("import.tar.gz")
	assert.EqualError(receivedError, errorMessage, "Importing an absent file should fail")
}

func TestRunImportSuccess(t *testing.T) {
	filePath = "import.tar.gz"
	appFS = afero.NewMemMapFs()
	appFS.Create(filePath)
	archive, _ := appFS.Open(filePath)

	assert := assert.New(t)

	ctrl := gomock.NewController(t)
	defer ctrl.Finish()

	api := mock_api.NewMockClient(ctrl)
	token := mock_token.NewMockToken(ctrl)
	operationsMock := mock_operations.NewMockClientService(ctrl)
	client := &client.PuppetdbCli{
		Operations: operationsMock,
	}
	api.EXPECT().GetClient().Return(client, nil)
	token.EXPECT().Read().Return("my token", nil)

	resp := operations.NewPostImportOK().Payload
	result := &operations.PostImportOK{
		Payload: resp,
	}

	postImportParameters := operations.NewPostImportParamsWithContext(context.Background())
	postImportParameters.SetArchive(archive)
	operationsMock.EXPECT().PostImport(postImportParameters, match.XAuthenticationWriter(t, "my token")).Return(result, nil)

	puppetDb := New()
	puppetDb.Token = token
	puppetDb.Client = api

	_, err := puppetDb.PostImportFile(filePath)
	assert.NoError(err)
}

func TestRunImportError(t *testing.T) {
	filePath = "import.tar.gz"
	appFS = afero.NewMemMapFs()
	appFS.Create(filePath)
	archive, _ := appFS.Open(filePath)

	assert := assert.New(t)

	ctrl := gomock.NewController(t)
	defer ctrl.Finish()

	api := mock_api.NewMockClient(ctrl)
	token := mock_token.NewMockToken(ctrl)
	operationsMock := mock_operations.NewMockClientService(ctrl)
	client := &client.PuppetdbCli{
		Operations: operationsMock,
	}
	api.EXPECT().GetClient().Return(client, nil)
	token.EXPECT().Read().Return("my token", nil)

	result := operations.NewPostImportDefault(404)
	result.Payload = &models.Error{
		Msg:     "error message",
		Details: "details",
	}

	postImportParameters := operations.NewPostImportParamsWithContext(context.Background())
	postImportParameters.SetArchive(archive)
	operationsMock.EXPECT().PostImport(postImportParameters, match.XAuthenticationWriter(t, "my token")).Return(nil, result)

	puppetDb := New()
	puppetDb.Token = token
	puppetDb.Client = api

	res, err := puppetDb.PostImportFile(filePath)
	assert.Nil(res)
	assert.EqualError(err, "[POST /pdb/admin/v1/archive][404] postImport default  &{Details:details Kind: Msg:error message}")
}
