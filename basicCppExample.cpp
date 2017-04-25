#include <string>
#include <cassert>
#include "termpose.cpp"
using namespace termpose;
using namespace termpose::parsingDSL;
using namespace std;


void processApplication(Term& v){
	assert(checkBool(v.findSubTerm("knows_javascript")));

	Term& knowsCpp = v.findTerm("knows_C++");
	assert(checkBool(knowsCpp.listContents().at(1)));

	Term* passionSpecification = v.seekTerm("passion");
	if(!passionSpecification){
		cout<< "well that's alright ¯\\_(ツ)_/¯. You don't necessarily need that" <<endl;
	}else{
		if(passionSpecification->seekTerm("delusions") != nullptr){
			cout<< "You're going to have to let go of those thoughts. Let us help you" <<endl;
		}else{
			if(checkString(passionSpecification->findSubTerm("legacy")) == "eternal"){
				cout<< "your position is assured. You will be appointed as an arbiter of style" <<endl;
			}else{
				cout<< "requires further processing." <<endl;
			}
		}
	}
}

int main(){
	Term firstApplicant(Term::parse("\n\
		applicant\n\
			knows_javascript true\n\
			knows_C++ true\n\
			passion\n\
				delusions messianic\n\
				legacy eternal"
	));
	
	Term secondApplicant = terms(
		"applicant",
			terms("knows_javascript", "true"),
			terms("knows_C++", "true")
	);
	
	processApplication(firstApplicant);
	processApplication(secondApplicant);
	
	return 0;
}