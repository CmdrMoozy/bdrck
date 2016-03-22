#ifndef bdrck_config_makeDefault_HPP
#define bdrck_config_makeDefault_HPP

#include <map>
#include <string>
#include <utility>
#include <vector>

#include <boost/variant/variant.hpp>

#include "bdrck/config/serialize.hpp"

namespace bdrck
{
namespace config
{
class Configuration;

namespace detail
{
typedef boost::variant<std::string, std::vector<std::string>>
        ConfigurationDefaultValue;
}

/*!
 * \brief This type provides a container for configuration default values.
 *
 * The exact type is to be considered an implementation detail. The notable
 * property of this type is simply that it can be defined using an
 * initializer list of ConfigurationDefaultsItems, as returned by
 * makeDefault.
 */
typedef std::map<std::string, detail::ConfigurationDefaultValue>
        ConfigurationDefaults;

/*!
 * \brief A particular item in a ConfigurationDetaults structure.
 */
typedef std::pair<std::string, detail::ConfigurationDefaultValue>
        ConfigurationDefaultsItem;

template <typename T>
ConfigurationDefaultsItem makeDefault(std::string const &key, T const &value)
{
	return std::make_pair(
	        key, detail::ConfigurationDefaultValue(serialize(value)));
}

template <typename T>
ConfigurationDefaultsItem makeDefault(std::string const &key,
                                      std::vector<T> const &values)
{
	std::vector<std::string> serializedValues;
	serializedValues.reserve(values.size());
	for(auto const &value : values)
		serializedValues.emplace_back(serialize(value));
	return std::make_pair(
	        key, detail::ConfigurationDefaultValue(serializedValues));
}

void setFromDefaultValue(Configuration &config,
                         ConfigurationDefaultsItem const &item);
}
}

#endif
