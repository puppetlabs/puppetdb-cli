package json

import (
	"bytes"
	"encoding/json"
	"fmt"
	"os"

	"github.com/puppetlabs/puppet-code/log"
)

// PrettyPrintPayload returns a payload as pretty-printed string
func PrettyPrintPayload(payload interface{}) string {
	if payload == nil {
		return ""
	}
	e, err := json.MarshalIndent(payload, "", "  ")
	if err != nil {
		log.Debug(err.Error())
		return ""
	}
	return fmt.Sprint(string(e))
}

//WritePayload prints payload to output
func WritePayload(output *os.File, payload interface{}) error {
	e, err := MarshalIndent(payload, "", "  ")
	if err != nil {
		return err
	}

	_, err = output.Write(e)
	if err != nil {
		return err
	}
	return nil
}

// Marshal without html escaping
func Marshal(t interface{}) ([]byte, error) {
	buffer := &bytes.Buffer{}
	encoder := json.NewEncoder(buffer)
	encoder.SetEscapeHTML(false)
	err := encoder.Encode(t)
	return buffer.Bytes(), err
}

// MarshalIndent without html escaping
func MarshalIndent(v interface{}, prefix, indent string) ([]byte, error) {
	b, err := Marshal(v)
	if err != nil {
		return nil, err
	}
	var buf bytes.Buffer
	err = json.Indent(&buf, b, prefix, indent)
	if err != nil {
		return nil, err
	}
	return buf.Bytes(), nil
}
