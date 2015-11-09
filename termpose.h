#ifndef TERMPOSE
#define TERMPOSE


#include <stdint.h>
typedef union Term Term;
typedef struct{
	uint8_t tag/*=0*/;
	uint32_t line;
	uint32_t column;
	uint32_t nTerms;
	Term* s;
} Seqs;
typedef struct{
	uint8_t tag/*=1*/;
	uint32_t line;
	uint32_t column;
	uint32_t length; //v is null terminated though.. not sure why I'm giving you this.. I guess it's just free space that might as well be put to the same use as in Seqs
	char* v;
} Stri;
typedef union Term {  Seqs seqs;  Stri stri;  } Term;
uint8_t isStri(Term* v);
Seqs* newSeqs(Term* s, uint32_t nTerms, uint32_t line, uint32_t column);
Stri* newStri(char* v, uint32_t length, uint32_t line, uint32_t column);
void drainTerm(Term* v);
void destroyTerm(Term* v);

Term* parseToSeqs(char const* unicodeString, char** errorOut); //contents will be contained in a root seqs even if there is only a single line at root. If there is an error, return value will be null and errorOut will be set to contain the text. If there was no error, errorOut will be set to null. errorOut can be null, if a report is not desired.

char* stringifyTerm(Term* t);


/*TERMPOSE*/
#endif