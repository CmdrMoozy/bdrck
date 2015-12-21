set(bdrck-string_SOURCES

	StringRef.cpp
	StringRef.hpp

)

add_library(bdrck-string STATIC ${bdrck-string_SOURCES})
target_link_libraries(bdrck-string

	${bdrck_LIBRARIES}

)
