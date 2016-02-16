#include "parse.hpp"

#include <algorithm>
#include <cstddef>
#include <locale>
#include <stack>
#include <sstream>
#include <stdexcept>
#include <utility>
#include <type_traits>

#include <yajl/yajl_parse.h>

#include "bdrck/util/ScopeExit.hpp"

namespace
{
template <typename F, typename... Arg>
int executeCallback(F const &callback, Arg &&... arg)
{
	if(callback)
		return callback(std::forward<Arg>(arg)...) ? 1 : 0;
	return 0;
}

int nullCallback(void *ctx)
{
	auto callbacks = static_cast<bdrck::json::ParseCallbacks const *>(ctx);
	return executeCallback(callbacks->nullCallback);
}

int booleanCallback(void *ctx, int boolVal)
{
	auto callbacks = static_cast<bdrck::json::ParseCallbacks const *>(ctx);
	return executeCallback(callbacks->booleanCallback, boolVal != 0);
}

int integerCallback(void *ctx, long long integerVal)
{
	auto callbacks = static_cast<bdrck::json::ParseCallbacks const *>(ctx);
	return executeCallback(callbacks->integerCallback, integerVal);
}

int doubleCallback(void *ctx, double doubleVal)
{
	auto callbacks = static_cast<bdrck::json::ParseCallbacks const *>(ctx);
	return executeCallback(callbacks->doubleCallback, doubleVal);
}

int stringCallback(void *ctx, unsigned char const *stringVal,
                   std::size_t stringLen)
{
	auto callbacks = static_cast<bdrck::json::ParseCallbacks const *>(ctx);
	return executeCallback(
	        callbacks->stringCallback,
	        bdrck::json::StringType(stringVal, stringVal + stringLen));
}

int startMapCallback(void *ctx)
{
	auto callbacks = static_cast<bdrck::json::ParseCallbacks const *>(ctx);
	return executeCallback(callbacks->startMapCallback);
}

int mapKeyCallback(void *ctx, unsigned char const *key, std::size_t stringLen)
{
	auto callbacks = static_cast<bdrck::json::ParseCallbacks const *>(ctx);
	return executeCallback(callbacks->mapKeyCallback,
	                       bdrck::json::StringType(key, key + stringLen));
}

int endMapCallback(void *ctx)
{
	auto callbacks = static_cast<bdrck::json::ParseCallbacks const *>(ctx);
	return executeCallback(callbacks->endMapCallback);
}

int startArrayCallback(void *ctx)
{
	auto callbacks = static_cast<bdrck::json::ParseCallbacks const *>(ctx);
	return executeCallback(callbacks->startArrayCallback);
}

int endArrayCallback(void *ctx)
{
	auto callbacks = static_cast<bdrck::json::ParseCallbacks const *>(ctx);
	return executeCallback(callbacks->endArrayCallback);
}

constexpr yajl_callbacks CALLBACKS = {
        nullCallback,   booleanCallback,    integerCallback,  doubleCallback,
        nullptr,        stringCallback,     startMapCallback, mapKeyCallback,
        endMapCallback, startArrayCallback, endArrayCallback};

constexpr std::size_t BUFFER_SIZE = 4096;

void checkStatus(yajl_status status)
{
	switch(status)
	{
	case yajl_status_ok:
	case yajl_status_client_canceled:
		return;

	case yajl_status_error:
	{
		std::ostringstream oss;
		oss << "JSON parsing failed: "
		    << std::string(yajl_status_to_string(status));
		throw std::runtime_error(oss.str());
	}
	};
}

struct ParseAllContext
{
	boost::optional<bdrck::json::JsonValue> root;
	boost::optional<bdrck::json::StringType> key;
	std::stack<bdrck::json::JsonValue *> current;

