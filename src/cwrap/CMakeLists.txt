set(bdrck-cwrap_SOURCES

	String.cpp
	String.hpp
	Unistd.cpp
	Unistd.hpp

)

add_library(bdrck-cwrap STATIC ${bdrck-cwrap_SOURCES})
target_link_libraries(bdrck-cwrap bdrck-util)
