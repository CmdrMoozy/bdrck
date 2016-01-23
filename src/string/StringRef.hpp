#ifndef bdrck_string_StringRef_HPP
#define bdrck_string_StringRef_HPP

#include <algorithm>
#include <cstddef>
#include <cstring>
#include <iterator>
#include <memory>
#include <string>

namespace bdrck
{
namespace string
{
template <typename CharT, typename Traits = std::char_traits<CharT>>
class BasicStringRef
{
public:
	typedef Traits traits_type;
	typedef typename traits_type::char_type value_type;
	typedef typename std::size_t size_type;
	typedef typename std::allocator_traits<
	        std::allocator<CharT>>::difference_type difference_type;
	typedef value_type &reference;
	typedef value_type const &const_reference;
	typedef value_type *pointer;
	typedef value_type const *const_pointer;
	typedef const_pointer iterator;
	typedef const_pointer const_iterator;
	typedef typename std::reverse_iterator<iterator> reverse_iterator;
	typedef typename std::reverse_iterator<const_iterator>
	        const_reverse_iterator;

	BasicStringRef();
	BasicStringRef(CharT const *str);
	BasicStringRef(CharT const *str, size_type len);
	template <typename Allocator>
	BasicStringRef(std::basic_string<CharT, Traits, Allocator> const &str);

	BasicStringRef(BasicStringRef const &) = default;
	BasicStringRef(BasicStringRef &&) = default;
	BasicStringRef &operator=(BasicStringRef const &) = default;
	BasicStringRef &operator=(BasicStringRef &&) = default;

	~BasicStringRef() = default;

	int compare(BasicStringRef<CharT, Traits> const &o) const;

	bool operator==(BasicStringRef<CharT, Traits> const &o) const;
	bool operator!=(BasicStringRef<CharT, Traits> const &o) const;
	bool operator<(BasicStringRef<CharT, Traits> const &o) const;
	bool operator<=(BasicStringRef<CharT, Traits> const &o) const;
	bool operator>(BasicStringRef<CharT, Traits> const &o) const;
	bool operator>=(BasicStringRef<CharT, Traits> const &o) const;

	size_type size() const;
	size_type length() const;
	size_type max_size() const;
	bool empty() const;

	const_iterator begin() const;
	const_iterator cbegin() const;
	const_iterator end() const;
	const_iterator cend() const;
	const_reverse_iterator rbegin() const;
	const_reverse_iterator crbegin() const;
	const_reverse_iterator rend() const;
	const_reverse_iterator crend() const;

	value_type const &operator[](size_type pos) const;
	value_type const &at(size_type pos) const;
	value_type const &front() const;
	value_type const &back() const;
	const_pointer data() const;

