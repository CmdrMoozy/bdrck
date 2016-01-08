#include "Library.hpp"

#include <cassert>
#include <stdexcept>

#include <git2.h>

#include "bdrck/git/checkReturn.hpp"

std::mutex bdrck::git::Library::mutex;
std::unique_ptr<bdrck::git::Library> bdrck::git::Library::instance;

namespace bdrck
{
namespace git
{
LibraryInstance::LibraryInstance()
{
	std::lock_guard<std::mutex> lock(Library::mutex);
	if(!!Library::instance)
		throw std::runtime_error("Can't initialize libgit2 twice.");
	Library::instance.reset(new Library());
}

LibraryInstance::~LibraryInstance()
{
	std::lock_guard<std::mutex> lock(Library::mutex);
	assert(!!Library::instance);
	Library::instance.reset();
}

bool Library::isInitialized()
{
	std::lock_guard<std::mutex> lock(mutex);
	return !!instance;
}

Library::~Library()
{
	git_libgit2_shutdown();
}

Library::Library()
{
	checkReturn(git_libgit2_init());
}
}
}
