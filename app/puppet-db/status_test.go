package app

import (
	"context"
	"errors"
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

func TestRunStatusFailsIfNoClient(t *testing.T) {
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
	_, receivedError := puppetCode.GetStatus()
	assert.EqualError(receivedError, errorMessage)
}

func TestRunStatusSucces(t *testing.T) {
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

	result := &operations.GetStatusOK{
		Payload: "ok",
	}

	getStatusParameters := operations.NewGetStatusParamsWithContext(context.Background())
	operationsMock.EXPECT().GetStatus(getStatusParameters, match.XAuthenticationWriter(t, "my token")).Return(result, nil)

	puppetCode := New()
	puppetCode.Token = token
	puppetCode.Client = api
	res, err := puppetCode.GetStatus()

	assert.Equal("ok", res.Payload)
	assert.Nil(err)
}

func TestRunStatusError(t *testing.T) {
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

	result := operations.NewGetStatusDefault(404)
	result.Payload = &models.Error{
		Msg:     "error message",
		Details: "details",
	}

	getStatusParameters := operations.NewGetStatusParamsWithContext(context.Background())
	operationsMock.EXPECT().GetStatus(getStatusParameters, match.XAuthenticationWriter(t, "my token")).Return(nil, result)

	puppetCode := New()
	puppetCode.Token = token
	puppetCode.Client = api
	res, err := puppetCode.GetStatus()

	assert.Nil(res)
	assert.EqualError(err, "[GET /status/v1/services][404] getStatus default  &{Details:details Kind: Msg:error message}")
}
