# Generate source files from a template based on the project name.

# Set C++ compile flags based on project settings.
set(CMAKE_CXX_FLAGS ${${PROJECT_NAME_UPPER}_CXX_FLAGS})

# Set prefix for logging messages.
leatherman_logging_namespace("puppetlabs.${PROJECT_NAME}.command")

# Setup compiling the executable.
include_directories(
    ../lib/inc
    ${Boost_INCLUDE_DIRS}
    ${LEATHERMAN_LOGGING_INCLUDE}
    ${LEATHERMAN_JSON_CONTAINER_INCLUDE}
)

add_executable(puppet-query puppet-query.cc)
target_link_libraries(puppet-query
    lib${PROJECT_NAME}
    ${Boost_PROGRAM_OPTIONS_LIBRARY}
    ${Boost_FILESYSTEM_LIBRARY}
    ${LEATHERMAN_LIBRARIES}
)

add_executable(puppet-db puppet-db.cc)
target_link_libraries(puppet-db
    lib${PROJECT_NAME}
    ${Boost_PROGRAM_OPTIONS_LIBRARY}
    ${Boost_FILESYSTEM_LIBRARY}
    ${LEATHERMAN_LOGGING_LIB}
    ${LEATHERMAN_JSON_CONTAINER_LIB}
)

leatherman_install(puppet-query puppet-db)

# Tests for the executable. These don't verify behavior, simply that the
# executable runs without crashing or generating an error. Useful, but should be
# enhanced with test scripts and/or acceptance tests.

# We duplicate the `help_test` as `smoke_test` until the default functionality
# of the tool has been nailed down.
add_test(NAME "db_smoke_test" COMMAND puppet-db)
add_test(NAME "db_help_test" COMMAND puppet-db --help)
add_test(NAME "db_version_test" COMMAND puppet-db --version)
add_test(NAME "query_help_test" COMMAND puppet-query --help)
add_test(NAME "query_version_test" COMMAND puppet-query --version)
