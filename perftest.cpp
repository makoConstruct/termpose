#include "termpose.cpp"
using namespace termpose;
#include <string>
#include <unordered_map>
#include <cassert>
#include <sstream>
#include <chrono>
using namespace std;

using namespace termpose::parsingDSL;



Term makeTestTerm(unsigned nInts){
	vector<Term> out;
	out.reserve(nInts + 1);
	out.emplace_back("inside");
	string ss;
	for(uint i=0; i<nInts; ++i){
		string key("adb");
		key += to_string(i);
		out.emplace_back(terms(key, "hoble"));
	}
	return out;
}


vector<unsigned> fairlyRandomNumberSequence;
unsigned curi=0;
unsigned nextRand(){
	unsigned ret = fairlyRandomNumberSequence[curi];
	++curi;
	if(curi == fairlyRandomNumberSequence.size()){ curi = 0; }
	return ret;
}

unsigned acc=0;

void test(unsigned setSize, unsigned probes, unsigned repetitions){
	cout<< "setSize:"<<setSize<<" probes:"<<probes<<" repetitions:"<<repetitions<<endl;
	Term testTerm = makeTestTerm(setSize);
	
	
	auto start_time = std::chrono::high_resolution_clock::now();
	for(uint i=0; i<repetitions; ++i){
		for(uint j=0; j<probes; ++j){
			string key("adb");
			key += to_string(nextRand()%setSize);
			Term& v = testTerm.findTerm(key);
			if(v.isList()){ acc += 1; }
		}
	}
	auto current_time = std::chrono::high_resolution_clock::now();
	cout<< "  linear lookup took "<< std::chrono::duration_cast<std::chrono::milliseconds>(current_time - start_time).count() << " milliseconds" << std::endl;
	
	vector<pair<string, string>> pairs = sliceOffTag("inside", sequenceTrans(pairTrans(stringTrans(), stringTrans())))->check(testTerm);
	
	auto linear_pure_start_time = std::chrono::high_resolution_clock::now();
	for(uint i=0; i<repetitions; ++i){
		for(uint j=0; j<probes; ++j){
			string key("adb");
			key += to_string(nextRand()%setSize);
			for(uint k=0; k<pairs.size(); ++k){
				if(pairs[k].first == key){
					acc += 1;
					break;
				}
			}
		}
	}
	auto linear_pure_current_time = std::chrono::high_resolution_clock::now();
	cout<< "  linear pure lookup took "<< std::chrono::duration_cast<std::chrono::milliseconds>(linear_pure_current_time - linear_pure_start_time).count() << " milliseconds" << std::endl;
	
	
	unordered_map<string, string> mrep = sliceOffTag("inside", mapTrans(stringTrans(), stringTrans()))->check(testTerm);
	
	auto map_start_time = std::chrono::high_resolution_clock::now();
	for(uint i=0; i<repetitions; ++i){
		for(uint j=0; j<probes; ++j){
			string key("adb");
			key += to_string(nextRand()%setSize);
			string v = mrep.at(key);
			if(v.size() >= 1){ acc += 1; }
		}
	}
	auto map_current_time = std::chrono::high_resolution_clock::now();
	cout<< "  constant lookup took "<< std::chrono::duration_cast<std::chrono::milliseconds>(map_current_time - map_start_time).count() << " milliseconds" << std::endl;
	
}

int main(){
	
	fairlyRandomNumberSequence.push_back(9239);
	fairlyRandomNumberSequence.push_back(33841);
	fairlyRandomNumberSequence.push_back(79);
	for(uint i=0; i<1345; ++i){
		unsigned lll = fairlyRandomNumberSequence[fairlyRandomNumberSequence.size()-3];
		unsigned ll = fairlyRandomNumberSequence[fairlyRandomNumberSequence.size()-2];
		unsigned l = fairlyRandomNumberSequence[fairlyRandomNumberSequence.size()-1];
		fairlyRandomNumberSequence.push_back(lll ^ (ll*l + lll));
	}
	
	test(5, 100, 600);
	test(10, 100, 600);
	test(20, 100, 600);
	test(40, 100, 600);
	test(80, 100, 600);
	
	return 0;
}