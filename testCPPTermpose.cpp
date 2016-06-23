#include "termpose.cpp"
using namespace termpose;
#include <string>
using namespace std;

using namespace termpose::parsingDSL;

int main(){
	string st("hogsmouth(heme heme)");
	Term t(Term::parse(st));
	cout<<t.toString()<<endl;
	cout<<t.prettyPrint()<<endl;
	
	string bts("true false true false");
	Term boolTerm = Term::parse(bts);
	auto bools = sequence(boolTranslator())->check(boolTerm);
	unsigned counter = 0;
	for(bool b:bools){if(b) counter += 1;}
	cout<<"counter at "<<counter<<endl;
	return 0;
}