	void clear();
	void remove_prefix(size_type n);
	void remove_suffix(size_type n);

private:
	const_pointer beginPtr;
	const_pointer endPtr;
};

template <typename CharT, typename Traits>
BasicStringRef<CharT, Traits>::BasicStringRef()
        : beginPtr(nullptr), endPtr(nullptr)
{
}

template <typename CharT, typename Traits>
BasicStringRef<CharT, Traits>::BasicStringRef(CharT const *str)
        : beginPtr(str), endPtr(str + traits_type::length(str))
{
	if(beginPtr == endPtr)
	{
		beginPtr = nullptr;
		endPtr = nullptr;
	}
}

template <typename CharT, typename Traits>
BasicStringRef<CharT, Traits>::BasicStringRef(CharT const *str, size_type len)
        : beginPtr(str), endPtr(str + len)
{
	if(beginPtr == endPtr)
	{
		beginPtr = nullptr;
		endPtr = nullptr;
	}
}

template <typename CharT, typename Traits>
template <typename Allocator>
BasicStringRef<CharT, Traits>::BasicStringRef(
        std::basic_string<CharT, Traits, Allocator> const &str)
        : beginPtr(str.data()), endPtr(str.data() + str.length())
{
	if(beginPtr == endPtr)
	{
		beginPtr = nullptr;
		endPtr = nullptr;
	}
}

template <typename CharT, typename Traits>
int BasicStringRef<CharT, Traits>::compare(
        BasicStringRef<CharT, Traits> const &o) const
{
	auto lhsSize = size();
	auto rhsSize = o.size();
	int ret = traits_type::compare(data(), o.data(),
	                               std::min(lhsSize, rhsSize));

	if(ret != 0)
		return ret;
	else if(lhsSize < rhsSize)
		return -1;
	else if(lhsSize > rhsSize)
		return 1;
	else
		return 0;
}

template <typename CharT, typename Traits>
bool BasicStringRef<CharT, Traits>::
operator==(BasicStringRef<CharT, Traits> const &o) const
{
	return compare(o) == 0;
}

template <typename CharT, typename Traits>
bool BasicStringRef<CharT, Traits>::
operator!=(BasicStringRef<CharT, Traits> const &o) const
{
	return compare(o) != 0;
}

template <typename CharT, typename Traits>
bool BasicStringRef<CharT, Traits>::
operator<(BasicStringRef<CharT, Traits> const &o) const
{
	return compare(o) < 0;
}

template <typename CharT, typename Traits>
bool BasicStringRef<CharT, Traits>::
operator<=(BasicStringRef<CharT, Traits> const &o) const
{
	return compare(o) <= 0;
}

template <typename CharT, typename Traits>
bool BasicStringRef<CharT, Traits>::
operator>(BasicStringRef<CharT, Traits> const &o) const
{
	return compare(o) > 0;
}

template <typename CharT, typename Traits>
bool BasicStringRef<CharT, Traits>::
operator>=(BasicStringRef<CharT, Traits> const &o) const
{
	return compare(o) >= 0;
}

template <typename CharT, typename Traits>
typename BasicStringRef<CharT, Traits>::size_type
BasicStringRef<CharT, Traits>::size() const
{
	return static_cast<size_type>(endPtr - beginPtr);
}

template <typename CharT, typename Traits>
typename BasicStringRef<CharT, Traits>::size_type
BasicStringRef<CharT, Traits>::length() const
{
	return size();
}

template <typename CharT, typename Traits>
typename BasicStringRef<CharT, Traits>::size_type
BasicStringRef<CharT, Traits>::max_size() const
{
	return size();
}

template <typename CharT, typename Traits>
bool BasicStringRef<CharT, Traits>::empty() const
{
	return beginPtr == endPtr;
}

template <typename CharT, typename Traits>
typename BasicStringRef<CharT, Traits>::const_iterator
BasicStringRef<CharT, Traits>::begin() const
{
	return beginPtr;
}

template <typename CharT, typename Traits>
typename BasicStringRef<CharT, Traits>::const_iterator
BasicStringRef<CharT, Traits>::cbegin() const
{
	return beginPtr;
}

template <typename CharT, typename Traits>
typename BasicStringRef<CharT, Traits>::const_iterator
BasicStringRef<CharT, Traits>::end() const
{
	return endPtr;
}

template <typename CharT, typename Traits>
typename BasicStringRef<CharT, Traits>::const_iterator
BasicStringRef<CharT, Traits>::cend() const
{
	return endPtr;
}

template <typename CharT, typename Traits>
typename BasicStringRef<CharT, Traits>::const_reverse_iterator
BasicStringRef<CharT, Traits>::rbegin() const
{
	return const_reverse_iterator(end());
}

template <typename CharT, typename Traits>
typename BasicStringRef<CharT, Traits>::const_reverse_iterator
BasicStringRef<CharT, Traits>::crbegin() const
{
	return const_reverse_iterator(end());
}

template <typename CharT, typename Traits>
typename BasicStringRef<CharT, Traits>::const_reverse_iterator
BasicStringRef<CharT, Traits>::rend() const
{
	return const_reverse_iterator(begin());
}

template <typename CharT, typename Traits>
typename BasicStringRef<CharT, Traits>::const_reverse_iterator
BasicStringRef<CharT, Traits>::crend() const
{
	return const_reverse_iterator(begin());
}

template <typename CharT, typename Traits>
typename BasicStringRef<CharT, Traits>::const_reference
        BasicStringRef<CharT, Traits>::
        operator[](size_type pos) const
{
	return *(beginPtr + pos);
}

template <typename CharT, typename Traits>
typename BasicStringRef<CharT, Traits>::const_reference
BasicStringRef<CharT, Traits>::at(size_type pos) const
{
	auto ptr = beginPtr + pos;
	if(ptr >= endPtr)
		throw std::out_of_range("Position is out of bounds.");
	return *ptr;
}

template <typename CharT, typename Traits>
typename BasicStringRef<CharT, Traits>::const_reference
BasicStringRef<CharT, Traits>::front() const
{
	return *begin();
}

template <typename CharT, typename Traits>
typename BasicStringRef<CharT, Traits>::const_reference
BasicStringRef<CharT, Traits>::back() const
{
	return *rbegin();
}

template <typename CharT, typename Traits>
typename BasicStringRef<CharT, Traits>::const_pointer
BasicStringRef<CharT, Traits>::data() const
{
	return beginPtr;
}

template <typename CharT, typename Traits>
void BasicStringRef<CharT, Traits>::clear()
{
	beginPtr = nullptr;
	endPtr = nullptr;
}

template <typename CharT, typename Traits>
void BasicStringRef<CharT, Traits>::remove_prefix(size_type n)
{
	auto newBeginPtr = beginPtr + n;
	if(newBeginPtr >= endPtr)
	{
		beginPtr = nullptr;
		endPtr = nullptr;
	}
	else
	{
		beginPtr = newBeginPtr;
	}
}

template <typename CharT, typename Traits>
void BasicStringRef<CharT, Traits>::remove_suffix(size_type n)
{
	auto newEndPtr = endPtr - n;
	if(newEndPtr <= beginPtr)
	{
		beginPtr = nullptr;
		endPtr = nullptr;
	}
	else
	{
		endPtr = newEndPtr;
	}
}

typedef BasicStringRef<char> StringRef;
typedef BasicStringRef<wchar_t> WStringRef;
typedef BasicStringRef<char16_t> U16StringRef;
typedef BasicStringRef<char32_t> U32StringRef;
}
}

#endif
