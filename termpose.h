#ifndef TERMPOSE
#define TERMPOSE


#include <stdint.h>
typedef struct Term Term;
typedef struct Term{
	uint8_t tag;
	uint32_t line;
	uint32_t column;
	uint32_t len;
	union { Term* con; char* str; };
} Term;

uint8_t isStri(Term* v); //tells you whether this is a string term(as opposed to a list term). If so, you can access the str member, knowing that it is a null terminated string of length len. If not, you can access the con member, knowing that it is an array of len subterms.
Term* newSeqs(Term* s, uint32_t nTerms, uint32_t line, uint32_t column);
Term* newStri(char* v, uint32_t length, uint32_t line, uint32_t column);
void drainTerm(Term* v);
void destroyTerm(Term* v);

Term* parseAsList(char const* unicodeString, char** errorOut); //contents will be contained in a root list even if there is only a single line at root (if you don't want that, see parse()). If there is an error, return value will be null and errorOut will be set to contain the text. If there was no error, errorOut will be set to null. errorOut can be null, if a report is not desired.

Term* parse(char const* unicodeString, char** errorOut); //same as above but root line will not be wrapped in a root seqs iff there is only one root element. This is what you want if you know there's only one line in the string you're parsing, but use parseAsList if there could be more(or less) than one, as it is more consistent.

char* stringifyTerm(Term* t);


/*TERMPOSE*/
#endif