	ParseAllContext() : root(boost::none), key(boost::none), current()
	{
	}
};

[[noreturn]] void throwMultipleValues()
{
	throw std::runtime_error(
	        "Found multiple values outside of a JSON container.");
}

[[noreturn]] void throwInvalidKey()
{
	throw std::runtime_error(
	        "Found JSON map key outside of map container.");
}

template <typename ValueType> struct IsJsonContainer
{
	typedef typename std::decay<ValueType>::type type;
	static constexpr bool value = std::conditional<
	        !std::is_same<type, bdrck::json::MapType>::value,
	        typename std::conditional<
	                !std::is_same<type, bdrck::json::ArrayType>::value,
	                std::false_type, std::true_type>::type,
	        std::true_type>::type::value;
};

template <typename ValueType>
class EmplaceValueVisitor
        : public boost::static_visitor<bdrck::json::JsonValue *>
{
private:
	ParseAllContext &context;
	ValueType const &value;

public:
	EmplaceValueVisitor(ParseAllContext &c, ValueType const &v)
	        : context(c), value(v)
	{
	}

	bdrck::json::JsonValue *operator()(bdrck::json::MapType &map)
	{
		if(!context.key)
		{
			throw std::runtime_error(
			        "Found JSON map value without a key.");
		}

		auto ptr = &map.emplace(*context.key, value).first->second;
		context.key = boost::none;
		return ptr;
	}

	bdrck::json::JsonValue *operator()(bdrck::json::ArrayType &array)
	{
		array.emplace_back(value);
		return &array.back();
	}

	[[noreturn]] bdrck::json::JsonValue *
	operator()(bdrck::json::NullType const &)
	{
		throwMultipleValues();
	}
	[[noreturn]] bdrck::json::JsonValue *
	operator()(bdrck::json::BooleanType const &)
	{
		throwMultipleValues();
	}
	[[noreturn]] bdrck::json::JsonValue *
	operator()(bdrck::json::IntegerType const &)
	{
		throwMultipleValues();
	}
	[[noreturn]] bdrck::json::JsonValue *
	operator()(bdrck::json::DoubleType const &)
	{
		throwMultipleValues();
	}
	[[noreturn]] bdrck::json::JsonValue *
	operator()(bdrck::json::StringType const &)
	{
		throwMultipleValues();
	}
};

struct NoopPushCurrent
{
	void operator()(ParseAllContext &, bdrck::json::JsonValue *)
	{
	}
};

struct PushCurrent
{
	void operator()(ParseAllContext &context, bdrck::json::JsonValue *value)
	{
		context.current.push(value);
	}
};

template <typename ValueType>
void emplaceCurrentValue(ParseAllContext &context, ValueType const &value)
{
	if(!context.root && context.current.empty())
	{
		// This is the very first value being parsed.
		context.root.emplace(value);
		context.current.push(&context.root.value());
	}
	else if(!context.current.empty())
	{
		// This is either a map value or an array value.
		EmplaceValueVisitor<ValueType> visitor(context, value);
		auto ptr =
		        boost::apply_visitor(visitor, *context.current.top());
		typename std::conditional<IsJsonContainer<ValueType>::value,
		                          PushCurrent, NoopPushCurrent>::type
		        maybePushCurrent{};
		maybePushCurrent(context, ptr);
	}
	else
	{
		throw std::runtime_error("Found multiple JSON values outside "
		                         "of a JSON container.");
	}
}

class EmplaceKeyVisitor : public boost::static_visitor<>
{
private:
	ParseAllContext &context;
	bdrck::json::StringType const &key;

public:
	EmplaceKeyVisitor(ParseAllContext &c, bdrck::json::StringType const &k)
	        : context(c), key(k)
	{
	}

	void operator()(bdrck::json::MapType const &)
	{
		if(!!context.key)
		{
			throw std::runtime_error(
			        "Found duplicate JSON map key.");
		}
		context.key.emplace(key);
	}

