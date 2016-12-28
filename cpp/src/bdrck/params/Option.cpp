#include "Option.hpp"

#include <algorithm>
#include <cassert>
#include <set>
#include <stdexcept>
#include <utility>

namespace
{
struct OptionNameComparator
{
	bool operator()(std::shared_ptr<bdrck::params::Option> a,
	                std::shared_ptr<bdrck::params::Option> b) const
	{
		return a->name < b->name;
	}
};

struct OptionShortNameComparator
{
	bool operator()(std::shared_ptr<bdrck::params::Option> a,
	                std::shared_ptr<bdrck::params::Option> b) const
	{
		return a->shortName < b->shortName;
	}
};
}

namespace bdrck
{
namespace params
{
Option Option::required(std::string const &n, std::string const &h,
                        boost::optional<char> const &sn,
                        boost::optional<std::string> const &dv)
{
	return Option(n, h, sn, dv, false, false);
}

Option Option::required(std::string const &n, std::string const &h,
                        boost::optional<char> const &sn, std::string const &dv)
{
	return Option(n, h, sn, dv, false, false);
}

Option Option::optional(std::string const &n, std::string const &h,
                        boost::optional<char> const &sn)
{
	return Option(n, h, sn, boost::none, true, false);
}

Option Option::flag(std::string const &n, std::string const &h,
                    boost::optional<char> const &sn)
{
	return Option(n, h, sn, boost::none, false, true);
}

Option::Option(std::string const &n, std::string const &h,
               boost::optional<char> const &sn,
               boost::optional<std::string> const &dv, bool o, bool f)
        : name(n),
          help(h),
          shortName(sn),
          defaultValue(dv),
          isOptional(o),
          isFlag(f)
{
	// Optionals and flags cannot have default values.
	assert(!(isOptional || isFlag) || !defaultValue);

	// The optional and flag parameters are mutually exclusive.
	assert(!isOptional || !isFlag);
}

Option::Option(std::string const &n)
        : Option(n, n, boost::none, boost::none, false, false)
{
}

OptionSetConstIterator::OptionSetConstIterator()
        : data(nullptr), length(0), current(0)
{
}

Option const &OptionSetConstIterator::operator*() const
{
	return *data[current];
}

OptionSetConstIterator &OptionSetConstIterator::operator++()
{
	current = std::min(current + 1, length);
	if(current == length)
	{
		data = nullptr;
		length = 0;
		current = 0;
	}
	return *this;
}

bool OptionSetConstIterator::operator==(OptionSetConstIterator const &o)
{
	return (data == o.data) && (length == o.length) &&
	       (current == o.current);
}

bool OptionSetConstIterator::operator!=(OptionSetConstIterator const &o)
{
	return !(*this == o);
}

OptionSetConstIterator::OptionSetConstIterator(
        std::vector<std::shared_ptr<Option>> const &v)
        : data(v.data()), length(v.size()), current(0)
{
}

struct OptionSet::OptionSetImpl
{
	std::vector<std::shared_ptr<Option>> unorderedOptions;
	std::set<std::shared_ptr<Option>, OptionNameComparator> nameOptions;
	std::set<std::shared_ptr<Option>, OptionShortNameComparator>
	        shortNameOptions;

	OptionSetImpl(std::initializer_list<Option> const &options)
	{
		for(auto const &option : options)
		{
			auto optionPtr = std::make_shared<Option>(option);
			unorderedOptions.push_back(optionPtr);
			nameOptions.insert(optionPtr);
			if(!!option.shortName)
				shortNameOptions.insert(optionPtr);
		}
	}

	OptionSetImpl(OptionSetImpl const &) = default;
	OptionSetImpl(OptionSetImpl &&) = default;
	OptionSetImpl &operator=(OptionSetImpl const &) = default;
	OptionSetImpl &operator=(OptionSetImpl &&) = default;

	~OptionSetImpl() = default;
};

OptionSet::OptionSet(std::initializer_list<Option> const &o)
        : impl(new OptionSetImpl(o))
{
}

OptionSet::OptionSet(OptionSet const &o)
{
	*this = o;
}

OptionSet::OptionSet(OptionSet &&o)
{
	impl = std::move(o.impl);
}

OptionSet &OptionSet::operator=(OptionSet const &o)
{
	if(this == &o)
		return *this;
	if(o.impl)
		impl.reset(new OptionSetImpl(*o.impl));
	else
		impl.reset();
	return *this;
}

OptionSet &OptionSet::operator=(OptionSet &&o)
{
	impl = std::move(o.impl);
	return *this;
}

OptionSet::~OptionSet()
{
}

std::size_t OptionSet::size() const
{
	return impl->unorderedOptions.size();
}

OptionSetConstIterator OptionSet::begin() const
{
	return OptionSetConstIterator(impl->unorderedOptions);
}

OptionSetConstIterator OptionSet::end() const
{
	return OptionSetConstIterator();
}

Option const *OptionSet::find(std::string const &parameter) const
{
	std::shared_ptr<Option> search(new Option(parameter));
	if(parameter.length() == 1)
		search->shortName = parameter[0];

	auto nameIt = impl->nameOptions.find(search);
	if(nameIt != impl->nameOptions.end())
		return &(**nameIt);

	if(!!search->shortName)
	{
		auto shortNameIt = impl->shortNameOptions.find(search);
		if(shortNameIt != impl->shortNameOptions.end())
			return &(**shortNameIt);
	}

	return nullptr;
}
}
}
