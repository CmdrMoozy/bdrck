#ifndef bdrck_git_Wrapper_HPP
#define bdrck_git_Wrapper_HPP

#include <memory>
#include <utility>

#include "bdrck/git/checkReturn.hpp"

namespace bdrck
{
namespace git
{
template <typename T, typename... Arg>
T *constructGitObject(int (*f)(T **, Arg...), Arg &&... arg)
{
	T *o = nullptr;
	checkReturn(f(&o, std::forward<Arg>(arg)...));
	return o;
}

template <typename T, void (*deleter)(T *)> class Wrapper
{
public:
	Wrapper(T *o) : object(o, deleter)
	{
	}

	template <typename... Arg>
	Wrapper(int (*f)(T **, Arg...), Arg &&... arg)
	        : Wrapper(constructGitObject(f, std::forward<Arg>(arg)...))
	{
	}

	Wrapper(Wrapper const &) = delete;
	Wrapper(Wrapper &&) = default;
	Wrapper &operator=(Wrapper const &) = delete;
	Wrapper &operator=(Wrapper &&) = default;

	~Wrapper() = default;

	T *get() const
	{
		return object.get();
	}

private:
	std::unique_ptr<T, void (*)(T *)> object;
};
}
}

#endif
