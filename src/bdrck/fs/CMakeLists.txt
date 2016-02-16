set(bdrck-fs_SOURCES

	Iterator.cpp
	Iterator.hpp
	TemporaryStorage.cpp
	TemporaryStorage.hpp
	Util.cpp
	Util.hpp

)

add_library(bdrck-fs STATIC ${bdrck-fs_SOURCES})
target_link_libraries(bdrck-fs

	bdrck-algorithm
	bdrck-util
	${bdrck_LIBRARIES}

)
