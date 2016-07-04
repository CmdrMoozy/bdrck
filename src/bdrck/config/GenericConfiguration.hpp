#ifndef bdrck_config_GenericConfiguration_HPP
#define bdrck_config_GenericConfiguration_HPP

#include <functional>
#include <memory>
#include <string>

#include <boost/signals2/connection.hpp>
#include <boost/signals2/signal.hpp>

#include <google/protobuf/message.h>

namespace bdrck
{
namespace config
{
class GenericConfiguration
{
public:
	/*!
	 * \param p The path to the serialized configuration file.
	 * \param d The default configuration message.
	 * \param c The current configuration message.
	 */
	GenericConfiguration(std::string const &p,
	                     google::protobuf::Message const &d,
	                     std::unique_ptr<google::protobuf::Message> &&c);
	~GenericConfiguration();

	google::protobuf::Message const &getMessage() const;
	void setMessage(google::protobuf::Message const &message);

	boost::signals2::scoped_connection handleConfigurationFieldChanged(
	        std::function<void(std::string const &)> const &slot);

	void resetAll();

private:
	std::string path;
	std::unique_ptr<google::protobuf::Message> defaults;
	std::unique_ptr<google::protobuf::Message> current;

	boost::signals2::signal<void(std::string const &)>
	        configurationFieldChangedSignal;

	void notifyListeners(google::protobuf::Message const *oldMessage,
	                     google::protobuf::Message const *newMessage);
};
}
}

#endif
