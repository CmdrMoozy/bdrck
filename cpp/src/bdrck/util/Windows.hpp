#ifdef _WIN32

#ifndef NOMINMAX
#define NOMINMAX
#endif

#ifndef WIN32_LEAN_AND_MEAN
#define WIN32_LEAN_AND_MEAN
#endif

#ifndef VC_EXTRALEAN
#define VC_EXTRALEAN
#endif

#include <cstddef>
#include <string>

#include <Windows.h>

#include <boost/optional/optional.hpp>

namespace bdrck
{
namespace util
{
std::string wstrToStdString(std::wstring const &str);
std::string tstrToStdString(const LPTSTR str,
                            boost::optional<std::size_t> length = boost::none);
}
}

#endif
