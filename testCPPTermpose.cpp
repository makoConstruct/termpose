#include "termpose.cpp"
using namespace termpose;
#include <string>
#include <sstream>
#include <cassert>
using namespace std;

using namespace termpose::parsingDSL;

struct v2{
	float x;
	float y;
};

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
	
	string mstrs("\
multistrings\n\
	multiString ba 90\n\
	multiString bo 90\n\
	multiString obab 45");
	Term mstt = Term::parse(mstrs);
	cout<<mstt.prettyPrint()<<endl;
	
	auto multiString = sliceOffTag("multiString", combineCheck([](string s, int n){
		stringstream ss;
		for(int i=0; i<n; ++i){
			ss << s;
		}
		return ss.str();
	}, stringCheck(), intCheck()));
	
	auto multiStringChecker = sliceOffTag("multistrings", sequenceTrans(multiString));
	
	vector<string> multiStrings = multiStringChecker->check(mstt);
	for(auto s : multiStrings){
		cout<< s <<endl;
	}
	
	auto vecTranslator = sliceOffTag("vec", combineTrans(
		[](float x, float y){ return v2{x,y}; },
		[](v2 v){ return make_tuple(v.x, v.y); },
		floatTrans(), floatTrans()));
	
	cout<< vecTranslator->termify(v2{1,1}).prettyPrint() <<endl;
	
	Term vecTerm = terms("vec","2","2");
	v2 ovec = vecTranslator->check(vecTerm);
	cout<<"vec("<<ovec.x<<" "<<ovec.y<<")"<<endl;
	
	
	Term magog = terms("magog", terms("isa", "spider"));
	assert(magog.startsWith("magog"));
	
	
	return 0;
}