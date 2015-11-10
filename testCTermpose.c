#include <stdlib.h>
#include <stdio.h>
#include <string.h>
#include "termpose.h"

// #define ANSI_COLOR_YELLOW  "\x1b[33m"
// #define ANSI_COLOR_BLUE    "\x1b[34m"
// #define ANSI_COLOR_MAGENTA "\x1b[35m"
// #define ANSI_COLOR_CYAN    "\x1b[36m"

#ifdef COLOR
	#define ANSI_COLOR_RED     "\x1b[31m"
	#define ANSI_COLOR_GREEN   "\x1b[32m"
	#define ANSI_COLOR_RESET   "\x1b[0m"
#else
	#define ANSI_COLOR_RED ""
	#define ANSI_COLOR_GREEN ""
	#define ANSI_COLOR_RESET ""
#endif

void test(char const* input, char const* output){
	char* err;
	Term* result = parseToSeqs(input, &err);
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

int main(int argc, char** argv){
	test(
		"a\n"
		"  a\n"
		"  b c",
		"((a a (b c)))");
	test(
		"a\n"
		"a\n"
		"b c",
		"(a a (b c))");
	test(
		"a\n"
		" b:\n"
		"  c", "((a (b c)))");
	test(
		"animol\n"
		"anmal\n"
		" oglomere:\n"
		"  hemisphere ok:\n"
		"   no more no\n"
		"  heronymous",
		"(animol (anmal (oglomere (hemisphere (ok (no more no))) heronymous)))");
	
}