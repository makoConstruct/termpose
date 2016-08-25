#ifndef TERMPOSE
#define TERMPOSE

#include <stdlib.h>
#include <assert.h>
#include <string.h>
#include <stdio.h>
#include <stdint.h>
#include <stdbool.h>
typedef struct Term_t{
	uint8_t tag;
	uint32_t line;
	uint32_t column;
	uint32_t len;
	union { struct Term_t* con; char* str; };
} Term_t;

typedef struct TermposeCoding{
	char openingBracket;
	char closingBracket;
	char pairingChar;
	uint8_t indentIsStrict;
	char* indent;
} TermposeCoding;
bool validateCoding(TermposeCoding const* coding);

typedef struct TermposeParsingError{
	uint32_t line;
	uint32_t column;
	const char* msg;
} TermposeParsingError;

uint8_t isStr(Term_t const* v); //tells you whether this is a string term(as opposed to a list term). If so, you can access the str member, knowing that it is a null terminated string of length len. If not, you can access the con member, knowing that it is an array of len subterms.
Term_t* newSeqs(Term_t* s, uint32_t nTerms, uint32_t line, uint32_t column);
Term_t* newStri(char* v, uint32_t length, uint32_t line, uint32_t column);
void drainTerm(Term_t* v);
void destroyTerm(Term_t* v); //destroy is just drain + free

Term_t* parseAsList(char const* unicodeString, TermposeParsingError* errorOut); //contents will be contained in a root list even if there is only a single line at root (if you don't want that, see parse()). If there is an error, return value will be null and errorOut.msg will be set. If there was no error, errorOut.msg will be set to null. errorOut can be null, if a report is not desired.
Term_t* parseAsListWithCoding(char const* unicodeString, TermposeParsingError* errorOut, TermposeCoding const* coding);

Term_t* parseLengthedToSeqs(char const * unicodeString, uint32_t length, TermposeParsingError* errorOut); //same as above but takes string with length
Term_t* parseLengthedToSeqsWithCoding(char const * unicodeString, uint32_t length, TermposeParsingError* errorOut, TermposeCoding const* coding);

Term_t* parse(char const* unicodeString, TermposeParsingError* errorOut); //same as above but root line will not be wrapped in a root seqs iff there is only one root element. This is what you want if you know there's only one line in the string you're parsing, but use parseAsList if there could be more(or less) than one, as it is more consistent.
Term_t* parseWithCoding(char const* unicodeString, TermposeParsingError* errorOut, TermposeCoding const* coding);
char* stringifyTerm(Term_t const* t);
char* stringifyTermWithCoding(Term_t const* t, TermposeCoding const* coding);

void destroyStr(char* str); //this needs to be defined in case linking libraries use a different allocator - their free() would not work properly. C users may usually ignore this.

/*TERMPOSE*/
#endif