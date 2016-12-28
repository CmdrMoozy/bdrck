#include "Windows.hpp"

#ifdef _WIN32

#include <codecvt>
#include <string>

#include <tchar.h>

namespace bdrck
{
namespace util
{
std::string wstrToStdString(std::wstring const &str)
{
	std::wstring_convert<std::codecvt_utf8<wchar_t>, wchar_t> converter;
	return converter.to_bytes(str);
}

std::string tstrToStdString(const LPTSTR str,
                            boost::optional<std::size_t> length)
{
	if(!length)
		length = _tcslen(str);
#ifdef UNICODE
	return wstrToStdString(std::wstring(str, str + length));
#else
	return std::string(str, str + *length);
#endif
}
}
}

#endif
