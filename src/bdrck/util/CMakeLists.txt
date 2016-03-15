set(bdrck-util_SOURCES

	Error.cpp
	Error.hpp
	floatCompare.hpp
	ScopeExit.cpp
	ScopeExit.hpp
	UUID.cpp
	UUID.hpp

)

add_library(bdrck-util STATIC ${bdrck-util_SOURCES})
target_link_libraries(bdrck-util

	${bdrck_LIBRARIES}

)

include(${PROJECT_SOURCE_DIR}/3rdparty/cmu/cmake/IdeSplitSources.cmake)
cmuIdeSplitSources(bdrck-util)
