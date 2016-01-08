#define CATCH_CONFIG_RUNNER
#include <catch/catch.hpp>

#include "bdrck/git/Library.hpp"

int main(int argc, char **argv)
{
	bdrck::git::LibraryInstance gitLibrary;

	return Catch::Session().run(argc, argv);
}
