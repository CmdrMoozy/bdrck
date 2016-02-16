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

if(WIN32)
	set(Boost_USE_STATIC_LIBS ON)
	set(Boost_USE_MULTITHREADED ON)
	set(Boost_USE_STATIC_RUNTIME ON)

	set(BOOST_INCLUDEDIR ${BOOST_ROOT}/boost)
	set(BOOST_LIBRARYDIR ${BOOST_ROOT}/stage/lib)
endif()

find_package(Boost REQUIRED
	COMPONENTS
		filesystem
		system
	)
find_package(RE2 REQUIRED)
find_package(Yajl REQUIRED)
find_package(Libgit2 REQUIRED)

# Setup our compile flags.

include(${CMAKE_CURRENT_SOURCE_DIR}/3rdparty/cmu/cmake/SetFlags.cmake)
cmuSetCompileFlags()

if(CMAKE_BUILD_TYPE_LOWER MATCHES debug)
	add_definitions(-DLIBBDRCK_DEBUG)
endif()

include_directories(src)

include_directories(SYSTEM

	3rdparty/include
	${Boost_INCLUDE_DIRS}
	${RE2_INCLUDE_DIRS}
	${YAJL_INCLUDE_DIRS}
	${LIBGIT2_INCLUDE_DIRS}

)

# Set the libraries we'll use.

set(bdrck_LIBRARIES

	${Boost_LIBRARIES}
	${RE2_LIBRARIES}
	${YAJL_LIBRARIES}
	${LIBGIT2_LIBRARIES}

)

# Add our targets.

enable_testing()

add_subdirectory(src/bdrck/algorithm)
add_subdirectory(src/bdrck/fs)
add_subdirectory(src/bdrck/git)
add_subdirectory(src/bdrck/json)
add_subdirectory(src/bdrck/params)
add_subdirectory(src/bdrck/process)
add_subdirectory(src/bdrck/string)
add_subdirectory(src/bdrck/util)

add_subdirectory(src/bdrck/test)
