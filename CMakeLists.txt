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

# 主项目源文件
file(GLOB_RECURSE SOURCES "cpp_version/src/*.cpp")
add_library(TinyServer ${SOURCES})
target_include_directories(TinyServer PUBLIC cpp_version/include)

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
add_executable(http_server cpp_version/examples/http_server.cpp)
target_link_libraries(http_server TinyServer pthread)
