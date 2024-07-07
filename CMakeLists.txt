cmake_minimum_required(VERSION 3.10)
project(HighPerformanceHub)

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
file(GLOB_RECURSE SOURCES "src/*.cpp")
add_library(HighPerformanceHub ${SOURCES})
target_include_directories(HighPerformanceHub PUBLIC include)

# 单元测试
enable_testing()
file(GLOB_RECURSE TEST_SOURCES "tests/*.cpp")
add_executable(unit_tests ${TEST_SOURCES})
target_link_libraries(unit_tests HighPerformanceHub gtest gtest_main pthread)
add_test(NAME unit_tests COMMAND unit_tests)

# 性能测试
add_executable(benchmark_tcp_server benchmarks/benchmark_tcp_server.cpp)
target_link_libraries(benchmark_tcp_server HighPerformanceHub benchmark::benchmark pthread)

# 示例应用
add_executable(chat_server examples/chat_server.cpp)
target_link_libraries(chat_server HighPerformanceHub pthread)