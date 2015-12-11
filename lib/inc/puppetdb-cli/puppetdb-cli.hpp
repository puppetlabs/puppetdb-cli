/**
 * @file
 * Declares a utility for retrieving the library version.
 */
#pragma once

#include <string>
#include "export.h"

#include <leatherman/json_container/json_container.hpp>
#include <leatherman/curl/response.hpp>
#include <leatherman/curl/client.hpp>

namespace puppetdb_cli {


    /**
     * Query the library version.
     * @return A version string with \<major>.\<minor>.\<patch>
     */
    std::string LIBPUPPETDB_CLI_EXPORT
    version();

    /**
     * Parse a `puppetdb-cli` config at `~/.pdbrc`.
     * @return A JsonContainer of the config.
     */
    leatherman::json_container::JsonContainer LIBPUPPETDB_CLI_EXPORT
    parse_config();

    /**
     * Create a client for connecting to PuppetDB.
     * @param config JsonContainer of the cli configuration.
     * @return A leatherman::curl::client using credentials from config.
     */
    leatherman::curl::client LIBPUPPETDB_CLI_EXPORT
    pdb_client(const leatherman::json_container::JsonContainer& config);

    /**
     * Query a PuppetDB endpoint for a given config.
     * @param config JsonContainer of the cli configuration.
     * @param query JsonContainer of the query for PuppetDB.
     * @return A leatherman::curl::response of the response from a PuppetDB query.
     */
    leatherman::curl::response LIBPUPPETDB_CLI_EXPORT
    pdb_query(const leatherman::json_container::JsonContainer& config,
              const leatherman::json_container::JsonContainer& query);

    /**
    * Export a PuppetDB archive for a given config.
    * @param config JsonContainer of the cli configuration.
    * @param path string of the file path to which to stream the archive.
    * @param anonymization string of the anonymization to apply to the archive.
    * @return This function does not return anything.
    */
    void LIBPUPPETDB_CLI_EXPORT
    pdb_export(const leatherman::json_container::JsonContainer& config,
               const std::string& path,
               const std::string& anonymization);

}  // namespace puppetdb_cli
