#ifndef bdrck_config_Configuration_HPP
#define bdrck_config_Configuration_HPP

#include <map>
#include <memory>
#include <mutex>
#include <stdexcept>
#include <string>

#include <boost/optional/optional.hpp>

#include "bdrck/config/deserialize.hpp"
#include "bdrck/config/serialize.hpp"
#include "bdrck/json/Types.hpp"

namespace bdrck
{
namespace config
{
struct ConfigurationIdentifier
{
	std::string application;
	std::string name;

	int compare(ConfigurationIdentifier const &o) const;
	bool operator==(ConfigurationIdentifier const &o) const;
	bool operator!=(ConfigurationIdentifier const &o) const;
	bool operator<(ConfigurationIdentifier const &o) const;
	bool operator<=(ConfigurationIdentifier const &o) const;
	bool operator>(ConfigurationIdentifier const &o) const;
	bool operator>=(ConfigurationIdentifier const &o) const;
};

class ConfigurationInstance
{
public:
	ConfigurationInstance(
	        ConfigurationIdentifier const &id,
	        std::map<std::string, std::string> const &defaultValues = {});

	ConfigurationInstance(ConfigurationInstance const &) = delete;
	ConfigurationInstance(ConfigurationInstance &&) = default;
	ConfigurationInstance &
	operator=(ConfigurationInstance const &) = delete;
	ConfigurationInstance &operator=(ConfigurationInstance &&) = default;

	~ConfigurationInstance();

private:
	const ConfigurationIdentifier identifier;
};

class Configuration
{
public:
	static Configuration &
	instance(ConfigurationIdentifier const &identifier);

	~Configuration();

	boost::optional<std::string> tryGet(std::string const &key) const;
	std::string get(std::string const &key,
	                boost::optional<std::string> const &defaultValue =
	                        boost::none) const;

	template <typename T>
	boost::optional<T> tryGetAs(std::string const &key) const;
	template <typename T>
	T getAs(std::string const &key,
	        boost::optional<T> const &defaultValue = boost::none) const;

	bool contains(std::string const &key) const;

	void set(std::string const &key, std::string const &value);
	template <typename T>
	void setFrom(std::string const &key, T const &value);

	void remove(std::string const &key);

	void reset(std::string const &key);
	void resetAll();

private:
	friend class ConfigurationInstance;

	static std::mutex mutex;
	static std::map<ConfigurationIdentifier, std::unique_ptr<Configuration>>
	        instances;

	std::map<std::string, std::string> defaults;
	bdrck::json::MapType data;

	Configuration(ConfigurationIdentifier const &identifier,
	              std::map<std::string, std::string> const &defaultValues);
};

template <typename T>
boost::optional<T> Configuration::tryGetAs(std::string const &key) const
{
	boost::optional<std::string> serialized = tryGet(key);
	if(!serialized)
		return boost::none;
	return deserialize<T>(*serialized);
}

template <typename T>
T Configuration::getAs(std::string const &key,
                       boost::optional<T> const &defaultValue) const
{
	boost::optional<T> value = tryGetAs<T>(key);
	if(!value)
		value = defaultValue;
	if(!value)
	{
		throw std::runtime_error(
		        "Configuration key not found or deserializing failed.");
	}
	return *value;
}

template <typename T>
void Configuration::setFrom(std::string const &key, T const &value)
{
	set(key, serialize(value));
}
}
}

#endif
