set(bdrck-git_SOURCES

	Buffer.cpp
	Buffer.hpp
	checkReturn.cpp
	checkReturn.hpp
	Diff.cpp
	Diff.hpp
	Library.cpp
	Library.hpp
	Object.cpp
	Object.hpp
	Repository.cpp
	Repository.hpp
	Tree.cpp
	Tree.hpp
	Wrapper.hpp

)

add_library(bdrck-git STATIC ${bdrck-git_SOURCES})
target_link_libraries(bdrck-git

	bdrck-fs
	${bdrck_LIBRARIES}

)

include(${PROJECT_SOURCE_DIR}/3rdparty/cmu/cmake/IdeSplitSources.cmake)
cmuIdeSplitSources(bdrck-git)
