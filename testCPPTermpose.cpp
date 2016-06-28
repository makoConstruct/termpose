#include "termpose.cpp"
using namespace termpose;
#include <string>
#include <sstream>
using namespace std;

using namespace termpose::parsingDSL;

int main(){
	string st("hogsmouth(heme heme)");
	Term t(Term::parse(st));
	cout<<t.toString()<<endl;
	cout<<t.prettyPrint()<<endl;
	
	string termposeString("\n\
memories\n\
	\"anthropolage\"\"\n\
		when everyone gets scattered\n\
	occulse\"\n\
		gleaming hero, shining light, greatest matron");
	Term btt = Term::parse(termposeString);
	cout << termposeString <<endl;
	cout << btt.prettyPrint() <<endl;
	
	unordered_map<string,string> mems =
		mapConversion(taggedSequence(
			"memories",
			pairTrans(stringTrans(),stringTrans())
		))->check(Term::parse(termposeString));
	
	auto multiString = sliceOffTag("multiString", combine([](string s, int n){
		stringstream ss;
		for(int i=0; i<n; ++i){
			ss << s;
		}
		return ss.str();
	}, stringCheck(), intCheck()));
	
	auto multiStringChecker = sliceOffTag("multistrings", sequenceTrans(multiString));
	
	string mstrs("\
multistrings\n\
	multiString ba 90\n\
	multiString bo 90\n\
	multiString obab 45");
	Term mstt = Term::parse(mstrs);
	cout<<mstt.prettyPrint()<<endl;
	vector<string> multiStrings = multiStringChecker->check(mstt);
	for(auto s : multiStrings){
		cout<< s <<endl;
	}
	return 0;
}