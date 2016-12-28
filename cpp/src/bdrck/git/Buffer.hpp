#ifndef bdrck_git_Buffer_HPP
#define bdrck_git_Buffer_HPP

#include <cstddef>

#include <git2.h>

namespace bdrck
{
namespace git
{
class Buffer
{
public:
	Buffer();

	Buffer(Buffer const &) = delete;
	Buffer(Buffer &&) = default;
	Buffer &operator=(Buffer const &) = delete;
	Buffer &operator=(Buffer &&) = default;

	~Buffer();

	git_buf *get();
	git_buf const *get() const;

	std::size_t size() const;
	std::size_t allocatedSize() const;

	char *begin();
	char const *begin() const;
	char *end();
	char const *end() const;

	bool containsNulByte() const;
	bool isBinary() const;

	bool operator==(Buffer const &o) const;
	bool operator!=(Buffer const &o) const;

	bool operator!() const;

private:
	git_buf buffer;
};
}
}

#endif
