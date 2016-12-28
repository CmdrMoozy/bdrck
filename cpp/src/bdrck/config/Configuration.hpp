#ifndef bdrck_config_Configuration_HPP
#define bdrck_config_Configuration_HPP

#include <cassert>
#include <fstream>
#include <functional>
#include <map>
#include <memory>
#include <mutex>
#include <stdexcept>
#include <string>
#include <utility>

#include <boost/optional/optional.hpp>
#include <boost/signals2/connection.hpp>
#include <boost/signals2/signal.hpp>

#include <google/protobuf/message.h>

#include "bdrck/config/GenericConfiguration.hpp"

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

template <typename MessageType> class ConfigurationInstance
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
	 * \param defaults The default configuration values.
	 * \param customPath A custom configuration file path, if desired.
	 */
	ConfigurationInstance(
	        ConfigurationIdentifier const &id,
	        MessageType const &defaults = MessageType(),
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

template <typename MessageType> class Configuration
{
public:
	static Configuration &
	instance(ConfigurationIdentifier const &identifier);

	~Configuration() = default;

	MessageType const &get() const;
	void set(MessageType const &message);

	boost::signals2::scoped_connection handleConfigurationFieldChanged(
	        std::function<void(std::string const &)> const &slot);

	void resetAll();

private:
	friend class ConfigurationInstance<MessageType>;

	static std::mutex mutex;
	static std::map<ConfigurationIdentifier, std::unique_ptr<Configuration>>
	        instances;

	GenericConfiguration configuration;

	Configuration(std::string const &p,
	              MessageType const &d = MessageType());
};

namespace detail
{
std::string
getConfigurationPath(bdrck::config::ConfigurationIdentifier const &identifier);

template <typename MessageType>
std::unique_ptr<google::protobuf::Message>
parseConfiguration(std::string const &path)
{
	std::unique_ptr<google::protobuf::Message> message =
	        std::make_unique<MessageType>();
	std::ifstream in(path, std::ios_base::in | std::ios_base::binary);
	if(in.is_open())
		message->ParseFromIstream(&in);
	return message;
}
}

template <typename MessageType>
ConfigurationInstance<MessageType>::ConfigurationInstance(
        ConfigurationIdentifier const &id, MessageType const &defaults,
        boost::optional<std::string> const &customPath)
        : identifier(id)
{
	std::lock_guard<std::mutex> lock(Configuration<MessageType>::mutex);
	if(Configuration<MessageType>::instances.find(identifier) !=
	   Configuration<MessageType>::instances.end())
	{
		throw std::runtime_error("Can't initialize two Configuration "
		                         "instances with the same name.");
	}

	std::string path = !!customPath
	                           ? *customPath
	                           : detail::getConfigurationPath(identifier);
	Configuration<MessageType>::instances[identifier].reset(
	        new Configuration<MessageType>(path, defaults));
}

template <typename MessageType>
ConfigurationInstance<MessageType>::~ConfigurationInstance()
{
	std::lock_guard<std::mutex> lock(Configuration<MessageType>::mutex);
	auto it = Configuration<MessageType>::instances.find(identifier);
	assert(it != Configuration<MessageType>::instances.end());
	Configuration<MessageType>::instances.erase(it);
}

template <typename MessageType> std::mutex Configuration<MessageType>::mutex;

template <typename MessageType>
std::map<ConfigurationIdentifier, std::unique_ptr<Configuration<MessageType>>>
        Configuration<MessageType>::instances;

template <typename MessageType>
Configuration<MessageType> &
Configuration<MessageType>::instance(ConfigurationIdentifier const &identifier)
{
	std::lock_guard<std::mutex> lock(Configuration::mutex);
	auto it = instances.find(identifier);
	if(it == instances.end())
	{
		throw std::runtime_error("Can't access Configuration instances "
		                         "before construction.");
	}
	return *(it->second);
}

template <typename MessageType>
MessageType const &Configuration<MessageType>::get() const
{
	return dynamic_cast<MessageType const &>(configuration.getMessage());
}

template <typename MessageType>
void Configuration<MessageType>::set(MessageType const &message)
{
	configuration.setMessage(message);
}

template <typename MessageType>
boost::signals2::scoped_connection
Configuration<MessageType>::handleConfigurationFieldChanged(
        std::function<void(std::string const &)> const &slot)
{
	return configuration.handleConfigurationFieldChanged(slot);
}

template <typename MessageType> void Configuration<MessageType>::resetAll()
{
	configuration.resetAll();
}

template <typename MessageType>
Configuration<MessageType>::Configuration(std::string const &p,
                                          MessageType const &d)
        : configuration(p, d, detail::parseConfiguration<MessageType>(p))
{
}
}
}

#endif
