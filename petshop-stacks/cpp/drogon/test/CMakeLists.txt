include(FetchContent)
FetchContent_Declare(
    doctest
    URL https://github.com/doctest/doctest/releases/download/v2.4.11/doctest.h
    DOWNLOAD_NO_EXTRACT TRUE
)
FetchContent_MakeAvailable(doctest)

add_executable(petstore-tests
    test_helpers.cc
    test_config.cc
    ../src/helpers.cc
    ../src/config.cc
)

target_include_directories(petstore-tests PRIVATE ../src ${doctest_SOURCE_DIR})

find_package(nlohmann_json CONFIG QUIET)
if(nlohmann_json_FOUND)
    target_link_libraries(petstore-tests PRIVATE nlohmann_json::nlohmann_json)
endif()

add_test(NAME petstore-unit-tests COMMAND petstore-tests)
