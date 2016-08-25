testCTermpose: testCTermpose.c termpose.c termpose.h
	gcc -std=c11 testCTermpose.c -o testCTermpose

testCPPTermpose: testCPPTermpose.cpp termpose.c termpose.h
	g++ -std=c++11 testCPPTermpose.cpp -o testCPPTermpose