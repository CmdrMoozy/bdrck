#include "Util.hpp"

#include <cstdint>
#include <sstream>
#include <stdexcept>
#include <vector>

#include "bdrck/algorithm/String.hpp"

namespace bdrck
{
namespace config
{
SpecificFieldDescriptor
pathToDescriptor(std::string const &path,
                 google::protobuf::Message const &message)
{
	std::vector<std::string> components =
	        bdrck::algorithm::string::split(path, '.');

	google::protobuf::Message const *currentMessage = &message;
	for(auto it = components.begin(); it != components.end(); ++it)
	{
		auto next = it;
		++next;

		google::protobuf::FieldDescriptor const *fieldDescriptor =
		        currentMessage->GetDescriptor()->FindFieldByName(*it);
		if(fieldDescriptor == nullptr)
		{
			std::ostringstream oss;
			oss << "Invalid field path '" << *it << "'.";
			throw std::runtime_error(oss.str());
		}

		if(next == components.end())
			return std::make_pair(currentMessage, fieldDescriptor);

		currentMessage = &currentMessage->GetReflection()->GetMessage(
		        *currentMessage, fieldDescriptor);
	}

	return {nullptr, nullptr};
}

MutableSpecificFieldDescriptor
pathToMutableDescriptor(std::string const &path,
                        google::protobuf::Message &message)
{
	return {&message, pathToDescriptor(path, message).second};
}

std::string getFieldAsString(SpecificFieldDescriptor const &descriptor)
{
	if(descriptor.first == nullptr || descriptor.second == nullptr)
		return "";

	if(descriptor.second->is_repeated())
	{
		throw std::runtime_error(
		        "Can't convert repeated fields to string.");
	}

	switch(descriptor.second->cpp_type())
	{
	case google::protobuf::FieldDescriptor::CPPTYPE_INT32:
		return std::to_string(
		        descriptor.first->GetReflection()->GetInt32(
		                *descriptor.first, descriptor.second));
	case google::protobuf::FieldDescriptor::CPPTYPE_INT64:
		return std::to_string(
		        descriptor.first->GetReflection()->GetInt64(
		                *descriptor.first, descriptor.second));
	case google::protobuf::FieldDescriptor::CPPTYPE_UINT32:
		return std::to_string(
		        descriptor.first->GetReflection()->GetUInt32(
		                *descriptor.first, descriptor.second));
	case google::protobuf::FieldDescriptor::CPPTYPE_UINT64:
		return std::to_string(
		        descriptor.first->GetReflection()->GetUInt64(
		                *descriptor.first, descriptor.second));
	case google::protobuf::FieldDescriptor::CPPTYPE_DOUBLE:
		return std::to_string(
		        descriptor.first->GetReflection()->GetFloat(
		                *descriptor.first, descriptor.second));
	case google::protobuf::FieldDescriptor::CPPTYPE_FLOAT:
		return std::to_string(
		        descriptor.first->GetReflection()->GetDouble(
		                *descriptor.first, descriptor.second));
	case google::protobuf::FieldDescriptor::CPPTYPE_BOOL:
		return descriptor.first->GetReflection()->GetBool(
		               *descriptor.first, descriptor.second)
		               ? "true"
		               : "false";
	case google::protobuf::FieldDescriptor::CPPTYPE_ENUM:
		throw std::runtime_error(
		        "Can't convert enumeration fields to strings.");
	case google::protobuf::FieldDescriptor::CPPTYPE_STRING:
		return descriptor.first->GetReflection()->GetString(
		        *descriptor.first, descriptor.second);
	case google::protobuf::FieldDescriptor::CPPTYPE_MESSAGE:
		throw std::runtime_error(
		        "Can't convert message fields to strings.");
	}

	return "";
}

std::string getFieldAsString(std::string const &path,
                             google::protobuf::Message const &message)
{
	return getFieldAsString(pathToDescriptor(path, message));
}

void setFieldFromString(MutableSpecificFieldDescriptor const &descriptor,
                        std::string const &value)
{
	if(descriptor.first == nullptr || descriptor.second == nullptr)
		return;

	if(descriptor.second->is_repeated())
	{
		throw std::runtime_error(
		        "Can't set repeated fields from strings.");
	}

	switch(descriptor.second->cpp_type())
	{
	case google::protobuf::FieldDescriptor::CPPTYPE_INT32:
		descriptor.first->GetReflection()->SetInt32(
		        descriptor.first, descriptor.second,
		        static_cast<int32_t>(std::stoll(value)));
		break;
	case google::protobuf::FieldDescriptor::CPPTYPE_INT64:
		descriptor.first->GetReflection()->SetInt64(
		        descriptor.first, descriptor.second,
		        static_cast<int64_t>(std::stoll(value)));
		break;
	case google::protobuf::FieldDescriptor::CPPTYPE_UINT32:
		descriptor.first->GetReflection()->SetUInt32(
		        descriptor.first, descriptor.second,
		        static_cast<uint32_t>(std::stoull(value)));
		break;
	case google::protobuf::FieldDescriptor::CPPTYPE_UINT64:
		descriptor.first->GetReflection()->SetUInt64(
		        descriptor.first, descriptor.second,
		        static_cast<uint64_t>(std::stoull(value)));
		break;
	case google::protobuf::FieldDescriptor::CPPTYPE_DOUBLE:
		descriptor.first->GetReflection()->SetDouble(
		        descriptor.first, descriptor.second, std::stod(value));
		break;
	case google::protobuf::FieldDescriptor::CPPTYPE_FLOAT:
		descriptor.first->GetReflection()->SetFloat(
		        descriptor.first, descriptor.second, std::stof(value));
		break;
	case google::protobuf::FieldDescriptor::CPPTYPE_BOOL:
	{
		auto boolString = bdrck::algorithm::string::toLower(value);
		bdrck::algorithm::string::trim(boolString);
		if(boolString != "true" && boolString != "false")
		{
			std::ostringstream oss;
			oss << "Invalid boolean value string '" << value
			    << "'.";
			throw std::runtime_error(oss.str());
		}
		descriptor.first->GetReflection()->SetBool(
		        descriptor.first, descriptor.second,
		        boolString == "true");
		break;
	}
	case google::protobuf::FieldDescriptor::CPPTYPE_ENUM:
		throw std::runtime_error(
		        "Can't set enumeration fields from strings.");
	case google::protobuf::FieldDescriptor::CPPTYPE_STRING:
		descriptor.first->GetReflection()->SetString(
		        descriptor.first, descriptor.second, value);
		break;
	case google::protobuf::FieldDescriptor::CPPTYPE_MESSAGE:
		throw std::runtime_error(
		        "Can't set message fields from strings.");
	}
}

void setFieldFromString(std::string const &path,
                        google::protobuf::Message &message,
                        std::string const &value)
{
	setFieldFromString(pathToMutableDescriptor(path, message), value);
}
}
}
