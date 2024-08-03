cmake_minimum_required(VERSION 3.10)
project(TinyServer)

set(CMAKE_CXX_STANDARD 17)

# 添加 Google Test
include(FetchContent)
FetchContent_Declare(
    googletest
    GIT_REPOSITORY https://github.com/google/googletest.git
    GIT_TAG release-1.11.0
)
FetchContent_MakeAvailable(googletest)

# 添加 Google Benchmark
FetchContent_Declare(
    benchmark
    GIT_REPOSITORY https://github.com/google/benchmark.git
    GIT_TAG v1.6.1
)
FetchContent_MakeAvailable(benchmark)

# MySQL 配置
set(CMAKE_MODULE_PATH ${CMAKE_MODULE_PATH} "${CMAKE_SOURCE_DIR}/cmake/")
find_package(MySQL REQUIRED)
if(NOT MySQL_FOUND)
    message(FATAL_ERROR "MySQL not found. Please install MySQL development libraries.")
endif()

# 定义头文件目录
set(HEADER_DIR ${CMAKE_SOURCE_DIR}/cpp_version/include)

# 主项目源文件
file(GLOB_RECURSE SOURCES "cpp_version/src/*.cpp")
add_library(TinyServer ${SOURCES})
target_include_directories(TinyServer PUBLIC ${HEADER_DIR} ${MySQL_INCLUDE_DIRS})
target_link_libraries(TinyServer ${MySQL_LIBRARIES})

# 单元测试
enable_testing()
file(GLOB_RECURSE TEST_SOURCES "cpp_version/tests/*.cpp")
add_executable(unit_tests ${TEST_SOURCES})
target_link_libraries(unit_tests TinyServer gtest gtest_main pthread)
add_test(NAME unit_tests COMMAND unit_tests)

# 性能测试
add_executable(benchmark_tcp_server cpp_version/benchmarks/benchmark_tcp_server.cpp)
target_link_libraries(benchmark_tcp_server TinyServer benchmark::benchmark pthread)

# 示例应用(http_server)
add_executable(http_server cpp_version/src/http/http_server.cpp)
target_link_libraries(http_server 
    TinyServer 
    pthread 
    ${MySQL_LIBRARIES}
)
target_include_directories(http_server PRIVATE ${HEADER_DIR} ${MySQL_INCLUDE_DIRS})