set(bdrck-git_SOURCES

	checkReturn.cpp
	checkReturn.hpp
	Library.cpp
	Library.hpp
	Repository.cpp
	Repository.hpp
	Wrapper.hpp

)

add_library(bdrck-git STATIC ${bdrck-git_SOURCES})
target_link_libraries(bdrck-git

	bdrck-fs
	${bdrck_LIBRARIES}

)
