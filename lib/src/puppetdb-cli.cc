#include <puppetdb-cli/version.h>
#include <puppetdb-cli/puppetdb-cli.hpp>

#include <leatherman/logging/logging.hpp>

namespace puppetdb_cli {

    using namespace std;

    string version()
    {
        LOG_DEBUG("puppetdb-cli version is %1%", PUPPETDB_CLI_VERSION_WITH_COMMIT);
        return PUPPETDB_CLI_VERSION_WITH_COMMIT;
    }

}  // puppetdb_cli
