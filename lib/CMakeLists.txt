# Generate source files from a template based on the project name.

include(leatherman)
get_commit_string(PROJECT_COMMIT)

# Generate a file containing the above version numbers.
configure_file("version.h.in" "${CMAKE_CURRENT_LIST_DIR}/inc/${PROJECT_NAME}/version.h")

# Generate files for Doxygen
configure_file("Doxyfile.in"       "${CMAKE_CURRENT_LIST_DIR}/Doxyfile")
configure_file("namespaces.dox.in" "${CMAKE_CURRENT_LIST_DIR}/docs/namespaces.dox")

# Set C++ compile flags based on project settings.
set(CMAKE_CXX_FLAGS ${${PROJECT_NAME_UPPER}_CXX_FLAGS})

# Set prefix for logging messages.
leatherman_logging_namespace("puppetlabs.puppetdb")

# Setup compiling the library.
include_directories(
  inc
  ${CURL_INCLUDE_DIRS}
  ${Boost_INCLUDE_DIRS}
  ${LEATHERMAN_INCLUDE_DIRS}
)

set(PROJECT_SOURCES "src/${PROJECT_NAME}.cc")

## An object target is generated that can be used by both the library and test executable targets.
## Without the intermediate target, unexported symbols can't be tested.
add_library(libprojectsrc OBJECT ${PROJECT_SOURCES})
set_target_properties(libprojectsrc PROPERTIES POSITION_INDEPENDENT_CODE true)

add_library(libpuppetdb $<TARGET_OBJECTS:libprojectsrc>)
set_target_properties(libpuppetdb PROPERTIES VERSION "${PROJECT_VERSION_MAJOR}.${PROJECT_VERSION_MINOR}.${PROJECT_VERSION_PATCH}")
target_link_libraries(libpuppetdb
    ${Boost_LIBRARIES}
    ${LEATHERMAN_LIBRARIES}
    ${CURL_LIBRARIES}
)

symbol_exports(libpuppetdb "${CMAKE_CURRENT_LIST_DIR}/inc/${PROJECT_NAME}/export.h")

leatherman_install(libpuppetdb)
install(DIRECTORY inc/${PROJECT_NAME} DESTINATION include)

add_subdirectory(tests)
