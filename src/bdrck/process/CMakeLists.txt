set(bdrck-process_SOURCES

	Pipe.cpp
	Pipe.hpp
	PipeCast.cpp
	PipeCast.hpp
	Process.cpp
	Process.hpp
	Terminal.cpp
	Terminal.hpp

)

add_library(bdrck-process STATIC ${bdrck-process_SOURCES})
target_link_libraries(bdrck-process

	bdrck-util
	${bdrck_LIBRARIES}

)
