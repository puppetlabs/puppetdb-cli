/**
 * @file
 * Declares a utility for retrieving the library version.
 */
#pragma once

#include <string>
#include "export.h"

#include <leatherman/json_container/json_container.hpp>
#include <leatherman/curl/response.hpp>

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
    leatherman::json_container::JsonContainer LIBPUPPETDB_CLI_EXPORT
    parse_config();

    /**
     * Query a PuppetDB endpoint for a given config.
     * @param config JsonContainer of the cli configuration.
     * @param query JsonContainer of the query for PuppetDB.
     * @return A leatherman::curl::response of the response from a PuppetDB query.
     */
    leatherman::curl::response LIBPUPPETDB_CLI_EXPORT
    query(const leatherman::json_container::JsonContainer& config,
          const leatherman::json_container::JsonContainer& query);

}  // namespace puppetdb_cli
