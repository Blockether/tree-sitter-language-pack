cmake_minimum_required(VERSION 3.20)
project(test_app_c C)

set(CMAKE_C_STANDARD 11)

# Link against ts-pack-ffi from installed location or GitHub release
find_library(TS_PACK_FFI ts_pack_ffi HINTS ${CMAKE_PREFIX_PATH}/lib)
find_path(TS_PACK_INCLUDE ts_pack_ffi.h HINTS ${CMAKE_PREFIX_PATH}/include)

if(NOT TS_PACK_FFI OR NOT TS_PACK_INCLUDE)
    message(FATAL_ERROR "ts-pack-ffi library not found. Set CMAKE_PREFIX_PATH to installation directory.")
endif()

add_executable(smoke_test smoke_test.c)
target_include_directories(smoke_test PRIVATE ${TS_PACK_INCLUDE})
target_link_libraries(smoke_test PRIVATE ${TS_PACK_FFI})
