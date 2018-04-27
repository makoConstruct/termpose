testCTermpose: testCTermpose.c termpose.c termpose.h
	gcc -O0 -g -std=c11 testCTermpose.c -o testCTermpose

testCPPTermpose: testCPPTermpose.cpp termpose.cpp termpose.c termpose.h
	g++ -O0 -g -std=c++11 testCPPTermpose.cpp -o testCPPTermpose

basicCppExample: basicCppExample.cpp termpose.cpp termpose.c termpose.h
	g++ -O0 -g -std=c++11 basicCppExample.cpp -o basicCppExample

perftest: perftest.cpp termpose.cpp termpose.c termpose.h
	g++ -std=c++11 perftest.cpp -o perftest