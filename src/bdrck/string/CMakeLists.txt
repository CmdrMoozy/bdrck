set(bdrck-string_SOURCES

	RegEx.cpp
	RegEx.hpp
	StringRef.cpp
	StringRef.hpp

)

add_library(bdrck-string STATIC ${bdrck-string_SOURCES})
target_link_libraries(bdrck-string

	${bdrck_LIBRARIES}

)

include(${PROJECT_SOURCE_DIR}/3rdparty/cmu/cmake/IdeSplitSources.cmake)
cmuIdeSplitSources(bdrck-string)
