package testing

import (
	"fmt"
	"testing"

	"github.com/golang/mock/gomock"

	"github.com/go-openapi/runtime"
	"github.com/go-openapi/strfmt"
)

type authToken struct {
	s string
	t *testing.T
}

//XAuthenticationWriter matcher
func XAuthenticationWriter(t *testing.T, s string) gomock.Matcher {
	return &authToken{
		t: t,
		s: s,
	}
}

func (o *authToken) Matches(x interface{}) bool {
	a, ok := x.(runtime.ClientAuthInfoWriterFunc)
	if !ok {
		return false
	}

	ctrl := gomock.NewController(o.t)
	defer ctrl.Finish()

	clientRequest := NewMockClientRequest(ctrl)
	clientRequest.EXPECT().SetHeaderParam("X-Authentication", o.s)
	a(clientRequest, strfmt.Default)

	return true
}

func (o *authToken) String() string {
	return fmt.Sprintf("expected ClientAuthInfoWriter of \"%s\"", o.s)
}
