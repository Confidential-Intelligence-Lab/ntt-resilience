cmake_minimum_required(VERSION 3.21)

project(heaan_ccmm_oracle LANGUAGES CXX)

set(CMAKE_CXX_STANDARD 17)
set(CMAKE_CXX_STANDARD_REQUIRED ON)

set(
    SUBMISSION_720_ROOT
    "${CMAKE_CURRENT_LIST_DIR}/../../external/submission-720-impl"
)

if(APPLE AND CMAKE_SYSTEM_PROCESSOR MATCHES "arm64")
    list(
        APPEND CMAKE_PREFIX_PATH
        "${SUBMISSION_720_ROOT}/HEaaN/macos-arm64"
    )

    set(
        OpenMP_CXX_FLAGS
        "-Xpreprocessor -fopenmp -I/opt/homebrew/opt/libomp/include"
    )
    set(OpenMP_CXX_LIB_NAMES "omp")
    set(
        OpenMP_omp_LIBRARY
        "/opt/homebrew/opt/libomp/lib/libomp.dylib"
    )
elseif(UNIX AND NOT APPLE)
    list(
        APPEND CMAKE_PREFIX_PATH
        "${SUBMISSION_720_ROOT}/HEaaN/linux-x86_64"
    )
else()
    message(
        FATAL_ERROR
        "Unsupported platform for bundled HEaaN reference"
    )
endif()

find_package(HEaaN REQUIRED)
find_package(OpenMP REQUIRED)

find_library(
    FLINT_LIBRARIES
    NAMES flint flint-3.3.1
    HINTS /usr/local/lib /opt/homebrew/lib
    REQUIRED
)

add_subdirectory(
    "${SUBMISSION_720_ROOT}/matrix-mult"
    "${CMAKE_BINARY_DIR}/matrix-mult"
)

add_executable(ccmm-oracle CCMMOracle.cpp)

target_include_directories(
    ccmm-oracle
    PRIVATE
    "${SUBMISSION_720_ROOT}/matrix-mult/include"
)

target_link_libraries(
    ccmm-oracle
    PRIVATE
    HEaaN::HEaaN
    matrix-mult
    ${FLINT_LIBRARIES}
    OpenMP::OpenMP_CXX
)
