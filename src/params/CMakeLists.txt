set(bdrck-params_SOURCES

	Argument.hpp
	Argument.cpp
	Command.cpp
	Command.hpp
	Option.cpp
	Option.hpp
	parseAndExecuteCommand.cpp
	parseAndExecuteCommand.hpp
	ProgramParameters.cpp
	ProgramParameters.hpp

	detail/parseArguments.cpp
	detail/parseArguments.hpp
	detail/parseCommand.cpp
	detail/parseCommand.hpp
	detail/parseOptions.cpp
	detail/parseOptions.hpp

)

add_library(bdrck-params STATIC ${bdrck-params_SOURCES})
