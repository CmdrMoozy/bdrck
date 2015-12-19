#ifndef bdrck_util_ScopeExit_HPP
#define bdrck_util_ScopeExit_HPP

#include <functional>

namespace bdrck
{
namespace util
{
class ScopeExit
{
public:
	ScopeExit(std::function<void()> f);
	~ScopeExit();

private:
	std::function<void()> function;
};
}
}

#endif
