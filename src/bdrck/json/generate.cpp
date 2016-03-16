#include "generate.hpp"

#include <stdexcept>

#include <boost/variant/apply_visitor.hpp>
#include <boost/variant/static_visitor.hpp>

#include <yajl/yajl_gen.h>

#include "bdrck/util/ScopeExit.hpp"

namespace
{
struct GenerateContext
{
	std::ostream &out;
};

void printCallback(void *ctx, char const *str, unsigned int len)
{
	GenerateContext *context = static_cast<GenerateContext *>(ctx);
	context->out.write(str, static_cast<std::streamsize>(len));
}

class GenerateVisitor : public boost::static_visitor<yajl_gen_status>
{
public:
	GenerateVisitor(yajl_gen h) : handle(h)
	{
	}

	yajl_gen_status operator()(bdrck::json::NullType const &)
	{
		return yajl_gen_null(handle);
	}

	yajl_gen_status operator()(bdrck::json::BooleanType const &value)
	{
		return yajl_gen_bool(handle, value ? 1 : 0);
	}

	yajl_gen_status operator()(bdrck::json::IntegerType const &value)
	{
		return yajl_gen_integer(handle, static_cast<long int>(value));
	}

	yajl_gen_status operator()(bdrck::json::DoubleType const &value)
	{
		return yajl_gen_double(handle, value);
	}

	yajl_gen_status operator()(bdrck::json::StringType const &value)
	{
		return yajl_gen_string(
		        handle, reinterpret_cast<uint8_t const *>(value.data()),
		        value.size());
	}

	yajl_gen_status operator()(bdrck::json::MapType const &value)
	{
		yajl_gen_status ret = yajl_gen_map_open(handle);
		if(ret != yajl_gen_status_ok)
			return ret;

		for(auto const &pair : value)
		{
			ret = (*this)(pair.first);
			if(ret != yajl_gen_status_ok)
				return ret;

			ret = boost::apply_visitor(*this, pair.second);
			if(ret != yajl_gen_status_ok)
				return ret;
		}

		return yajl_gen_map_close(handle);
	}

	yajl_gen_status operator()(bdrck::json::ArrayType const &value)
	{
		yajl_gen_status ret = yajl_gen_array_open(handle);
		if(ret != yajl_gen_status_ok)
			return ret;

		for(auto const &v : value)
		{
			ret = boost::apply_visitor(*this, v);
			if(ret != yajl_gen_status_ok)
				return ret;
		}

		return yajl_gen_array_close(handle);
	}

private:
	yajl_gen handle;
};

template <typename... Arg> void configureGenerator(Arg... arg)
{
	int ret = yajl_gen_config(std::forward<Arg>(arg)...);
	if(ret == 0)
	{
		throw std::runtime_error("Configuring JSON generator failed.");
	}
}
}

namespace bdrck
{
namespace json
{
void generate(std::ostream &out, boost::optional<JsonValue> const &contents,
              bool beautify)
{
	GenerateContext context{out};

	yajl_gen generator = yajl_gen_alloc(nullptr);
	if(generator == nullptr)
	{
		throw std::runtime_error("Initializing JSON generator failed.");
	}
	bdrck::util::ScopeExit cleanup([&generator]()
	                               {
		                               yajl_gen_free(generator);
		                       });

	configureGenerator(generator, yajl_gen_beautify, beautify ? 1 : 0);
	configureGenerator(generator, yajl_gen_indent_string, "\t");
	configureGenerator(generator, yajl_gen_print_callback, printCallback,
	                   &context);

	if(!!contents)
	{
		GenerateVisitor visitor(generator);
		yajl_gen_status ret = boost::apply_visitor(visitor, *contents);

		if(ret != yajl_gen_status_ok &&
		   ret != yajl_gen_generation_complete)
		{
			throw std::runtime_error(
			        "Generating JSON output failed.");
		}
	}
}
}
}
