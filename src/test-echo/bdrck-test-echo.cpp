#include <iostream>
#include <iterator>
#include <string>

#include "bdrck/params/Command.hpp"
#include "bdrck/params/Option.hpp"
#include "bdrck/params/parseAndExecute.hpp"

namespace
{
void echoStdin(bool out, bool err)
{
	std::istream_iterator<char> it(std::cin);
	std::istream_iterator<char> end;
	std::string input(it, end);

	if(out)
		std::cout << input;

	if(err)
		std::cerr << input;
}

const std::initializer_list<bdrck::params::Option> TEST_ECHO_COMMAND_OPTIONS{
        bdrck::params::Option::flag("stdout", "Echo stdin to stdout.", '1'),
        bdrck::params::Option::flag("stderr", "Echo stdin to stderr.", '2'),
        bdrck::params::Option::required("exitcode", "The exit code to return.",
                                        'e', "0")};
}

int main(int argc, char **argv)
{
	int exitCode = 0;

	bdrck::params::parseAndExecute(
	        argc, argv,
	        bdrck::params::Command(
	                "test-echo", "Echo stdin to stdout and/or stderr",
	                [&exitCode](bdrck::params::OptionsMap const &options,
	                            bdrck::params::FlagsMap const &flags,
	                            bdrck::params::ArgumentsMap const &)
	                {
		                exitCode = std::stoi(options.at("exitcode"));
		                echoStdin(flags.at("stdout"),
		                          flags.at("stderr"));
		        },
	                TEST_ECHO_COMMAND_OPTIONS));
	return exitCode;
}
