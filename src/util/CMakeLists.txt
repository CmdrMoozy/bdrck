set(bdrck-util_SOURCES

	Error.cpp
	Error.hpp
	ScopeExit.cpp
	ScopeExit.hpp
	UUID.cpp
	UUID.hpp

)

add_library(bdrck-util STATIC ${bdrck-util_SOURCES})
