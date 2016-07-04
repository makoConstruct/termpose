##Intro to termpose in C++

###What is it?

The termpose language is a minimal, flexible, pretty way of expressing trees of strings. Barring platform constraints, it is much nicer than JSON as a substrate for configuration files and serializations.

The termpose API provides a set of functions for generating termpose parsers. It has some things in common with other parser generator DSLs, but it is simplified and streamlined by specializing on termpose trees.

With termpose, it is very easy to compose a strongly typed, bidirectional function for mapping from text to data, and from data to text, by combining simpler translation functions. I will start by showing an example of how you can compose a one directional function that converts termpose data to a vector of, I don't know, let's say product descriptions.

First, the data. An example of what termpose data can look like:

```
products
  hammer
    cost 5
    description "premium hammer. great for smashing"
  "bee's knee"
    cost 9.50
    description "supposedly really good thing"
  twine
    cost 0
    description "write a text adventure"
```

I think the structure of the tree is obvious, but if you're familiar with s-expressions, I can make this extra clear by adding the parens you would have had to type if you'd been using a format that couldn't understand the perfectly useful information conveyed by indentation:

```
(products
	(hammer (cost 5) (description "premium hammer. great for smashing"))
	("bee's knee" (cost 9.50) (description "supposedly really good thing"))
	(twine (cost 0) (description "write a text adventure")) )
```

I like s-expressions, honestly. It would be pretty easy to bolt a basic s-expression parser onto the termpose suite. Maybe I'll do that.

Anyway, I did say termpose is flexible. The very transparent representation I gave isn't the only way to write this data. I'd be more inclined to do it like this:

```
products
  hammer cost:5 description"
    premium hammer. great for smashing
  "bee's knee" cost:9.50 description"
    supposedly really good thing
  twine cost:0 description"
    write a text adventure
```

Termpose's multiline string syntax is pretty nuts. You just leave an open double quote at the end and indent, and when you unindent, the multiline string ends.

Now. Type checking and translation. The termpose API is inspired by scala Play's JSON read combinators. I don't know if there's anything equivalent in C++ for JSON, but there is for termpose, now, so why would you want to use json anyway. Look at this code:

```C++
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
	
	//parsing the termpose data into a Term.
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
```

But, why limit ourselves to a `Checker<vector<Product>>`, when we can have a `Translator<vector<Product>>` that goes both ways?

```C++
{
	using namespace termpose::parsingDSL;
	
	//constructing the 
	shared_ptr<Translator<vector<Product>>> productDataChecker = taggedSequence("products",
		combineTrans(
			[](string name, float cost, string description){
				return Product(name, cost, description); },
			[](Product p){
				return make_tuple(p.name, p.cost, p.description); },
			stringTrans(),
			ensureTag("cost", floatTrans()),
			ensureTag("description", stringTrans()) ) );
	
	vector<Product> products = productDataChecker->check(data);
	
	Term andBackAgain = productDataChecker->termify(products);
	
	cout<< andBackAgain.prettyPrint() <<endl;
}
```
