#include "termpose.cpp"
#include <string>
#include <vector>
#include <iostream>
using namespace std;
using namespace termpose;

struct Product{
	string name;
	float cost;
	string description;
	Product(string name, float cost, string description):
		name(name), cost(cost), description(description)
	{}
};

string textData = "\
products\n\
	hammer cost:5 description\"\n\
		premium hammer. great for smashing\n\
	\"bee's knee\" cost:9.50 description\"\n\
		supposedly really good thing\n\
	twine cost:0 description\"\n\
		make a text adventure\n";

int main(int argc, char const *argv[]){
	
	//first, we parse the termpose data into a tree structure.
	Term data = Term::parse(textData);
	cout<< data.prettyPrint() <<endl;
	
	{
		using namespace termpose::parsingDSL;
		
		vector<Product> products = taggedSequence("products",
			combineCheck(
				[](string name, float cost, string description){
					return Product(name, cost, description); },
				stringCheck(),
				ensureTag("cost", floatCheck()),
				ensureTag("description", stringCheck()) )
		)->check(data);
		
		cout<< products[1].description <<endl;
	}
	
	return 0;
}