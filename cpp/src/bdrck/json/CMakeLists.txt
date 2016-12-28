set(bdrck-json_SOURCES

	generate.cpp
	generate.hpp
	parse.cpp
	parse.hpp
	Types.cpp
	Types.hpp

)

add_library(bdrck-json STATIC ${bdrck-json_SOURCES})
target_link_libraries(bdrck-json

	bdrck-string
	bdrck-util
	${bdrck_LIBRARIES}

)

include(${PROJECT_SOURCE_DIR}/3rdparty/cmu/cmake/IdeSplitSources.cmake)
cmuIdeSplitSources(bdrck-json)
