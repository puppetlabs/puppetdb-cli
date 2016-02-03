#include <catch.hpp>
#include <puppetdb-cli/version.h>
#include <puppetdb-cli/puppetdb-cli.hpp>

SCENARIO("version() returns the version") {
    REQUIRE(puppetdb::version() == PUPPETDB_CLI_VERSION_WITH_COMMIT);
}
