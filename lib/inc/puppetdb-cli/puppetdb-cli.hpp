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
 * Implements a PuppetDB Connection
 */
class LIBPUPPETDB_EXPORT PuppetDBConn  {
  public:
    /**
     * Construct a PuppetDB connection using defaults
     */
    PuppetDBConn();

    /**
     * Construct a PuppetDB connection using a config
     * @param config JsonContainer of the cli configuration.
     */
    PuppetDBConn(const leatherman::json_container::JsonContainer& config);

    ~PuppetDBConn() {}

    /**
     * Get a server_url from the PuppetDB config
     * @return the first server_url in the config
     */
    std::string getServerUrl() const;

    /**
     * Get a cURL handle with SSL configuration attached
     * @return a unique_ptr to a cURL handle
     */
    std::unique_ptr<CURL, std::function<void(CURL*)> > getCurlHandle() const;


  private:
    /**
     * Parse the config for server_urls
     * @param JsonContainer of your PuppetDB CLI configuration
     * @return a list of server_urls
     */
    server_urls_t parseServerUrls(const leatherman::json_container::JsonContainer& config);
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
 * @return PuppetDBConn for connecting to PuppetDB.
 */
LIBPUPPETDB_EXPORT PuppetDBConn get_puppetdb(const std::string& config_path);

/**
 * Query a PuppetDB endpoint for a given config.
 * @param config JsonContainer of the cli configuration.
 * @param query string of the query for PuppetDB (can be either AST or PQL syntax).
 * @return This function does not return anything.
 */
LIBPUPPETDB_EXPORT void pdb_query(const PuppetDBConn& conn,
                                  const std::string& query);

/**
 * Export a PuppetDB archive for a given config.
 * @param config JsonContainer of the cli configuration.
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
