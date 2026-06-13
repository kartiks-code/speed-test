cmake_minimum_required(VERSION 3.16)
project(petstore-drogon CXX)
set(CMAKE_CXX_STANDARD 17)
set(CMAKE_CXX_STANDARD_REQUIRED ON)

find_package(Drogon CONFIG REQUIRED)

add_executable(petstore-drogon
    src/main.cc
    src/config.cc
    src/helpers.cc
    src/db.cc
    src/controllers/PetController.cc
    src/controllers/StoreController.cc
    src/controllers/UserController.cc
)

target_include_directories(petstore-drogon PRIVATE src)
target_link_libraries(petstore-drogon PRIVATE Drogon::Drogon)

enable_testing()
add_subdirectory(test)
