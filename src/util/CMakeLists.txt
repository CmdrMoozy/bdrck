set(bdrck-util_SOURCES

	Error.cpp
	Error.hpp
	ScopeExit.cpp
	ScopeExit.hpp

)

add_library(bdrck-util STATIC ${bdrck-util_SOURCES})
