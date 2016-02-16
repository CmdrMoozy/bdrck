set(bdrck-algorithm_SOURCES

	String.cpp
	String.hpp

)

add_library(bdrck-algorithm STATIC ${bdrck-algorithm_SOURCES})
target_link_libraries(bdrck-algorithm

	${bdrck_LIBRARIES}

)
