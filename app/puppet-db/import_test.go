package app

import (
	"context"
	"errors"
	"fmt"
	"os"
	"testing"

	"github.com/golang/mock/gomock"

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
	filePath := "import.tar.gz"
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

	_, receivedError := puppetDb.PostImportFile(filePath)
	assert.EqualError(receivedError, errorMessage)
}

func TestRunImportFailsIfFileIsAbsent(t *testing.T) {
	assert := assert.New(t)
	filePath := "import.tar.gz"
	errorMessage := fmt.Sprintf("open %s: file does not exist", filePath)

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

	fsmock := match.NewMockFs(ctrl)
	appFS = fsmock
	fsmock.EXPECT().Open(filePath).Return(nil, errors.New(errorMessage))

	puppetDb := New()
	puppetDb.Token = token
	puppetDb.Client = api

	_, receivedError := puppetDb.PostImportFile(filePath)
	assert.EqualError(receivedError, errorMessage, "Importing an absent file should fail")
}

func TestRunImportSuccess(t *testing.T) {
	assert := assert.New(t)
	filePath := "import.tar.gz"

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

	fsmock := match.NewMockFs(ctrl)
	appFS = fsmock
	archive := os.NewFile(0, filePath)
	fsmock.EXPECT().Open(filePath).Return(archive, nil)

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
	assert := assert.New(t)
	filePath := "import.tar.gz"

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

	fsmock := match.NewMockFs(ctrl)
	appFS = fsmock

	archive := os.NewFile(0, filePath)
	fsmock.EXPECT().Open(filePath).Return(archive, nil)

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
