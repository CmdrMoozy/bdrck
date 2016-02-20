set(bdrck-algorithm_SOURCES

	String.cpp
	String.hpp

)

add_library(bdrck-algorithm STATIC ${bdrck-algorithm_SOURCES})
target_link_libraries(bdrck-algorithm

	${bdrck_LIBRARIES}

)

include(${PROJECT_SOURCE_DIR}/3rdparty/cmu/cmake/IdeSplitSources.cmake)
cmuIdeSplitSources(bdrck-algorithm)
