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
	
	string bts("\n\
memories\n\
	\"anthropolage\"\"\n\
		when everyone gets scattered\n\
	occulse\"\n\
		gleaming hero, shining light, greatest matron");
	Term btt = Term::parse(bts);
	cout << bts <<endl;
	cout << btt.prettyPrint() <<endl;
	
	auto memoryTrans = mapConversion(taggedListTrans("memories", pairTrans(stringTrans(),stringTrans())));
	unordered_map<string,string> mems = memoryTrans->check(btt);
	return 0;
}