#include "Buffer.hpp"

#include <cstring>

namespace bdrck
{
namespace git
{
Buffer::Buffer() : buffer({nullptr, 0, 0})
{
}

Buffer::~Buffer()
{
	if(buffer.ptr != nullptr)
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

std::size_t Buffer::capacity() const
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
	return buffer.ptr == nullptr ? nullptr : buffer.ptr + size();
}

char const *Buffer::end() const
{
	return buffer.ptr == nullptr ? nullptr : buffer.ptr + size();
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
	if(buffer.ptr == nullptr && o.buffer.ptr == nullptr)
		return true;
	if(buffer.ptr == nullptr || o.buffer.ptr == nullptr)
		return false;
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
	return buffer.ptr == nullptr;
}
}
}
