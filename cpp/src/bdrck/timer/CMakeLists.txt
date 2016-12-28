set(bdrck-timer_SOURCES

	TimerService.cpp
	TimerService.hpp
	TimerToken.cpp
	TimerToken.hpp

)

add_library(bdrck-timer STATIC ${bdrck-timer_SOURCES})
target_link_libraries(bdrck-timer

	${bdrck_LIBRARIES}

)

include(${PROJECT_SOURCE_DIR}/3rdparty/cmu/cmake/IdeSplitSources.cmake)
cmuIdeSplitSources(bdrck-timer)
