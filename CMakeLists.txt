cmake_minimum_required(VERSION 3.0.2)
project(bdrck)

set(CMAKE_COLOR_MAKEFILE ON)
set(CMAKE_VERBOSE_MAKEFILE OFF)

set(CMAKE_MODULE_PATH ${CMAKE_CURRENT_SOURCE_DIR}/3rdparty/cmu/cmake)

# Note which type of build we're performing.

if(NOT CMAKE_BUILD_TYPE)
	message(STATUS "No CMAKE_BUILD_TYPE specified; defaulting to Release.")
	set(CMAKE_BUILD_TYPE "Release")
endif()

string(TOLOWER "${CMAKE_BUILD_TYPE}" CMAKE_BUILD_TYPE_LOWER)

# Find our dependencies.

find_package(RE2 REQUIRED)
find_package(Libgit2 REQUIRED)

# Setup our compile flags.

include(${CMAKE_CURRENT_SOURCE_DIR}/3rdparty/cmu/cmake/SetFlags.cmake)
cmuSetCompileFlags()

if(CMAKE_BUILD_TYPE_LOWER MATCHES debug)
	add_definitions(-DLIBBDRCK_DEBUG)
endif()

include_directories(3rdparty/include include)

# Set the libraries we'll use.

set(bdrck_LIBRARIES

	${RE2_LIBRARIES}
	${LIBGIT2_LIBRARIES}

)

# Add our targets.

enable_testing()

add_subdirectory(src/algorithm)
add_subdirectory(src/cwrap)
add_subdirectory(src/fs)
add_subdirectory(src/git)
add_subdirectory(src/params)
add_subdirectory(src/process)
add_subdirectory(src/string)
add_subdirectory(src/util)

add_subdirectory(src/test)
