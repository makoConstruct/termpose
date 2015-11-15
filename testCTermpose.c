#include <stdlib.h>
#include <stdio.h>
#include <string.h>
#include "termpose.h"

// #define ANSI_COLOR_YELLOW  "\x1b[33m"
// #define ANSI_COLOR_BLUE    "\x1b[34m"
// #define ANSI_COLOR_MAGENTA "\x1b[35m"
// #define ANSI_COLOR_CYAN    "\x1b[36m"

#define COLOR 1
#ifdef COLOR
	#define ANSI_COLOR_RED     "\x1b[31m"
	#define ANSI_COLOR_GREEN   "\x1b[32m"
	#define ANSI_COLOR_RESET   "\x1b[0m"
#else
	#define ANSI_COLOR_RED ""
	#define ANSI_COLOR_GREEN ""
	#define ANSI_COLOR_RESET ""
#endif


void compare(Term* result, char* err, char const* output){
	if(result){
		char* strf = stringifyTerm(result);
		if(strcmp(strf, output) == 0){
			printf(ANSI_COLOR_GREEN "[good]" ANSI_COLOR_RESET " %s\n", strf);
		}else{
			printf(ANSI_COLOR_RESET "[fail]" ANSI_COLOR_RESET "expected %s received %s\n", output, strf);
		}
		free(strf);
		destroyTerm(result);
	}else{
		printf(ANSI_COLOR_RED "[fail]" ANSI_COLOR_RESET " %s\n", err);
		free(err);
	}
}

void testParseAsList(char const* input, char const* output){
	char* err;
	Term* result = parseAsList(input, &err);
	compare(result, err, output);
}

void testParse(char const* input, char const* output){
	char* err;
	Term* result = parse(input, &err);
	compare(result, err, output);
}


int main(int argc, char** argv){
	testParseAsList(
		"a\n"
		"  a\n"
		"  b c",
		"((a a (b c)))");
	testParseAsList(
		"a\n"
		"a\n"
		"b c",
		"(a a (b c))");
	testParseAsList(
		"a\n"
		" b:\n"
		"  c", "((a (b c)))");
	testParseAsList(
		"animol\n"
		"anmal\n"
		" oglomere:\n"
		"  hemisphere ok:\n"
		"   no more no\n"
		"  heronymous",
		"(animol (anmal (oglomere (hemisphere (ok (no more no))) heronymous)))");
	testParse("harry has a:larry nice", "(harry has (a larry) nice)");
	testParse("harry has a:larry nice\nhori ana he", "((harry has (a larry) nice) (hori ana he))");
}