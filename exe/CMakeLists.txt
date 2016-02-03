# Generate source files from a template based on the project name.

# Set C++ compile flags based on project settings.
set(CMAKE_CXX_FLAGS ${${PROJECT_NAME_UPPER}_CXX_FLAGS})

# Set prefix for logging messages.
leatherman_logging_namespace("puppetlabs.puppetdb.command")

# Setup compiling the executable.
include_directories(
    ../lib/inc
    ${CURL_INCLUDE_DIRS}
    ${Boost_INCLUDE_DIRS}
    ${LEATHERMAN_LOGGING_INCLUDE}
    ${LEATHERMAN_JSON_CONTAINER_INCLUDE}
)

add_executable(puppet-query puppet-query.cc)
target_link_libraries(puppet-query
    libpuppetdb
    ${Boost_PROGRAM_OPTIONS_LIBRARY}
    ${Boost_FILESYSTEM_LIBRARY}
    ${LEATHERMAN_LIBRARIES}
)

add_executable(puppet-db puppet-db.cc)
target_link_libraries(puppet-db
    libpuppetdb
    ${Boost_PROGRAM_OPTIONS_LIBRARY}
    ${Boost_FILESYSTEM_LIBRARY}
    ${LEATHERMAN_LOGGING_LIB}
    ${LEATHERMAN_JSON_CONTAINER_LIB}
)

leatherman_install(puppet-query puppet-db)
