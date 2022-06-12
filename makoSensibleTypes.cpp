#ifndef MAKO_SENSIBLE_TYPES
#define MAKO_SENSIBLE_TYPES

#include <stdint.h>

typedef uint8_t u8;
typedef uint16_t u16;
typedef uint32_t u32;
typedef uint64_t u64;

typedef int8_t i8;
typedef int16_t i16;
typedef int32_t i32;
typedef int64_t i64;

typedef unsigned int uint;


#include <sstream>
#include <string>

inline void strc_impl(std::stringstream& ss){}
template<typename... Params, typename T> inline void strc_impl(std::stringstream& ss, T first, Params... rest){
	ss << first;
	strc_impl(ss, rest...);
}
template<typename... R> std::string strc(R... r){
	std::stringstream ss;
	strc_impl(ss, r...);
	return ss.str();
}


//MAKO_SENSIBLE_TYPES:
#endif