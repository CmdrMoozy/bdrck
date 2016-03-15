#ifndef bdrck_config_Serializer_HPP
#define bdrck_config_Serializer_HPP

#include <iomanip>
#include <sstream>
#include <string>
#include <type_traits>

namespace bdrck
{
namespace config
{
namespace detail
{
struct FloatSerializeImpl
{
	template <typename T> std::string operator()(T const &value) const
	{
		std::ostringstream oss;
		oss << std::fixed << value;
		return oss.str();
	}
};

struct GenericSerializeImpl
{
	template <typename T> std::string operator()(T const &value) const
	{
		std::ostringstream oss;
		oss << value;
		return oss.str();
	}
};
}

template <typename T> std::string serialize(T const &value)
{
	using ImplType = typename std::conditional<
	        std::is_floating_point<T>::value, detail::FloatSerializeImpl,
	        detail::GenericSerializeImpl>::type;
	ImplType impl;
	return impl(value);
}

std::string serialize(bool const &value);
}
}

#endif
