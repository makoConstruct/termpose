#include <stdlib.h>
#include <stdio.h>
#include <chrono>
#include <numeric>
#include <string.h>
#include "termpose.h"
#include "termpose.c"
#include <time.h>
#include <unistd.h>
#include <fcntl.h>
#include <sys/mman.h>



char const* readFile(char const* name){
	int fd = open(name, O_RDONLY);
	int len = lseek(fd, 0, SEEK_END);
	void *data = mmap(0, len, PROT_READ, MAP_PRIVATE, fd, 0);
	return (char const*)data;
}


int main(int argc, char** argv){
	char const* dat = readFile("longterm.term");
	TermposeParsingError err;
	unsigned n = 4000;
	unsigned total = 0;
	
	

	auto begin = std::chrono::high_resolution_clock::now();
	// logfile << s << "," <<  << std::endl;
	// struct timespec startTime, endTime;
	// clock_gettime(CLOCK_REALTIME, &startTime); //there's no CLOCK_REALTIME nor clock_gettime on my system? In time.h? It just isn't there?
	
	for(unsigned i=0; i<n; ++i){
		Term_t* result = parse(dat, &err);
		total += *(unsigned*)(void*)&result;
	}
	
	auto end = std::chrono::high_resolution_clock::now();
	// clock_gettime(CLOCK_REALTIME, &endTime);
	// struct timespec dif = diff(startTime, endTime);
	// <<":"<<<<endl;
	
	double perTest = (double)std::chrono::duration_cast<std::chrono::nanoseconds>(end - begin).count()/n;
	
	printf("the result was %u, the time taken was %f per test\n", total, perTest);
	
	return 0;
}