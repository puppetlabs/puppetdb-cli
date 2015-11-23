/**
 * @file
 * Declares a utility for retrieving the library version.
 */
#pragma once

#include <string>
#include "export.h"

#include <leatherman/json_container/json_container.hpp>

namespace puppetdb_cli {


    /**
     * Query the library version.
     * @return A version string with \<major>.\<minor>.\<patch>
     */
    std::string LIBPUPPETDB_CLI_EXPORT version();

    /**
    * Parse a `puppetdb-cli` config at `~/.pdbrc`.
    * @return A JsonContainer of the config.
    */
    leatherman::json_container::JsonContainer LIBPUPPETDB_CLI_EXPORT parse_config();

    /**
     * Query a PuppetDB endpoint for a given config.
     * @param config JsonContainer of the cli configuration.
     * @param endpoint string of the PuppetDB endpoint to query.
     * @param query_string JSON encoded string to query PuppetDB with.
     * @param limit integer paging option for PuppetDB query
     * @return A string of the response from a PuppetDB query.
     */
    std::string LIBPUPPETDB_CLI_EXPORT query(const leatherman::json_container::JsonContainer& config,
                                             const std::string& endpoint,
                                             const std::string& query_string,
                                             const int limit);

}  // namespace puppetdb_cli
