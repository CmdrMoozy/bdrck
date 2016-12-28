#ifndef bdrck_config_Util_HPP
#define bdrck_config_Util_HPP

#include <string>
#include <utility>

#include <google/protobuf/descriptor.h>
#include <google/protobuf/message.h>

namespace bdrck
{
namespace config
{
bool messagesAreEqual(google::protobuf::Message const &a,
                      google::protobuf::Message const &b);

typedef std::pair<google::protobuf::Message const *,
                  google::protobuf::FieldDescriptor const *>
        SpecificFieldDescriptor;
typedef std::pair<google::protobuf::Message *,
                  google::protobuf::FieldDescriptor const *>
        MutableSpecificFieldDescriptor;

SpecificFieldDescriptor
pathToDescriptor(std::string const &path,
                 google::protobuf::Message const &message);
MutableSpecificFieldDescriptor
pathToMutableDescriptor(std::string const &path,
                        google::protobuf::Message &message);

std::string getFieldAsString(SpecificFieldDescriptor const &descriptor);
std::string getFieldAsString(std::string const &path,
                             google::protobuf::Message const &message);
void setFieldFromString(MutableSpecificFieldDescriptor const &descriptor,
                        std::string const &value);
void setFieldFromString(std::string const &path,
                        google::protobuf::Message &message,
                        std::string const &value);
}
}

#endif
