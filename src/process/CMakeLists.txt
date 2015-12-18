set(bdrck-process_SOURCES

	Process.cpp
	Process.hpp
	Terminal.cpp
	Terminal.hpp

)

add_library(bdrck-process SHARED ${bdrck-process_SOURCES})
