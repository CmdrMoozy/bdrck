#ifndef bdrck_fs_Iterator_HPP
#define bdrck_fs_Iterator_HPP

#include <iterator>
#include <memory>
#include <string>
#include <experimental/optional>

namespace bdrck
{
namespace fs
{
namespace detail
{
struct IteratorImpl;
}
/*!
 * \brief A recursive iterator for filesystems.
 *
 * Iterates over a directory and all of its contents, recursively. Implements
 * the InputIterator concept, so although it can be copied when an instance of
 * this iterator is incremented it may invalidate all previous copies of the
 * iterator.
 *
 * The default constructor instantiates an "end" iterator.
 *
 * For more information on the semantics of this class, see:
 * http://en.cppreference.com/w/cpp/concept/InputIterator
 */
class Iterator : public std::iterator<std::input_iterator_tag, std::string>
{
public:
	Iterator();
	Iterator(std::string const &p, bool followSymlinks, bool sort);

	Iterator(Iterator const &) = default;
	Iterator(Iterator &&) = default;
	Iterator &operator=(Iterator const &) = default;
	Iterator &operator=(Iterator &&) = default;

	~Iterator() = default;

	bool operator==(Iterator const &o) const;
	bool operator!=(Iterator const &o) const;

	std::string const &operator*() const;
	std::string const *operator->() const;
	Iterator &operator++();

private:
	std::shared_ptr<detail::IteratorImpl> impl;
	std::experimental::optional<std::string> current;
};
}
}

#endif
