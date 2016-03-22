#include "Configuration.hpp"

#include <cassert>
#include <fstream>
#include <utility>

#include <boost/variant/get.hpp>

#include "bdrck/fs/Util.hpp"
#include "bdrck/json/generate.hpp"
#include "bdrck/json/parse.hpp"

namespace
{
std::string
getConfigurationPath(bdrck::config::ConfigurationIdentifier const &identifier)
{
	return bdrck::fs::combinePaths(bdrck::fs::getConfigurationDirectoryPath(
	                                       identifier.application),
	                               identifier.name + ".json");
}

bdrck::json::MapType parseConfiguration(std::string const &path)
{
	std::ifstream in(path, std::ios_base::in | std::ios_base::binary);
	if(!in.is_open())
		return bdrck::json::MapType();

	boost::optional<bdrck::json::JsonValue> parsed =
	        bdrck::json::parseAll(in);
	if(!parsed)
		return bdrck::json::MapType();

	return bdrck::json::get<bdrck::json::MapType>(*parsed);
}
}

namespace bdrck
{
namespace config
{
std::mutex Configuration::mutex;
std::map<ConfigurationIdentifier, std::unique_ptr<Configuration>>
        Configuration::instances;

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

ConfigurationInstance::ConfigurationInstance(
        ConfigurationIdentifier const &id,
        ConfigurationDefaults const &defaultValues,
        boost::optional<std::string> const &customPath)
        : identifier(id)
{
	std::lock_guard<std::mutex> lock(Configuration::mutex);
	if(Configuration::instances.find(identifier) !=
	   Configuration::instances.end())
	{
		throw std::runtime_error("Can't initialize two Configuration "
		                         "instances with the same name.");
	}

	Configuration::instances[identifier].reset(
	        new Configuration(identifier, defaultValues, customPath));
}

ConfigurationInstance::~ConfigurationInstance()
{
	std::lock_guard<std::mutex> lock(Configuration::mutex);
	auto it = Configuration::instances.find(identifier);
	assert(it != Configuration::instances.end());
	Configuration::instances.erase(it);
}

Configuration &
Configuration::instance(ConfigurationIdentifier const &identifier)
{
	std::lock_guard<std::mutex> lock(Configuration::mutex);
	auto it = Configuration::instances.find(identifier);
	if(it == Configuration::instances.end())
	{
		throw std::runtime_error("Can't access Configuration instances "
		                         "before construction.");
	}
	return *(it->second);
}

Configuration::~Configuration()
{
	std::ofstream out(path, std::ios_base::out | std::ios_base::binary |
	                                std::ios_base::trunc);
	if(out.is_open())
		bdrck::json::generate(out, bdrck::json::JsonValue(data),
		                      /*beautify=*/true);
}

boost::signals2::scoped_connection Configuration::handleConfigurationChanged(
        std::function<void(std::string const &)> const &slot)
{
	return boost::signals2::scoped_connection(
	        configurationChangedSignal.connect(slot));
}

std::vector<std::string> Configuration::getKeys() const
{
	std::vector<std::string> keys;
	keys.reserve(data.size());
	for(auto const &pair : data)
	{
		auto key = bdrck::json::toString(pair.first);
		keys.emplace_back(key.begin(), key.end());
	}
	return keys;
}

boost::optional<std::string> Configuration::tryGet(std::string const &key) const
{
	auto it = data.find(bdrck::json::fromString(key));
	if(it == data.end())
		return boost::none;
	bdrck::json::StringType const *value =
	        boost::get<bdrck::json::StringType>(&(it->second));
	if(value == nullptr)
		return boost::none;

	auto stringRef = bdrck::json::toString(*value);
	return std::string(stringRef.data(),
	                   stringRef.data() + stringRef.size());
}

std::string
Configuration::get(std::string const &key,
                   boost::optional<std::string> const &defaultValue) const
{
	boost::optional<std::string> value = tryGet(key);
	if(!value)
		value = defaultValue;
	if(!value)
		throw std::runtime_error("Configuration key not found.");
	return *value;
}

boost::optional<std::vector<std::string>>
Configuration::tryGetAll(std::string const &key) const
{
	auto it = data.find(bdrck::json::fromString(key));
	if(it == data.end())
		return boost::none;
	bdrck::json::ArrayType const *values =
	        boost::get<bdrck::json::ArrayType>(&(it->second));
	if(values == nullptr)
		return boost::none;

	std::vector<std::string> stringValues;
	for(auto const &value : *values)
	{
		bdrck::json::StringType const *stringValue =
		        boost::get<bdrck::json::StringType>(&value);
		if(stringValue == nullptr)
			continue;
		auto stringRef = bdrck::json::toString(*stringValue);

		stringValues.emplace_back(stringRef.data(),
		                          stringRef.data() + stringRef.size());
	}
	return stringValues;
}

std::vector<std::string> Configuration::getAll(
        std::string const &key,
        boost::optional<std::vector<std::string>> const &defaultValues) const
{
	boost::optional<std::vector<std::string>> values = tryGetAll(key);
	if(!values)
		values = defaultValues;
	if(!values)
		throw std::runtime_error("Configuration key not found.");
	return *values;
}

bool Configuration::empty() const
{
	return data.empty();
}

bool Configuration::contains(std::string const &key) const
{
	return !!tryGet(key);
}

void Configuration::set(std::string const &key, std::string const &value)
{
	auto jsonKey = bdrck::json::fromString(key);
	auto it = data.find(jsonKey);
	if(it == data.end())
	{
		data.insert(std::make_pair(jsonKey,
		                           bdrck::json::fromString(value)));
	}
	else
	{
		it->second = bdrck::json::fromString(value);
	}

	configurationChangedSignal(key);
}

void Configuration::setAll(std::string const &key,
                           std::vector<std::string> const &values)
{
	bdrck::json::ArrayType jsonValues;
	for(auto const &value : values)
		jsonValues.emplace_back(bdrck::json::fromString(value));

	auto jsonKey = bdrck::json::fromString(key);
	auto it = data.find(jsonKey);
	if(it == data.end())
		data.insert(std::make_pair(jsonKey, jsonValues));
	else
		it->second = jsonValues;
}

void Configuration::remove(std::string const &key)
{
	auto it = data.find(bdrck::json::fromString(key));
	if(it != data.end())
		data.erase(it);

	configurationChangedSignal(key);
}

void Configuration::clear()
{
	std::vector<std::string> keys = getKeys();
	data.clear();

	for(auto const &key : keys)
		configurationChangedSignal(key);
}

void Configuration::reset(std::string const &key)
{
	auto defaultIt = defaults.find(key);
	if(defaultIt == defaults.end())
		remove(key);
	else
		setFromDefaultValue(*this, *defaultIt);
}

void Configuration::resetAll()
{
	std::vector<std::string> keys = getKeys();

	data.clear();
	for(auto const &defaultValue : defaults)
		setFromDefaultValue(*this, defaultValue);

	for(auto const &key : keys)
		configurationChangedSignal(key);
}

Configuration::Configuration(ConfigurationIdentifier const &identifier,
                             ConfigurationDefaults const &defaultValues,
                             boost::optional<std::string> const &customPath)
        : configurationChangedSignal(),
          defaults(defaultValues),
          path(!!customPath ? *customPath : getConfigurationPath(identifier)),
          data(parseConfiguration(path))
{
	for(auto const &defaultValue : defaults)
	{
		if(data.find(bdrck::json::fromString(defaultValue.first)) ==
		   data.end())
		{
			setFromDefaultValue(*this, defaultValue);
		}
	}
}
}
}
