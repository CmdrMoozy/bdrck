#include "Buffer.hpp"

#include <cassert>
#include <cstring>

namespace
{
char GIT_BUF_INIT_PTR[1] = {'\0'};
}

namespace bdrck
{
namespace git
{
Buffer::Buffer() : buffer({&GIT_BUF_INIT_PTR[0], 0, 0})
{
}

Buffer::~Buffer()
{
	assert(buffer.ptr != nullptr);
	git_buf_free(&buffer);
}

git_buf *Buffer::get()
{
	return &buffer;
}

git_buf const *Buffer::get() const
{
	return &buffer;
}

std::size_t Buffer::size() const
{
	return buffer.size;
}

std::size_t Buffer::allocatedSize() const
{
	return buffer.asize;
}

char *Buffer::begin()
{
	return buffer.ptr;
}

char const *Buffer::begin() const
{
	return buffer.ptr;
}

char *Buffer::end()
{
	return buffer.ptr + size();
}

char const *Buffer::end() const
{
	return buffer.ptr + size();
}

bool Buffer::containsNulByte() const
{
	return git_buf_contains_nul(&buffer) == 1;
}

bool Buffer::isBinary() const
{
	return git_buf_is_binary(&buffer) == 1;
}

bool Buffer::operator==(Buffer const &o) const
{
	assert(buffer.ptr != nullptr);
	assert(o.buffer.ptr != nullptr);
	if(buffer.size != o.buffer.size)
		return false;
	return std::memcmp(buffer.ptr, o.buffer.ptr, buffer.size) == 0;
}

bool Buffer::operator!=(Buffer const &o) const
{
	return !(*this == o);
}

bool Buffer::operator!() const
{
	assert(buffer.ptr != nullptr);
	return size() == 0 && allocatedSize() == 0;
}
}
}
