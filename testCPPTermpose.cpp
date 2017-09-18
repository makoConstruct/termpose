#include "termpose.cpp"
using namespace termpose;
#include <string>
#include <cassert>
#include <sstream>
using namespace std;

using namespace termpose::parsingDSL;

struct v2{
	float x;
	float y;
};

int main(){
	
	auto printBothWays = [](Term const& t){
		cout<<t.toString()<<endl;
		cout<<t.prettyPrint()<<endl;
	};
	
	string st("hogsmouth(heme heme)");
	Term t(Term::parse(st));
	printBothWays(t);
	string termposeString("\n\
memories\n\
	\"anthropolage\"\"\n\
		when everyone gets scattered\n\
	occulse\"\n\
		gleaming hero, shining light, greatest matron");
	Term btt = Term::parse(termposeString);
	printBothWays(btt);
	
	string multilineString("\n\
memori\n\
	bogbobog\n\
dabado\n\
	belebe\n\
		todoro\"mogodeseseseseseseseseseses somaro domese domadega damaramomo\n\
	medebe");
	t = Term::parse(multilineString);
	printBothWays(t);
	
	
	
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
	
	Term magag;
	magag = move(magog);
	
	cout<<"magag:"<<magag.initialString()<<" magog:"<<magog.initialString()<<endl;
	
	
	Term structure(Term::parse("\n\
application\n\
	knows_javascript true\n\
	knows_C++ true\n\
	passion\n\
		delusions messianic\n\
		legacy eternal"));
	
	assert(checkBool(structure.findSubTerm("knows_javascript")));
	
	Term& knowsCpp = structure.findTerm("knows_C++");
	assert(checkBool(knowsCpp.listContents().at(1)));
	
	Term* passionSpecification = structure.seekTerm("passion");
	if(!passionSpecification){
		cout<< "well that's alright ¯\\_(ツ)_/¯. You don't necessarily need that" <<endl;
	}else if(passionSpecification->seekTerm("delusions") != nullptr){
		cout<< "You're going to have to let go of those thoughts. Let us help you" <<endl;
	}else if(checkString(passionSpecification->findSubTerm("legacy")) == "eternal"){
		cout<< "your position is assured. You will be appointed as the final arbiter of style" <<endl;
	}else{
		cout<< "requires further processing." <<endl;
	}
	
	
	{
		Term a = "strterm";
		Term b = vector<Term>{"on", "bo", "den"};
		Term c = terms("on", "bo", terms("dell", "brook", "ringing glass"));
	}
	
	{
		Term a = Term::parse("a(b)(b)");
		cout<<"it'sa "<<a.toString()<<endl;
		assert(a.toString() == "((a b) b)");
	}
	
	{
		Term a = Term::parse("(a b):a");
		cout<<"it's "<<a.toString()<<endl;
		assert(a.toString() == "((a b) a)");
	}
	
	return 0;
}