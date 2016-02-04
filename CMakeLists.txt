# Defines how cmake should behave, and the minimum version necessary to build.
cmake_minimum_required(VERSION 3.2.2)
project(puppetdb-cli VERSION 0.1.0)

configure_file("${CMAKE_SOURCE_DIR}/version.in" "${CMAKE_BINARY_DIR}/version")

string(MAKE_C_IDENTIFIER ${PROJECT_NAME} PROJECT_C_NAME)
string(TOUPPER ${PROJECT_C_NAME} PROJECT_NAME_UPPER)
string(TOLOWER ${PROJECT_C_NAME} PROJECT_NAME_LOWER)

# Common cmake setup
if (NOT CMAKE_BUILD_TYPE)
    message(STATUS "Defaulting to a release build.")
    set(CMAKE_BUILD_TYPE "Release" CACHE STRING "Choose the type of build, options are: None Debug Release RelWithDebInfo MinSizeRel." FORCE)
endif()

enable_testing()

# pod2man setup
list(APPEND CMAKE_MODULE_PATH "${PROJECT_SOURCE_DIR}/cmake")

include("${PROJECT_SOURCE_DIR}/cmake/pod2man.cmake")
file(MAKE_DIRECTORY "${PROJECT_BINARY_DIR}/share/man")
pod2man(
  ${PROJECT_SOURCE_DIR}/man/puppet-db.pod
  "puppet-db" 8 share/man
  "manages PuppetDB administrative tasks"
)

pod2man(
  ${PROJECT_SOURCE_DIR}/man/puppet-query.pod
  "puppet-query" 8 share/man
  "queries PuppetDB data"
)

pod2man(
  ${PROJECT_SOURCE_DIR}/man/puppetdb_conf.pod
  "puppetdb_conf" 8 share/man
  "PuppetDB CLI configuration"
)

# Leatherman setup
include(FeatureSummary)
SET(LEATHERMAN_COMPONENTS locale catch nowide logging util file_util rapidjson json_container)
find_package(Leatherman REQUIRED COMPONENTS ${LEATHERMAN_COMPONENTS})

## Before we find any packages, we want to pull in the common leatherman options, as they can affect commonly-used packages.
include(options)
## Pull in common cflags setting from leatherman. Don't override CMAKE_CXX_FLAGS at the project root to avoid impacting 3rd party code.
include(cflags)
set(${PROJECT_NAME_UPPER}_CXX_FLAGS "${LEATHERMAN_CXX_FLAGS}")
add_definitions(${LEATHERMAN_DEFINITIONS})
## Pull in helper macros for working with leatherman libraries
include(leatherman)

# Add other dependencies
find_package(Boost 1.54 REQUIRED COMPONENTS program_options filesystem)
find_package(CURL REQUIRED)

if (CURL_STATIC)
    add_definitions(-DCURL_STATICLIB)
    if (WIN32)
      # Linking statically on Windows requires some extra libraries.
      set(CURL_DEPS wldap32.lib ws2_32.lib)
    endif()
endif()

list(APPEND CURL_LIBRARIES ${CURL_DEPS})
export_var(CURL_LIBRARIES)

# Display a summary of the features
include(FeatureSummary)
feature_summary(WHAT ALL)

add_subdirectory(lib)
add_subdirectory(exe)

# Add cpplint and cppcheck targets
file(GLOB_RECURSE ALL_SOURCES lib/src/*.cc lib/inc/*.hpp exe/*.cc)
add_cpplint_files(${ALL_SOURCES})
enable_cpplint()

add_cppcheck_dirs("${PROJECT_SOURCE_DIR}/lib" "${PROJECT_SOURCE_DIR}/exe")
enable_cppcheck()


# We duplicate the `help_test` as `smoke_test` until the default functionality
# of the tool has been nailed down.
add_test(NAME "db_smoke_test" COMMAND puppet-db)
add_test(NAME "db_help_test" COMMAND puppet-db --help)
add_test(NAME "query_help_test" COMMAND puppet-query --help)

if (NOT WIN32)
  add_test(test-db-version test/test-db-version)
  add_test(test-query-version test/test-query-version)
else()
  add_test(NAME "db_version_test" COMMAND puppet-db --version)
  add_test(NAME "query_version_test" COMMAND puppet-query --version)
endif()
