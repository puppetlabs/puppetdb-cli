package app

// PuppetDb interface
type PuppetDb struct {
	Version string
	Options PuppetDbOptions
}

// PuppetDbOptions stores all global options
type PuppetDbOptions struct {
	Debug    bool
	LogLevel string
	Config   string
	Anon     string
	Urls     string
	Cacert   string
	Cert     string
	Key      string
	Token    string
}

// NewPuppetDbApp FIXME
func NewPuppetDbApp(version string) *PuppetDb {
	return &PuppetDb{
		Version: version,
	}
}
