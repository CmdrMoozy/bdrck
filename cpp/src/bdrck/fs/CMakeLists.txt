set(bdrck-fs_SOURCES

	ExclusiveFileLock.cpp
	ExclusiveFileLock.hpp
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

include(${PROJECT_SOURCE_DIR}/3rdparty/cmu/cmake/IdeSplitSources.cmake)
cmuIdeSplitSources(bdrck-fs)
