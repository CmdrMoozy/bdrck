#include "GenericConfiguration.hpp"

#include <fstream>
#include <utility>
#include <vector>

#include <google/protobuf/util/message_differencer.h>

namespace
{
void mergeDefaults(google::protobuf::Message &current,
                   google::protobuf::Message const &defaults)
{
	std::unique_ptr<google::protobuf::Message> defaultsCopy(defaults.New());
	defaultsCopy->CopyFrom(defaults);

	std::vector<google::protobuf::FieldDescriptor const *> defaultFields;
	defaultsCopy->GetReflection()->ListFields(*defaultsCopy,
	                                          &defaultFields);
	for(auto field : defaultFields)
	{
		current.GetReflection()->SwapFields(
		        &current, defaultsCopy.get(), {field});
	}
}

class ListenerReporter
        : public google::protobuf::util::MessageDifferencer::Reporter
{
public:
	ListenerReporter(
	        boost::signals2::signal<void(std::string const &)> const &s)
	        : signal(s)
	{
	}

	virtual ~ListenerReporter() = default;

	virtual void
	ReportAdded(google::protobuf::Message const &,
	            google::protobuf::Message const &,
	            std::vector<google::protobuf::util::MessageDifferencer::
	                                SpecificField> const &path)
	{
		report(path);
	}

	virtual void
	ReportDeleted(google::protobuf::Message const &,
	              google::protobuf::Message const &,
	              std::vector<google::protobuf::util::MessageDifferencer::
	                                  SpecificField> const &path)
	{
		report(path);
	}

	virtual void
	ReportModified(google::protobuf::Message const &,
	               google::protobuf::Message const &,
	               std::vector<google::protobuf::util::MessageDifferencer::
	                                   SpecificField> const &path)
	{
		report(path);
	}

private:
	boost::signals2::signal<void(std::string const &)> const &signal;

	void
	report(std::vector<
	        google::protobuf::util::MessageDifferencer::SpecificField> const
	               &path)
	{
		signal(path.front().field->name());
	}
};
}

namespace bdrck
{
namespace config
{
GenericConfiguration::GenericConfiguration(
        std::string const &p, google::protobuf::Message const &d,
        std::unique_ptr<google::protobuf::Message> &&c)
        : path(p),
          defaults(d.New()),
          current(std::move(c)),
          configurationFieldChangedSignal()
{
	defaults->CopyFrom(d);
	mergeDefaults(*current, *defaults);
}

GenericConfiguration::~GenericConfiguration()
{
	std::ofstream out(path, std::ios_base::out | std::ios_base::binary |
	                                std::ios_base::trunc);
	if(out.is_open())
		current->SerializeToOstream(&out);
}

google::protobuf::Message const &GenericConfiguration::getMessage() const
{
	return *current.get();
}

void GenericConfiguration::setMessage(google::protobuf::Message const &message)
{
	std::unique_ptr<google::protobuf::Message> oldMessage(
	        std::move(current));
	current.reset(message.New());
	current->CopyFrom(message);
	notifyListeners(oldMessage.get(), current.get());
}

boost::signals2::scoped_connection
GenericConfiguration::handleConfigurationFieldChanged(
        std::function<void(std::string const &)> const &slot)
{
	return boost::signals2::scoped_connection(
	        configurationFieldChangedSignal.connect(slot));
}

void GenericConfiguration::resetAll()
{
	std::unique_ptr<google::protobuf::Message> oldMessage(
	        std::move(current));
	current.reset(oldMessage->New());
	current->CopyFrom(*defaults);
	notifyListeners(oldMessage.get(), current.get());
}

void GenericConfiguration::notifyListeners(
        google::protobuf::Message const *oldMessage,
        google::protobuf::Message const *newMessage)
{
	ListenerReporter reporter(configurationFieldChangedSignal);
	google::protobuf::util::MessageDifferencer differencer;
	differencer.ReportDifferencesTo(&reporter);
	differencer.Compare(*oldMessage, *newMessage);
}
}
}
