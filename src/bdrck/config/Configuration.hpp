#ifndef bdrck_config_Configuration_HPP
#define bdrck_config_Configuration_HPP

#include <functional>
#include <map>
#include <memory>
#include <mutex>
#include <stdexcept>
#include <string>
#include <vector>

#include <boost/optional/optional.hpp>
#include <boost/signals2/connection.hpp>
#include <boost/signals2/signal.hpp>

#include "bdrck/config/ConfigurationDefaults.hpp"
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
	/*!
	 * Construct a new global configuration instance. The actual
	 * instance is later accessible via Configuration::instance().
	 *
	 * If no custom configuration file path is specified, then a proper
	 * operating-system-dependent path is used. Generally, the default
	 * (no custom path) is what is desired.
	 *
	 * \param id The identifier for this unique configuration instance.
	 * \param defaultValues The set of default configuration values.
	 * \param customPath A custom configuration file path, if desired.
	 */
	ConfigurationInstance(
	        ConfigurationIdentifier const &id,
	        ConfigurationDefaults const &defaultValues = {},
	        boost::optional<std::string> const &customPath = boost::none);

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

	boost::signals2::scoped_connection handleConfigurationChanged(
	        std::function<void(std::string const &)> const &slot);

	std::vector<std::string> getKeys() const;

	boost::optional<std::string> tryGet(std::string const &key) const;
	std::string get(std::string const &key,
	                boost::optional<std::string> const &defaultValue =
	                        boost::none) const;

	boost::optional<std::vector<std::string>>
	tryGetAll(std::string const &key) const;
	std::vector<std::string>
	getAll(std::string const &key,
	       boost::optional<std::vector<std::string>> const &defaultValues =
	               boost::none) const;

	template <typename T>
	boost::optional<T> tryGetAs(std::string const &key) const;
	template <typename T>
	T getAs(std::string const &key,
	        boost::optional<T> const &defaultValue = boost::none) const;
	template <typename T>
	boost::optional<std::vector<T>>
	tryGetAllAs(std::string const &key) const;
	template <typename T>
	std::vector<T>
	getAllAs(std::string const &key,
	         boost::optional<std::vector<T>> const &defaultValues =
	                 boost::none) const;

	bool empty() const;
	bool contains(std::string const &key) const;

	void set(std::string const &key, std::string const &value);
	void setAll(std::string const &key,
	            std::vector<std::string> const &values);

	template <typename T>
	void setFrom(std::string const &key, T const &value);
	template <typename T>
	void setAllFrom(std::string const &key, std::vector<T> const &values);

	void remove(std::string const &key);
	void clear();

	void reset(std::string const &key);
	void resetAll();

private:
	friend class ConfigurationInstance;

	static std::mutex mutex;
	static std::map<ConfigurationIdentifier, std::unique_ptr<Configuration>>
	        instances;

	boost::signals2::signal<void(std::string const &)>
	        configurationChangedSignal;

	ConfigurationDefaults defaults;
	std::string path;
	bdrck::json::MapType data;

	Configuration(ConfigurationIdentifier const &identifier,
	              ConfigurationDefaults const &defaultValues,
	              boost::optional<std::string> const &customPath);
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
boost::optional<std::vector<T>>
Configuration::tryGetAllAs(std::string const &key) const
{
	boost::optional<std::vector<std::string>> serialized = tryGetAll(key);
	if(!serialized)
		return boost::none;

	std::vector<T> values;
	values.reserve((*serialized).size());
	for(auto const &value : *serialized)
		values.emplace_back(deserialize<T>(value));
	return values;
}

template <typename T>
std::vector<T> Configuration::getAllAs(
        std::string const &key,
        boost::optional<std::vector<T>> const &defaultValues) const
{
	boost::optional<std::vector<T>> values = tryGetAllAs<T>(key);
	if(!values)
		values = defaultValues;
	if(!values)
	{
		throw std::runtime_error(
		        "Configuration key not found or deserializing failed.");
	}
	return *values;
}

template <typename T>
void Configuration::setFrom(std::string const &key, T const &value)
{
	set(key, serialize(value));
}

template <typename T>
void Configuration::setAllFrom(std::string const &key,
                               std::vector<T> const &values)
{
	std::vector<std::string> serializedValues;
	serializedValues.reserve(values.size());
	for(auto it = values.begin(); it != values.end(); ++it)
		serializedValues.emplace_back(serialize(*it));

	setAll(key, serializedValues);
}
}
}

#endif
