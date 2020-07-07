package app

import (
	"context"
	"errors"
	"fmt"
	"io"
	"path/filepath"
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

var filePath string = "/path/to/export.tar.gz"

func TestRunExportFailsIfNoClient(t *testing.T) {
	assert := assert.New(t)
	errorMessage := "No client"

	ctrl := gomock.NewController(t)
	defer ctrl.Finish()

	token := mock_token.NewMockToken(ctrl)
	api := mock_api.NewMockClient(ctrl)

	token.EXPECT().Read().Return("my token", nil)
	api.EXPECT().GetClient().Return(nil, errors.New(errorMessage))

	puppetCode := New()
	puppetCode.Token = token
	puppetCode.Client = api
	_, receivedError := puppetCode.GetExportFile(filePath, "none")
	assert.EqualError(receivedError, errorMessage)
}

func TestRunExportFailsIfFileCreationFails(t *testing.T) {
	appFS = afero.NewReadOnlyFs(afero.NewMemMapFs())
	assert := assert.New(t)
	errorMessage := "operation not permitted"

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

	puppetCode := New()
	puppetCode.Token = token
	puppetCode.Client = api
	_, receivedError := puppetCode.GetExportFile(filePath, "none")
	assert.EqualError(receivedError, errorMessage, "Archive file creation should fail")
}

func TestRunExportSucces(t *testing.T) {
	appFS = afero.NewMemMapFs()
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

	var mockPayload io.Writer
	result := &operations.GetExportOK{
		Payload: mockPayload,
	}

	getExportParameters := operations.NewGetExportParamsWithContext(context.Background())
	anon := "none"
	getExportParameters.SetAnonymizationProfile(&anon)
	operationsMock.EXPECT().GetExport(getExportParameters, match.XAuthenticationWriter(t, "my token"), gomock.Any()).Return(result, nil)

	puppetDb := New()
	puppetDb.Token = token
	puppetDb.Client = api

	_, err := puppetDb.GetExportFile(filePath, "none")
	_, stat := appFS.Stat(filePath)

	assert.NoError(err)
	assert.NoError(stat, "archive file should be created")
}

func TestRunExportError(t *testing.T) {
	appFS = afero.NewMemMapFs()
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

	result := operations.NewGetExportDefault(404)
	result.Payload = &models.Error{
		Msg:     "error message",
		Details: "details",
	}

	getExportParameters := operations.NewGetExportParamsWithContext(context.Background())
	anon := "none"
	getExportParameters.SetAnonymizationProfile(&anon)
	operationsMock.EXPECT().GetExport(getExportParameters, match.XAuthenticationWriter(t, "my token"), gomock.Any()).Return(nil, result)

	puppetCode := New()
	puppetCode.Token = token
	puppetCode.Client = api
	res, err := puppetCode.GetExportFile("/tmp/archive.tar.gz", "none")

	assert.Nil(res)
	assert.EqualError(err, "[GET /pdb/admin/v1/archive][404] getExport default  &{Details:details Kind: Msg:error message}")

	_, stat := appFS.Stat(filePath)
	expectedMessage := fmt.Sprintf("open %s: file does not exist", filepath.Join(filePath))
	assert.EqualError(stat, expectedMessage, "GetExportFile should clean up if API call fails")
}
