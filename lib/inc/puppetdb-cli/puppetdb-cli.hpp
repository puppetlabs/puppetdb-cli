/**
 * @file
 * Declares a utility for retrieving the library version.
 */
#pragma once

#include <string>
#include <curl/curl.h>
#include "export.h"

#include <leatherman/json_container/json_container.hpp>

namespace puppetdb {

/**
 * Typedef for server_urls.
 */
LIBPUPPETDB_EXPORT typedef std::vector<std::string> server_urls_t;

/**
 * SSL credentials for cURL
 */
struct LIBPUPPETDB_EXPORT SSLCredentials {
    /// cacert string cacert to use for curl.
    const std::string cacert;
    /// cert client cert to use for curl.
    const std::string cert;
    /// key client private key to use for curl.
    const std::string key;
};

/**
 * Implements a PuppetDB Connection
 */
class LIBPUPPETDB_EXPORT PuppetDBConn  {
  public:
    /**
     * Construct a PuppetDB connection using defaults
     */
    PuppetDBConn();

    /**
     * Construct a PuppetDB connection using only config flags
     * @param urls string of the urls of PuppetDB.
     * @param ssl_creds SSLCrendentials ssl auth credentials to use for curl.
     */
    PuppetDBConn(const std::string& urls,
                 const SSLCredentials& ssl_creds);

    /**
     * Construct a PuppetDB connection using a config
     * @param config JsonContainer of the cli configuration.
     * @param urls string of the urls of PuppetDB.
     * @param ssl_creds SSLCrendentials ssl auth credentials to use for curl.
     */
    PuppetDBConn(const leatherman::json_container::JsonContainer& config,
                 const std::string& urls,
                 const SSLCredentials& ssl_creds);

    ~PuppetDBConn() {}

    /**
     * Get a server_url from the PuppetDB config
     * @return the first server_url in the config
     */
    std::string getServerUrl() const;

    /**
     * Get the SSLCredentials from the PuppetDB config
     * @return the SSLCredentials for the PuppetDB connection
     */
    SSLCredentials getSSLCredentials() const;


  private:
    /// List of PuppetDB server_urls
    server_urls_t server_urls_;
    /// The cacert for SSL connections
    std::string cacert_;
    /// The client cert for SSL connections
    std::string cert_;
    /// The client key for SSL connections
    std::string key_;
};



/**
 * Query the library version.
 * @return A version string with \<major>.\<minor>.\<patch>
 */
LIBPUPPETDB_EXPORT std::string version();

/**
 * Parse a `puppetdb-cli` config file and return a PuppetDB connection.
 * @param config_path string path to your PuppetDB CLI configuration.
 * @param urls string of the urls of PuppetDB.
 * @param ssl_creds SSLCrendentials ssl auth credentials to use for curl.
 * @return PuppetDBConn for connecting to PuppetDB.
 */
LIBPUPPETDB_EXPORT PuppetDBConn get_puppetdb(const std::string& config_path,
                                             const std::string& urls,
                                             const SSLCredentials& ssl_creds);

/**
 * Query a PuppetDB endpoint for a given config.
 * @param conn PuppetDBConn of the cli configuration.
 * @param query string of the query for PuppetDB (can be either AST or PQL syntax).
 * @return This function does not return anything.
 */
LIBPUPPETDB_EXPORT void pdb_query(const PuppetDBConn& conn,
                                  const std::string& endpoint,
                                  const std::string& query);

/**
 * Export a PuppetDB archive for a given config.
 * @param conn PuppetDBConn of the cli configuration.
 * @param path string of the file path to which to stream the archive.
 * @param anonymization string of the anonymization to apply to the archive.
 * @return This function does not return anything.
 */
LIBPUPPETDB_EXPORT void pdb_export(const PuppetDBConn& conn,
                                   const std::string& path,
                                   const std::string& anonymization);

/**
 * Upload a PuppetDB archive to an instance of PuppetDB.
 * @param config JsonContainer of the CLI configuration.
 * @param infile string path to archive file for upload.
*/
LIBPUPPETDB_EXPORT void pdb_import(const PuppetDBConn& conn,
                                   const std::string& infile);

}  // namespace puppetdb
