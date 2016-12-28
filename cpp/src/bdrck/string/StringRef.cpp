#include "StringRef.hpp"

namespace bdrck
{
namespace string
{
template class BasicStringRef<char>;
template class BasicStringRef<wchar_t>;
template class BasicStringRef<char16_t>;
template class BasicStringRef<char32_t>;
}
}
