#include "ConfigurationDefaults.hpp"

#include <boost/variant/apply_visitor.hpp>
#include <boost/variant/static_visitor.hpp>

#include "bdrck/config/Configuration.hpp"

namespace
{
class SetFromDefaultValueVisitor : public boost::static_visitor<>
{
public:
	SetFromDefaultValueVisitor(bdrck::config::Configuration &c,
	                           std::string const &k)
	        : configuration(c), key(k)
	{
	}

	void operator()(std::string const &value)
	{
		configuration.set(key, value);
	}

	void operator()(std::vector<std::string> const &values)
	{
		configuration.setAll(key, values);
	}

private:
	bdrck::config::Configuration &configuration;
	std::string key;
};
}

namespace bdrck
{
namespace config
{
void setFromDefaultValue(Configuration &config,
                         ConfigurationDefaultsItem const &item)
{
	SetFromDefaultValueVisitor visitor(config, item.first);
	boost::apply_visitor(visitor, item.second);
}
}
}
