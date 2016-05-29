set(bdrck-git_SOURCES

	Buffer.cpp
	Buffer.hpp
	checkReturn.cpp
	checkReturn.hpp
	Commit.cpp
	Commit.hpp
	Config.cpp
	Config.hpp
	Diff.cpp
	Diff.hpp
	Index.cpp
	Index.hpp
	Library.cpp
	Library.hpp
	Object.cpp
	Object.hpp
	Reference.cpp
	Reference.hpp
	Repository.cpp
	Repository.hpp
	Signature.cpp
	Signature.hpp
	StrArray.cpp
	StrArray.hpp
	Tree.cpp
	Tree.hpp
	Util.cpp
	Util.hpp
	Wrapper.hpp

)

add_library(bdrck-git STATIC ${bdrck-git_SOURCES})
target_link_libraries(bdrck-git

	bdrck-fs
	${bdrck_LIBRARIES}

)

include(${PROJECT_SOURCE_DIR}/3rdparty/cmu/cmake/IdeSplitSources.cmake)
cmuIdeSplitSources(bdrck-git)