	[[noreturn]] void operator()(bdrck::json::ArrayType const &)
	{
		throwInvalidKey();
	}
	[[noreturn]] void operator()(bdrck::json::NullType const &)
	{
		throwInvalidKey();
	}
	[[noreturn]] void operator()(bdrck::json::BooleanType const &)
	{
		throwInvalidKey();
	}
	[[noreturn]] void operator()(bdrck::json::IntegerType const &)
	{
		throwInvalidKey();
	}
	[[noreturn]] void operator()(bdrck::json::DoubleType const &)
	{
		throwInvalidKey();
	}
	[[noreturn]] void operator()(bdrck::json::StringType const &)
	{
		throwInvalidKey();
	}
};

void emplaceCurrentKey(ParseAllContext &context,
                       bdrck::json::StringType const &key)
{
	if(context.current.empty())
		throwInvalidKey();
	EmplaceKeyVisitor visitor(context, key);
	boost::apply_visitor(visitor, *context.current.top());
}

void popCurrentValue(ParseAllContext &context)
{
	if(context.current.empty())
		throw std::runtime_error("Unexpected end of JSON container.");
	context.current.pop();
}
}

namespace bdrck
{
namespace json
{
void parse(std::istream &in, ParseCallbacks &callbacks)
{
	yajl_handle parser = yajl_alloc(&CALLBACKS, nullptr, &callbacks);
	if(parser == nullptr)
		throw std::runtime_error("Constructing JSON parser failed.");
	bdrck::util::ScopeExit cleanup([&parser]()
	                               {
		                               yajl_free(parser);
		                       });

	std::vector<char> buffer(BUFFER_SIZE, '\0');
	std::streamsize read = 0;
	bool foundNonWhitespace = false;

	while((read = in.readsome(buffer.data(), BUFFER_SIZE)) > 0)
	{
		char const *begin = buffer.data();
		char const *end = buffer.data() + read;
		if(!foundNonWhitespace)
		{
			std::locale locale;
			auto it = std::find_if(begin, end,
			                       [&locale](char const &c) -> bool
			                       {
				return !std::isspace(c, locale);
			});
			if(it != end)
			{
				begin = it;
				foundNonWhitespace = true;
			}
			else
			{
				continue;
			}
		}

		checkStatus(yajl_parse(
		        parser, reinterpret_cast<unsigned char const *>(begin),
		        static_cast<std::size_t>(end - begin)));
	}

	if(foundNonWhitespace)
		checkStatus(yajl_complete_parse(parser));
}

boost::optional<JsonValue> parseAll(std::istream &in)
{
	ParseAllContext context;
	ParseCallbacks callbacks;

	callbacks.nullCallback = [&context]() -> bool
	{
		emplaceCurrentValue(context, NullType());
		return true;
	};

	callbacks.booleanCallback = [&context](BooleanType value) -> bool
	{
		emplaceCurrentValue(context, value);
		return true;
	};

	callbacks.integerCallback = [&context](IntegerType value) -> bool
	{
		emplaceCurrentValue(context, value);
		return true;
	};

	callbacks.doubleCallback = [&context](DoubleType value) -> bool
	{
		emplaceCurrentValue(context, value);
		return true;
	};

	callbacks.stringCallback = [&context](StringType const &value) -> bool
	{
		emplaceCurrentValue(context, value);
		return true;
	};

	callbacks.startMapCallback = [&context]() -> bool
	{
		emplaceCurrentValue(context, MapType());
		return true;
	};

	callbacks.mapKeyCallback = [&context](StringType const &key) -> bool
	{
		emplaceCurrentKey(context, key);
		return true;
	};

	callbacks.endMapCallback = [&context]() -> bool
	{
		popCurrentValue(context);
		return true;
	};

	callbacks.startArrayCallback = [&context]() -> bool
	{
		emplaceCurrentValue(context, ArrayType());
		return true;
	};

	callbacks.endArrayCallback = [&context]() -> bool
	{
		popCurrentValue(context);
		return true;
	};

	parse(in, callbacks);
	return context.root;
}
}
}
