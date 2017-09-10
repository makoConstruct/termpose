
### Parsing
```C++
Term Term::parse(string const& str);
Term Term::parseMultiline(string const& str);
```
Where a root level term is a term that starts at indent_level:0, `parseMultiline` will always return a list of the root level terms, no matter how many there are, while `parse` is a bit inconsistent in some cases. If there is only one root level term, it will return that directly. If there are multiple, it will bundle them all in an extra list. If there is no root level term, it will return an empty list.

### Building terms in code
By example
```C++
Term a = "strterm";
Term b = vector<Term>{"on", "bo", "den"};
Term c = terms("on", "bo", terms("dell", "brook", "ringing glass"));
```

### Basic Term API

```C++
bool Term::isList() const;
bool Term::isStr() const;
```

```C++
vector<Term>& Term::listContents();
vector<Term> const& Term::listContentsConst() const;
string& Term::stringContents();
string const& Term::stringContentsConst() const;
```

listContents throws a TyperError if the term is not a list,
stringContents throws a TyperError if the term is not a str,

```C++
bool Term::isEmpty() const;
```

Returns true iff the term is an empty list




#### Searching by key

It is often more efficient to do a linear search over a list than it is to generate a hashmap over the list's keys and values. Even just querying that hashmap can take longer than a linear search in many cases. For lists of fewer than 20 elements*, using the following methods will be more efficient than creating a mapping.

 (cases of lists with fewer than )

```C++
Term& Term::findTerm(string const& key);
Term const& Term::findTermConst(string const& key) const;
```

Finds a term within `this` that is a list that starts with `key`. Throws a TyperError if there is no such term, or if `this` is not a list.

```C++
Term* Term::seekTerm(string const& key);
Term const* Term::seekTermConst(string const& key) const;
```

Seeks a term within `this` that is a list that starts with `key`. Returns null if there is no such term. Throws a TyperError if `this` is not a list.



```C++
Term& Term::findSubTerm(string const& key);
Term const& Term::findSubTermConst(string const& key) const;
```

Finds a list of two elements within `this` that starts with `key`, then returns the second element. Throws a TyperError if there is no such term, or if `this` is not a list.

```C++
Term* Term::seekSubTerm(string const& key);
Term const* Term::seekSubTermConst(string const& key) const;
```

Seeks a list of two elements within `this` that starts with `key`, then returns the second element. Returns null if there is no such term. Throws a TyperError if `this` is not a list.



### Type Checking

```C++
float checkFloat(Term const& v);
Term termifyFloat(float const& v);

double checkDouble(Term const& v);
Term termifyDouble(double const& v);

int checkInt(Term const& v);
Term termifyInt(int const& v);

string checkString(Term const& v);
Term termifyString(string const& v);

bool checkBool(Term const& v);
Term termifyBool(bool const& v);
```

You can also compose bidirectional functions for checking or termifying complex structures

```C++
Term term;

rc<Translator<unordered_map<string, vector<bool>>>> tr = mapTrans(stringTrans(), sequenceTrans(boolTrans()));

unordered_map<string, vector<bool>>> r = tr->check(term);
cout<< tr->termify(r).prettyPrint() <<endl;
```

One of the advantages of using a `Translator`; if there are type errors, a `Translator` will find all of them and it will list their line and column number in the `TyperError` it will throw. For comparable code you would be likely to write using check and termify methods, it is likely that you will only find the first error, and then throw there.


### Example

```C++
#include <string>
#include <cassert>
#include "termpose.cpp"
using namespace termpose::parsingDSL;
using namespace std;

void processApplication(Term const& v){
	assert(checkBool(v.findSubTerm("knows_javascript")));

	Term& knowsCpp = v.findTerm("knows_C++");
	assert(checkBool(knowsCpp.listContents().at(1)));

	Term* passionSpecification = v.seekTerm("passion");
	if(!passionSpecification){
		cout<< "well that's alright ¯\\_(ツ)_/¯. You don't necessarily need that" <<endl;
	}else if(passionSpecification->seekTerm("delusions") != nullptr){
			cout<< "You're going to have to let go of those thoughts. Let us help you" <<endl;
	}else if(checkString(passionSpecification->findSubTerm("legacy")) == "eternal"){
		cout<< "your position is assured. You will be appointed as an arbiter of style" <<endl;
	}else{
		cout<< "requires further processing." <<endl;
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

```

