A configuration file may be used to retain settings. The file is in JSON format
and should be placed in ~/.puppetlabs/client-tools/FILE_NAME_TBD.conf

```json
    {
      "default_instance": "pdb-prod",
      "instances": {
        "pdb-prod": {
          "root_url": "https://localhost:8081",
          "ca_cert": "/Users/<home>/certs/ca.pem",
          "cert": "/Users/<home>/pdbq-cert.pem",
          "key": "/Users/<home>/test-certs/pdbq.pem"
        },
        "pdb-dev": {
          "root_url": "http://localhost:8080"
        }
      },
      "options": {
        "url": "https://master.example.com:8143",
        "environment": "production",
        "format": "yaml"
      }
    }
```