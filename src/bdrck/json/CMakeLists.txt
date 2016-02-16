set(bdrck-json_SOURCES

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
