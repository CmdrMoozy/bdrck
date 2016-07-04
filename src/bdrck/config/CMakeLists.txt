set(bdrck-config_SOURCES

	Configuration.cpp
	Configuration.hpp
	GenericConfiguration.cpp
	GenericConfiguration.hpp

)

add_library(bdrck-config STATIC ${bdrck-config_SOURCES})
target_link_libraries(bdrck-config

	bdrck-fs
	${bdrck_LIBRARIES}

)

include(${PROJECT_SOURCE_DIR}/3rdparty/cmu/cmake/IdeSplitSources.cmake)
cmuIdeSplitSources(bdrck-config)
