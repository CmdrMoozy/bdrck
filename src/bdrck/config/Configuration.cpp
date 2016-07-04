#include "Configuration.hpp"

#include <vector>

#include "bdrck/fs/Util.hpp"

namespace bdrck
{
namespace config
{
namespace detail
{
std::string
getConfigurationPath(bdrck::config::ConfigurationIdentifier const &identifier)
{
	return bdrck::fs::combinePaths(bdrck::fs::getConfigurationDirectoryPath(
	                                       identifier.application),
	                               identifier.name + ".json");
}
}

int ConfigurationIdentifier::compare(ConfigurationIdentifier const &o) const
{
	int ret = application.compare(o.application);
	if(ret == 0)
		ret = name.compare(o.name);
	return ret;
}

bool ConfigurationIdentifier::operator==(ConfigurationIdentifier const &o) const
{
	return compare(o) == 0;
}

bool ConfigurationIdentifier::operator!=(ConfigurationIdentifier const &o) const
{
	return !(*this == o);
}

bool ConfigurationIdentifier::operator<(ConfigurationIdentifier const &o) const
{
	return compare(o) < 0;
}

bool ConfigurationIdentifier::operator<=(ConfigurationIdentifier const &o) const
{
	return compare(o) <= 0;
}

bool ConfigurationIdentifier::operator>(ConfigurationIdentifier const &o) const
{
	return compare(o) > 0;
}

bool ConfigurationIdentifier::operator>=(ConfigurationIdentifier const &o) const
{
	return compare(o) >= 0;
}
}
}
