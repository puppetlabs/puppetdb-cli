/**
 * @file
 * Declares a utility for retrieving the library version.
 */
#pragma once

#include <string>
#include "export.h"

namespace puppetdb_cli {

    /**
     * Query the library version.
     * @return A version string with \<major>.\<minor>.\<patch>
     */
    std::string LIBPUPPETDB_CLI_EXPORT version();

}  // namespace puppetdb_cli
