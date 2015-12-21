set(bdrck-process_SOURCES

	Process.cpp
	Process.hpp
	Terminal.cpp
	Terminal.hpp

)

add_library(bdrck-process STATIC ${bdrck-process_SOURCES})
target_link_libraries(bdrck-process

	bdrck-cwrap
	bdrck-util
	${bdrck_LIBRARIES}

)
