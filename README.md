# puppetdb-cli

## Configuration
Example file to place at `~/.pdbrc`:
```json
{"default_environment":"prod",
 "environments": {"prod": {"root_url":"https://alpha-rho.local:8081",
                           "ca":"<path to ca.pem>",
                           "cert":"<path to cert .pem>",
                           "key":"<path to private-key .pem>",
                  "dev": {"root_url":"http://localhost:8080"}}}
```
