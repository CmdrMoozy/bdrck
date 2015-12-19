set(bdrck-fs_SOURCES

	TemporaryStorage.cpp
	TemporaryStorage.hpp
	Util.cpp
	Util.hpp

)

add_library(bdrck-fs STATIC ${bdrck-fs_SOURCES})
target_link_libraries(bdrck-fs bdrck-algorithm bdrck-cwrap bdrck-util)
