#include "String.hpp"

#include <algorithm>
#include <iterator>
#include <locale>

namespace bdrck
{
namespace algorithm
{
namespace string
{
std::string toLower(const std::string &s)
{
	std::string ret(s);
	std::locale locale;
	std::transform(ret.begin(), ret.end(), ret.begin(), [&locale](char c)
	               {
		               return std::tolower(c, locale);
		       });
	return ret;
}

std::vector<std::string> split(const std::string &s, char d)
{
	std::vector<std::string> components;

	auto start = s.begin();
	auto end = std::find(s.begin(), s.end(), d);
	while(start != s.end())
	{
		if(start != end)
			components.push_back(std::string(start, end));

		start = end;
		if(start != s.end())
			++start;
		end = std::find(start, s.end(), d);
	}

	return components;
}

std::string &removeRepeatedCharacters(std::string &str, char character)
{
	bool repeatState = false;
	std::vector<char> copied;
	std::copy_if(str.begin(), str.end(), std::back_inserter(copied),
	             [&repeatState, character](char const &c) -> bool
	             {
		             if(c == character)
		             {
			             if(repeatState)
				             return false;
			             repeatState = true;
		             }
		             else
		             {
			             repeatState = false;
		             }

		             return true;
		     });
	str.assign(copied.data(), copied.size());
	return str;
}
}
}
}
