# Generate source files from a template based on the project name.

# Setup compiling the test executable. C++ compile flags are inherited from the parent directory.
include_directories(
  ../inc
  ${CURL_INCLUDE_DIRS}
  ${Boost_INCLUDE_DIRS}
  ${LEATHERMAN_CATCH_INCLUDE}
)

set(TEST_CASES ${PROJECT_NAME}.cc)

add_executable(libpuppetdb_test $<TARGET_OBJECTS:libprojectsrc> ${TEST_CASES} main.cc)
target_link_libraries(libpuppetdb_test
    ${Boost_LIBRARIES}
    ${LEATHERMAN_LIBRARIES}
    ${CURL_LIBRARIES}
)

add_test(NAME "unit_tests" COMMAND libpuppetdb_test)
