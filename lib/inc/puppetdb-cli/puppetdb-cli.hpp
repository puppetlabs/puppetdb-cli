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
 * Query a PuppetDB endpoint for a given config.
 * @param config JsonContainer of the cli configuration.
 * @param query string of the query for PuppetDB (can be either AST or PQL syntax).
 * @return A leatherman::curl::response of the response from a PuppetDB query.
 */
void LIBPUPPETDB_CLI_EXPORT
pdb_query(const leatherman::json_container::JsonContainer& config,
          const std::string& query);

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

/**
 * Upload a PuppetDB archive to an instance of PuppetDB.
 * @param config JsonContainer of the CLI configuration.
 * @param infile string path to archive file for upload.
 * @param command_versions string json object containing PuppetDB command
 * versions to use on import.
 * Example: '{"replace_facts":4,"store_report":6,"replace_catalog":7}'
*/

void LIBPUPPETDB_CLI_EXPORT
pdb_import(const leatherman::json_container::JsonContainer& config,
           const std::string& infile,
           const std::string& command_versions);

}  // namespace puppetdb_cli
