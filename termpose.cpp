// 
#include <iostream>
#include <sstream>
#include <string>
#include <stdexcept>
#include <algorithm>
#include <vector>
#include <unordered_map>
#include <cstring>
#include <sstream>
#include <type_traits>
#include <memory>

#include "termpose.c"

namespace termpose{


namespace detail{
	template <typename T>
	std::vector<T> operator+(const std::vector<T> &A, const std::vector<T> &B)
	{
		std::vector<T> AB;
		AB.reserve( A.size() + B.size() );
		AB.insert( AB.end(), A.begin(), A.end() );
		AB.insert( AB.end(), B.begin(), B.end() );
		return AB;
	}

	//could probably be optimized for moves:
	template <typename T>
	void append(std::vector<T> &A, std::vector<T> const&B){
		A.reserve( A.size() + B.size() );
		A.insert( A.end(), B.begin(), B.end() );
	}

	// hhhhhhhhhhhhhhhhhhhhhhhhhhhhhhhhhhhhhhhhhhhhhhhhhhhhhhhhhhhhhhhhhhhhhhhh  
	template<typename T, typename B> std::vector<B> map(std::vector<T>& v, std::function<B (T)> f){
		unsigned ei = v.size();
		std::vector<B> ret(ei);
		for(int i=0; i<ei; ++i){
			ret[i] = f(v[i]);
		}
		return ret;
	}
}


struct Error{
	unsigned line;
	unsigned column;
	std::string msg;
	Error(unsigned line, unsigned column, std::string msg): line(line), column(column), msg(msg) {}
};
struct TermposeError:public std::runtime_error{
	TermposeError(std::string msg):std::runtime_error(msg){}
	static void putErr(std::stringstream& ss, Error const& e){
		ss<<"  error line:";
		ss<<e.line;
		ss<<" column:";
		ss<<e.column;
		ss<<' ';
		ss<<" msg:";
		ss<<e.msg;
		ss<<'\n';
	}
};
struct ParserError:public TermposeError{
public:
	Error e;
	ParserError(unsigned line, unsigned column, std::string msg):e{line, column, msg}, TermposeError("ParserError") {}
	virtual char const* what(){
		std::stringstream ss;
		ss<<"ParserError"<<'\n';
		putErr(ss, e);
		return ss.str().c_str();
	}
};
union Term;

bool escapeIsNeeded(std::string const& str){
	for(int i=0; i<str.size(); ++i){
		switch(str[i]){
			case ' ': return true; break;
			case '(': return true; break;
			case ':': return true; break;
			case '\t': return true; break;
			case '\n': return true; break;
			case '\r': return true; break;
			case ')': return true; break;
		}
	}
	return false;
}

void escapeSymbol(std::stringstream& ss, std::string const& str){
	for(int i=0; i<str.size(); ++i){
		char c = str[i];
		switch(c){
			case '\\':
				ss << '\\';
				ss << '\\';
			break;
			case '"':
				ss << '\\';
				ss << '"';
			break;
			case '\t':
				ss << '\\';
				ss << 't';
			break;
			case '\n':
				ss << '\\';
				ss << 'n';
			break;
			case '\r':
				ss << '\\';
				ss << 'r';
			break;
			default:
				ss << c;
		}
	}
}


struct List;
struct Stri{
	uint8_t tag;
	uint32_t line;
	uint32_t column;
	std::string s;
	static Stri create(std::string str);
	static Stri create(std::string str, uint32_t line, uint32_t column);
private:
	void baseLineStringify(std::stringstream& ss) const;
	void stringify(std::stringstream& ss) const;
	unsigned estimateLength() const;
	void prettyPrinting(std::stringstream& ss, unsigned depth, std::string indent, std::string lineEndings, unsigned lineWidth) const;
	friend Term;
	friend List;
};
struct List{
	uint8_t tag;
	uint32_t line;
	uint32_t column;
	std::vector<Term> l;
	static List create(std::vector<Term> ov);
	static List create(std::vector<Term> ov, uint32_t line, uint32_t column);
private:
	void stringify(std::stringstream& ss) const;
	unsigned estimateLength() const;
	void baseLineStringify(std::stringstream& ss) const;
	void prettyPrinting(std::stringstream& sb, unsigned depth, std::string indent, std::string lineEndings, unsigned lineWidth) const;
	friend Term;
	friend Stri;
};
union Term{
public:
	Stri s; List l;
	static void mimicInterTerm(Term* r, InterTerm* v);
	unsigned estimateLength() const;
	bool isStr() const;
	std::string toString() const;
	bool startsWith(std::string const& str) const;
	std::string prettyPrint() const;
	static inline Term parseMultiline(std::string& s);
	static inline Term parse(std::string& s);
	Term();
	Term(Term&& other);
	Term(Term const& other);
	Term(List other);
	Term(Stri other);
	Term(char const* other);
	Term(std::string other);
	Term& operator=(Term const& other);
	~Term();
private:
	void baseLineStringify(std::stringstream& ss) const;
	void stringify(std::stringstream& ss) const;
	void prettyPrinting(std::stringstream& sb, unsigned depth, std::string indent, std::string lineEndings, unsigned lineWidth) const;
	static void parseLengthedToSeqs(Term* termOut, std::string& unicodeString, char** errorOut);
	friend Stri;
	friend List;
};

struct TyperError:public TermposeError{
	std::vector<termpose::Error> errors;
	TyperError(Term const& t, std::string msg): TyperError(t.l.line,t.l.column,msg){}
	TyperError(std::vector<Error> errors): errors(errors), TermposeError("TyperError"){}
	TyperError(unsigned line, unsigned column, std::string msg):errors({Error{line, column, msg}}), TermposeError("TyperError"){}
	void suck(TyperError& other){
		using namespace detail;
		detail::append(errors, other.errors);
	}
	virtual char const* what() const noexcept {
		std::stringstream ss;
		ss<<"TyperError"<<'\n';
		for(Error const& e : errors) putErr(ss, e);
		return ss.str().c_str();
	}
	TyperError():TermposeError("TyperError"){}
};

Error error(Term& t, std::string msg){ return Error{t.l.line, t.l.column, msg}; }

void Stri::stringify(std::stringstream& ss) const{
	if(termpose::escapeIsNeeded(s)){
		ss<<'"';
		escapeSymbol(ss,s);
		ss<<'"';
	}else{
		ss<<s;
	}
}
void Stri::baseLineStringify(std::stringstream& ss) const{ stringify(ss); }
unsigned Stri::estimateLength() const{ return s.size(); }
void Stri::prettyPrinting(std::stringstream& ss, unsigned depth, std::string indent, std::string lineEndings, unsigned lineWidth) const{
	for(int i=0; i<depth; ++i){ ss<<indent; }
	stringify(ss);
	ss << lineEndings;
}

void Term::mimicInterTerm(Term* r, InterTerm* v){
	if(isInterStri(v)){
		auto is = (InterStri*)v;
		auto rs = &r->s;
		rs->tag = 1;
		rs->line = is->line;
		rs->column = is->column;
		new (& rs->s) std::string(is->v);
	}else{
		auto is = (InterSeqs*)v;
		auto rl = &r->l;
		rl->tag = 0;
		rl->line = is->line;
		rl->column = is->column;
		new (& rl->l) std::vector<Term>();
		std::vector<Term>& rv = rl->l;
		size_t vl = is->s.len;
		rv.reserve(vl);
		auto mimiced = [is](InterTerm* v){
			Term ntb;
			mimicInterTerm(&ntb, v);
			return ntb;
		};
		for(int i=0; i<vl; ++i){
			rv.emplace_back(mimiced((InterTerm*)(is->s.v[i])));
		}
	}
}
bool Term::isStr() const{ return (*(uint8_t*)this) != 0; }
Term::Term(){
	List* os = (List*)this;
	os->tag = 0;
	os->line = 0;
	os->column = 0;
	new (&os->l) std::vector<Term>();
}
Term::Term(Term&& other){
	if(other.isStr()){
		Stri* fs = &other.s;
		Stri* os = (Stri*)this;
		os->tag = 1;
		os->line = fs->line;
		os->column = fs->column;
		new (&os->s) std::string(std::move(fs->s));
	}else{
		List* fl = &other.l;
		List* ol = (List*)this;
		ol->tag = 0;
		ol->line = fl->line;
		ol->column = fl->column;
		new (&ol->l) std::vector<Term>(std::move(fl->l));
	}
}
Term::Term(Term const& other){
	if(other.isStr()){
		Stri const* fs = &other.s;
		Stri* os = (Stri*)this;
		os->tag = 1;
		os->line = fs->line;
		os->column = fs->column;
		new (&os->s) std::string(fs->s);
	}else{
		List const* fl = &other.l;
		List* ol = (List*)this;
		ol->tag = 0;
		ol->line = fl->line;
		ol->column = fl->column;
		new (&ol->l) std::vector<Term>(fl->l);
	}
}
Term::Term(char const* other){
	new (this) Term(std::string(other));
}
Term::Term(std::string other){
	Stri* os = (Stri*)this;
	os->tag = 1;
	os->line = 0;
	os->column = 0;
	new (&os->s) std::string(std::move(other));
}
Term::Term(List other){
	List* ol = (List*)this;
	ol->tag = other.tag;
	ol->line = other.line;
	ol->column = other.column;
	new (&ol->l) std::vector<Term>(std::move(other.l));
}
Term::Term(Stri other){
	Stri* os = (Stri*)this;
	os->tag = other.tag;
	os->line = other.line;
	os->column = other.column;
	new (&os->s) std::string(std::move(other.s));
}
Term& Term::operator=(Term const& other){
	this->~Term();
	new (this) Term(other);
}
Term::~Term(){
	if(isStr()){
		((Stri*)this)->~Stri();
	}else{
		((List*)this)->~List();
	}
}
unsigned Term::estimateLength() const{
	if(isStr()){
		s.estimateLength();
	}else{
		l.estimateLength();
	}
}
void Term::baseLineStringify(std::stringstream& ss) const{
	if(isStr()){
		s.baseLineStringify(ss);
	}else{
		l.baseLineStringify(ss);
	}
}
void Term::prettyPrinting(std::stringstream& sb, unsigned depth, std::string indent, std::string lineEndings, unsigned lineWidth) const{
	if(isStr()){
		s.prettyPrinting(sb, depth, indent, lineEndings, lineWidth);
	}else{
		l.prettyPrinting(sb, depth, indent, lineEndings, lineWidth);
	}
}
void Term::stringify(std::stringstream& ss) const{
	if(isStr()){
		s.stringify(ss);
	}else{
		l.stringify(ss);
	}
}
std::string Term::toString() const{
	std::stringstream ss;
	stringify(ss);
	return ss.str();
}
std::string Term::prettyPrint() const{
	std::stringstream ss;
	prettyPrinting(ss, 0, "  ", "\n", 80);
	return ss.str();
}

bool Term::startsWith(std::string const& str) const{
	return !isStr() && l.l.size() && l.l[0].isStr() && l.l[0].s.s == str;
}

inline Term Term::parseMultiline(std::string& unicodeString){ //note, throws ParserError if there was a syntax error
	//redefining stuff so that we can avoid the middle part of the translation from interterm to ctermpose::term to termpose::term. Musing: If I were REALLY serious I'd cut out the interterms alltogether and do a proper C++ port. Hm. I suppose there might be a way of abstracting over the differences between C and C++ and using the same parsing code for both.
	TermposeParsingError errorOut;
	Parser p;
	initParser(&p);
	parseLengthedToSeqsTakingParserRef(&p, unicodeString.c_str(), unicodeString.size(), &errorOut);
	if(!errorOut.msg){
		Term ret;
		List* lv = (List*)&ret;
		lv->tag = 0;
		lv->line = 0;
		lv->column = 0;
		new(&lv->l) std::vector<Term>();
		std::vector<Term>& outVec = lv->l;
		unsigned rarl = p.rootArBuf.len;
		InterTerm** rarv = (InterTerm**)p.rootArBuf.v;
		for(int i=0; i<rarl; ++i){
			Term t;
			mimicInterTerm(&t, rarv[i]);
			outVec.push_back(t);
		}
		return ret;
	}else{
		std::string msg(errorOut.msg);
		throw ParserError(errorOut.line,errorOut.column,msg);
	}
}

inline Term Term::parse(std::string& unicodeString){ //if there is exactly one line, it will be given sole, otherwise the lines(or lack thereof) will be wrapped in a List. parseMultiline has more consistent behavior, always wrapping the input in a List no matter how many root lines there were. throws ParserError if there was a syntax error
	Term ret = Term::parseMultiline(unicodeString);
	if(ret.isStr()){
		return ret;
	}else{
		List& rl = ret.l;
		if(rl.l.size() == 1){
			Term rr = rl.l[0];
			rl.l.pop_back();
			return rr;
		}else{
			return ret;
		}
	}
}

void List::stringify(std::stringstream& ss) const{
	ss<<'(';
	size_t ls = l.size();
	if(ls > 0){
		l[0].stringify(ss);
		for(int i=1; i<ls; ++i){
			ss<<' ';
			l[i].stringify(ss);
		}
	}
	ss<<')';
}
unsigned List::estimateLength() const{
	unsigned ret = 2;
	for(Term const& t : l){
		ret += 1 + t.estimateLength();
	}
	return ret;
}
void List::baseLineStringify(std::stringstream& ss) const{
	size_t ls = l.size();
	if(ls > 0){
		l[0].stringify(ss);
		for(int i=1; i<ls; ++i){
			ss<<' ';
			l[i].stringify(ss);
		}
	}else{
		ss << ':';
	}
}
void List::prettyPrinting(std::stringstream& sb, unsigned depth, std::string indent, std::string lineEndings, unsigned lineWidth) const{
	for(int i=0; i<depth; ++i){ sb << indent; }
	if(l.size() == 0){
		sb << ':';
		sb << lineEndings;
	}else{
		if(estimateLength() > lineWidth){
			//print in indent style
			if(l[0].estimateLength() > lineWidth){
				//indent sequence style
				sb << ':';
				sb << lineEndings;
				for(Term const& t : l){
					t.prettyPrinting(sb, depth+1, indent, lineEndings, lineWidth);
				}
			}else{
				l[0].stringify(sb);
				sb << lineEndings;
				size_t ln = l.size();
				for(int i=1; i<ln; ++i){
					l[i].prettyPrinting(sb, depth+1, indent, lineEndings, lineWidth);
				}
			}
		}else{
			//print inline style
			baseLineStringify(sb);
			sb << lineEndings;
		}
	}
}
Stri Stri::create(std::string str){ return Stri::create(str, 0,0); }
Stri Stri::create(std::string str, uint32_t line, uint32_t column){
	return Stri{1,line,column,str};
}
List List::create(std::vector<Term> iv){ return List::create(iv, 0,0); }
List List::create(std::vector<Term> ov, uint32_t line, uint32_t column){
	return List{0,line,column,ov};
}

bool operator==(Stri const& a, Stri const& b){ return a.s == b.s; }
bool operator!=(Term const& a, Term const& b);
bool operator==(List const& a, List const& b){
	unsigned len = a.l.size();
	if(len != b.l.size()) return false;
	for(int i=0; i<len; ++i){
		if(a.l[i] != b.l[i]) return false;
	}
	return true;
}
bool operator==(Term const& a, Term const& b){
	if(a.isStr()){
		if(b.isStr()){
			return (Stri const&)a == (Stri const&)b;
		}else{
			return false;
		}
	}else{
		if(b.isStr()){
			return false;
		}else{
			return (List const&)a == (List const&)b;
		}
	}
}

bool operator!=(Stri const& a, Stri const& b){ return a.s != b.s; }
bool operator!=(List const& a, List const& b){ return ! (a == b); }
bool operator!=(Term const& a, Term const& b){ return ! (a == b); }

template<class... Rest>
inline static void load_terms(std::vector<Term>& o, Term t, Rest... rest){
	o.push_back(t);
	load_terms(o,rest...);
}
inline static void load_terms(std::vector<Term>& o){}

template<class... Rest>
static Term terms(Rest... rest){
	std::vector<Term> o;
	load_terms(o, rest...);
	return List::create(o);
}


namespace parsingDSL{
	
	template<typename T> struct Checker{
		virtual T check(Term const& v) = 0; //throws TyperError
	};
	template<typename T> struct Termer{
		virtual Term termify(T const& v) = 0;
	};
	template<typename T> struct Translator : Checker<T>, Termer<T>{
	};
	template<typename T> struct BuiltTranslator : Translator<T> {
		Checker<T> ck;
		Termer<T> tk;
		virtual T check(Term& v){ return ck.check(v); }
		virtual Term termify(T& v){ return tk.termify(v); }
	};
	
	Term termifyBool(bool const& v){
		return Stri::create(v?"true":"false");
	}
	bool checkBool(Term const& v){
		if(v.isStr()){
			std::string const& st = v.s.s;
			if(st == "true" || st == "⊤") return true;
			else if(st == "false" || st == "⊥") return false;
			else throw TyperError(v, "expected a bool here");
		}else{
			throw TyperError(v, "expected a bool here");
		}
	}
	struct BoolTermer : Termer<bool>{ virtual Term termify(bool& b){
		return termifyBool(b);
	}};
	struct BoolChecker : Checker<bool>{ virtual bool check(Term& v){
		return checkBool(v);
	}};
	struct BoolTranslator : Translator<bool> {
		virtual Term termify(bool const& b){
			return termifyBool(b);
		}
		virtual bool check(Term const& v){
			return checkBool(v);
		}
	};
	
	Term termString(std::string const& b){
		std::stringstream ss;
		ss << b;
		return Stri::create(ss.str());
	}
	std::string checkString(Term const& v){
		if(v.isStr()){
			return v.s.s;
		}else{
			throw TyperError(v, "expected a string here");
		}
	}
	struct StringTermer : Termer<std::string>{ virtual Term termify(std::string const& b){
		return termString(b);
	}};
	struct StringChecker : Checker<std::string>{ virtual std::string check(Term const& v){
		return checkString(v);
	}};
	struct StringTranslator : Translator<std::string> {
		std::string check(Term const& v){ return checkString(v); }
		Term termify(std::string const& v){ return termString(v); }
	};
	
	struct BlandTermer : Termer<Term>{ virtual Term termify(Term const& b){ return b; }};
	struct BlandChecker : Checker<Term>{ virtual Term check(Term const& v){ return v; }};
	struct BlandTranslator : Translator<Term> {
		Term termify(Term const& b){ return b; }
		Term check(Term const& v){ return v; }
	};
	
	Term termInt(int const& b){
		std::stringstream ss;
		ss << b;
		return Stri::create(ss.str());
	}
	int checkInt(Term const& v){
		if(v.isStr()){
			std::string const& st = v.s.s;
			try{
				return stoi(st);
			}catch(std::invalid_argument e){
				throw TyperError(v, "expected a int here (can't be parsed to an int)");
			}catch(std::out_of_range e){
				throw TyperError(v, "expected a int here (is not in the allowable range)");
			}
		}else{
			throw TyperError(v, "expected a int here (is a list)");
		}
	}
	struct IntTermer : Termer<int>{ virtual Term termify(int const& b){ return termInt(b); } };
	struct IntChecker : Checker<int>{ int check(Term const& v){ return checkInt(v); } };
	struct IntTranslator : Translator<int> {
		virtual Term termify(int const& b){ return termInt(b); }
		virtual int check(Term const& v){ return checkInt(v); }
	};
	
	
	Term termifyFloat(float const& b){
		std::stringstream ss;
		ss << b;
		return Stri::create(ss.str());
	}
	float checkFloat(Term const& v){
		if(v.isStr()){
			std::string const& st = v.s.s;
			try{
				return stof(st);
			}catch(std::invalid_argument e){
				throw TyperError(v, "expected a float here (can't be parsed to an float)");
			}catch(std::out_of_range e){
				throw TyperError(v, "expected a float here (is not in the allowable range)");
			}
		}else{
			throw TyperError(v, "expected a float here (is a list)");
		}
	}
	struct FloatTermer : Termer<float>{ virtual Term termify(float const& b){ return termifyFloat(b); } };
	struct FloatChecker : Checker<float>{ virtual float check(Term const& v){ return checkFloat(v); } };
	struct FloatTranslator : Translator<float> {
		virtual Term termify(float const& b){ return termifyFloat(b); }
		virtual float check(Term const& v){ return checkFloat(v); }
	};
	
	
	Term termifyDouble(double const& b){
		std::stringstream ss;
		ss << b;
		return Stri::create(ss.str());
	}
	double checkDouble(Term const& v){
		if(v.isStr()){
			std::string const& st = v.s.s;
			try{
				return stof(st);
			}catch(std::invalid_argument e){
				throw TyperError(v, "expected a double here (can't be parsed to an double)");
			}catch(std::out_of_range e){
				throw TyperError(v, "expected a double here (is not in the allowable range)");
			}
		}else{
			throw TyperError(v, "expected a double here (is a list)");
		}
	}
	struct DoubleTermer : Termer<double>{ virtual Term termify(double const& b){ return termifyDouble(b); } };
	struct DoubleChecker : Checker<double>{ virtual double check(Term const& v){ return checkDouble(v); } };
	struct DoubleTranslator : Translator<double> {
		virtual Term termify(double const& b){ return termifyFloat(b); }
		virtual double check(Term const& v){ return checkFloat(v); }
	};
	
	template<typename A, typename B>
	struct PairTranslator : Translator<std::pair<A,B>> {
		std::shared_ptr<Translator<A>> at;
		std::shared_ptr<Translator<B>> bt;
		PairTranslator(std::shared_ptr<Translator<A>> at, std::shared_ptr<Translator<B>> bt):at(at),bt(bt) {}
		virtual Term termify(std::pair<A,B> const& p){
			return List::create(std::vector<Term>({at->termify(p.first), bt->termify(p.second)}));
		}
		virtual std::pair<A,B> check(Term const& v){
			//todo, if both terms are erronious, the TyperError should pass along both errors
			if(v.isStr()){
				throw TyperError(v, "expected a pair here, but it's a string term");
			}else if(v.l.l.size() < 2){
				throw TyperError(v, "expected a pair here, but this term has less than 2 items");
			}else if(v.l.l.size() > 2){
				throw TyperError(v, "expected a pair here, but this term has more than 2 items");
			}else{
				A a = at->check(v.l.l[0]);
				B b = bt->check(v.l.l[1]);
				return std::make_pair(move(a), move(b));
			}
		}
	};
	
	
	template<typename A, typename B>
	struct StartAndSequence : Translator<std::pair<A,std::vector<B>>> {
		std::shared_ptr<Translator<A>> at;
		std::shared_ptr<Translator<B>> bt;
		bool ignoreErroneousContent;
		StartAndSequence(std::shared_ptr<Translator<A>> at, std::shared_ptr<Translator<B>> bt, bool ignoreErroneousContent = false):at(at),bt(bt),ignoreErroneousContent(ignoreErroneousContent) {}
		virtual Term termify(std::pair<A,std::vector<B>> const& p){
			std::vector<Term> ov;
			ov.push_back(at.termify(p.first));
			for(B const& as : p.second){
				ov.push_back(bt.termify(as));
			}
			return List::create(move(ov));
		}
		virtual std::pair<A,std::vector<B>> check(Term const& v){
			//todo, propagate all errors
			if(v.isStr()){
				throw TyperError(v, "expected a sequence here, but it's a string term");
			}else{
				std::vector<Term> const& vl = v.l.l;
				if(vl.size() < 1){
					throw TyperError(v, "expected at least one element here, but this sequence is empty");
				}else{
					std::vector<B> results;
					std::vector<Error> errors;
					A a = at.check(vl[0]);
					for(int i=1; i<vl.size(); ++i){
						try{
							B res = bt->check(vl[i]);
							if(errors.size() == 0){
								results.push_back(std::move(res));
							}
						}catch(TyperError e){
							if(!ignoreErroneousContent){
								detail::append(errors, e.errors);
							}
						}
					}
					if(errors.size()){
						throw TyperError(std::move(errors));
					}else{
						return std::make_pair(move(a),move(results));
					}
				}
			}
		}
	};
	
	
	
	
	template <class A, class B>
	std::pair<A,B> checkSliceStartAndList(Term const& v, Checker<A>* ac, Checker<B>* bc){
		if(v.isStr()){
			throw TyperError(v, std::string("expected a sequence with at least one element, but it's a string"));
		}else{
			std::vector<Term> const& fv = v.l.l;
			if(fv.size() >= 1){
				Term const& tagTerm = fv[0];
				A a = ac->check(tagTerm);
				std::vector<Term> outv;
				outv.reserve(fv.size()-1);
				for(int i=1; i<fv.size(); ++i){
					outv.push_back(fv[i]);
				}
				return std::make_pair(a,bc->check(List::create(outv, v.l.line, v.l.column)));
			}else{
				throw TyperError(v, std::string("expected a list with at least one element, but the list is empty"));
			}
		}
	}
	
	template<class A, class B>
	struct SliceStartAndListChecker : Checker<std::pair<A,B>> {
		std::shared_ptr<Checker<A>> ac;
		std::shared_ptr<Checker<B>> bc;
		SliceStartAndListChecker(
			std::string tag,
			std::shared_ptr<Checker<A>> ac,
			std::shared_ptr<Checker<B>> bc
		):ac(ac),bc(bc) {}
		virtual std::pair<A,B> check(Term const& v){
			return checkSliceStartAndList(v,&*ac,&*bc);
		}
	};
	
	template<class A, class B>
	struct SliceStartAndListTranslator : Translator<std::pair<A,B>> {
		std::shared_ptr<Translator<A>> ac;
		std::shared_ptr<Translator<B>> bc;
		SliceStartAndListTranslator(
			std::shared_ptr<Translator<A>> ac,
			std::shared_ptr<Translator<B>> bc
		):ac(ac),bc(bc) {}
		virtual std::pair<A,B> check(Term const& v){
			return checkSliceStartAndList(v,&*ac,&*bc);
		}
		virtual Term termify(std::pair<A,B> const& v){
			Term at = ac->termify(v.first);
			Term bt = bc->termify(v.second);
			std::vector<Term> cv;
			if(bt.isStr()){
				// throw TyperError(0,0, "couldn't termify, tagSlice expects the content to be a list, but it termed to a string");
				cv.reserve(2);
				cv.push_back(std::move(at));
				cv.push_back(std::move(bt));
			}else{
				cv.reserve(bt.l.l.size()+1);
				cv.push_back(std::move(at));
				detail::append(cv, bt.l.l);
			}
			return List::create(std::move(cv));
		}
	};
	
	
	
	
	template<typename B>
	B checkEnsuredLiteral(Term const& v, std::string& aLiteral, Checker<B>* bt){
		//todo, detect and propagate all errors
		if(v.isStr()){
			throw TyperError(v, "expected a pair here, but it's a string term");
		}else if(v.l.l.size() < 2){
			throw TyperError(v, "expected a pair here, but this term has less than 2 items");
		}else if(v.l.l.size() > 2){
			throw TyperError(v, "expected a pair here, but this term has more than 2 items");
		}else{
			Term const& aterm = v.l.l[0];
			if(aterm.isStr()){
				if(aterm.s.s != aLiteral){
					throw TyperError(v.l.l[0], std::string("expected \"")+aLiteral+"\" here");
				}
				B b = bt->check(v.l.l[1]);
				return b;
			}else{
				throw TyperError(aterm, "expected a string literal, \""+aLiteral+"\", but it's a list term");
			}
		}
	}
	template<typename B>
	struct LiteralEnsureTranslator : Translator<B> {
		std::string aLiteral;
		std::shared_ptr<Translator<B>> bt;
		LiteralEnsureTranslator(std::string aLiteral, std::shared_ptr<Translator<B>> bt):aLiteral(aLiteral), bt(bt) {}
		virtual Term termify(B const& b){
			return List::create(std::vector<Term>{Stri::create(aLiteral), bt->termify(b)});
		}
		virtual B check(Term const& v){
			return checkEnsuredLiteral(v,aLiteral,&*bt);
		}
	};
	template<typename B>
	struct LiteralEnsureChecker : Checker<B> {
		std::string aLiteral;
		std::shared_ptr<Checker<B>> bt;
		LiteralEnsureChecker(std::string aLiteral, std::shared_ptr<Checker<B>> bt):aLiteral(aLiteral), bt(bt) {}
		virtual B check(Term const& v){
			return checkEnsuredLiteral(v,aLiteral,&*bt);
		}
	};
	template<typename B>
	struct LiteralEnsureTermer : Termer<B> {
		std::string aLiteral;
		std::shared_ptr<Termer<B>> bt;
		LiteralEnsureTermer(std::string aLiteral, std::shared_ptr<Termer<B>> bt):aLiteral(aLiteral), bt(bt) {}
		virtual Term termify(B const& b){
			return List::create(std::vector<Term>{Stri::create(aLiteral), bt.termify(b)});
		}
	};
	
	
	template<typename A, typename B>
	struct MapConverter : Translator<std::unordered_map<A,B>> {
		std::shared_ptr<Translator<std::vector<std::pair<A,B>>>> st;
		MapConverter(std::shared_ptr<Translator<std::vector<std::pair<A,B>>>> st):st(st) {}
		Term termify(std::unordered_map<A,B> const& m){
			return st->termify(std::vector<std::pair<A,B>>(m.begin(), m.end()));
		}
		std::unordered_map<A,B> check(Term const& v){
			auto vec = st->check(v);
			return std::unordered_map<A,B>(vec.begin(), vec.end());
		}
	};
	
	template<typename T>
	Term sequenceTermify(
		Termer<T>* innerTermer,
		std::vector<T> const& v
	){
		std::vector<Term> ov;
		for(T const& iv : v){
			ov.push_back(innerTermer->termify(iv));
		}
		return List::create(std::move(ov));
	}
	template<typename T>
	std::vector<T> sequenceCheck(
		Checker<T>* innerChecker,
		bool ignoreErroneousContent,
		Term const& t
	){
		if(t.isStr()){
			throw TyperError(t, "expected a sequence here");
		}else{
			List& tl = (List&)t;
			std::vector<T> results;
			std::vector<Error> errors;
			for(Term& it : tl.l){
				try{
					T res = innerChecker->check(it);
					if(errors.size() == 0){
						results.push_back(std::move(res));
					}
				}catch(TyperError e){
					if(!ignoreErroneousContent){
						detail::append(errors, e.errors);
					}
				}
			}
			if(errors.size()){
				throw TyperError(std::move(errors));
			}else{
				return results;
			}
		}
	}
	template<typename T> struct SequenceTermer : Termer<std::vector<T>> {
		std::shared_ptr<Termer<T>> innerTermer;
		SequenceTermer(std::shared_ptr<Termer<T>> it):innerTermer(it) {}
		virtual Term termify(std::vector<T> const& v){
			return sequenceTermify(&*innerTermer, v);
		}
	};
	template<typename T> struct SequenceChecker : Checker<std::vector<T>> {
		std::shared_ptr<Checker<T>> innerChecker;
		bool ignoreErroneousContent;
		SequenceChecker(std::shared_ptr<Checker<T>> ic, bool ignoreErroneousContent = false):innerChecker(ic), ignoreErroneousContent(ignoreErroneousContent) {}
		virtual std::vector<T> check(Term const& t){
			return sequenceCheck(&*innerChecker, ignoreErroneousContent, t);
		}
	};
	template<typename T> struct SequenceTranslator : Translator<std::vector<T>> {
		std::shared_ptr<Translator<T>> innerTranslator;
		bool ignoreErroneousContent = false;
		SequenceTranslator(
			std::shared_ptr<Translator<T>> ic,
			bool ignoreErroneousContent = false
		):innerTranslator(ic), ignoreErroneousContent(ignoreErroneousContent) {}
		virtual Term termify(std::vector<T> const& v){
			return sequenceTermify(&*innerTranslator, v);
		}
		virtual std::vector<T> check(Term const& t){
			return sequenceCheck(&*innerTranslator, ignoreErroneousContent, t);
		}
	};
	
	
	
	template<typename A, typename B>
	std::unordered_map<A,B> checkMap(Term const& v, Checker<A>* ac, Checker<B>* bc, bool ignoreErroneousContent){
		if(v.isStr()){
			throw TyperError(v, "expected a map here, but it's a string term");
		}else{
			List& tl = (List&)v;
			std::unordered_map<A,B> results;
			std::vector<Error> errors;
			for(Term& it : tl.l){
				std::cout<<"tlis "<<it.isStr()<<std::endl;
				if(it.isStr()){
					errors.push_back(error(it, "expected a pair(in a map) here, but it's a string term")); continue;
				}else if(it.l.l.size() < 2){
					errors.push_back(error(it, "expected a pair(in a map) here, but this term has less than 2 items")); continue;
				}else if(it.l.l.size() > 2){
					errors.push_back(error(it, "expected a pair(in a map) here, but this term has more than 2 items")); continue;
				}else{
					try{
						A a = ac->check(it.l.l[0]);
						try{
							B b = bc->check(it.l.l[1]);
							if(errors.size() == 0){
								results.insert({std::move(a), std::move(b)});
							}
						}catch(TyperError e){
							if(!ignoreErroneousContent){
								detail::append(errors, e.errors);
							}
						}
					}catch(TyperError e){
						if(!ignoreErroneousContent){
							detail::append(errors, e.errors);
						}
					}
				}
			}
			if(errors.size()){
				throw TyperError(std::move(errors));
			}else{
				return results;
			}
		}
	}
	
	template<typename A, typename B>
	struct MapTranslator : Translator<std::unordered_map<A,B>> {
		std::shared_ptr<Translator<A>> at;
		std::shared_ptr<Translator<B>> bt;
		bool ignoreErroneousContent = false;
		MapTranslator(std::shared_ptr<Translator<A>> at, std::shared_ptr<Translator<B>> bt, bool ignoreErroneousContent=false):
			at(at),
			bt(bt),
			ignoreErroneousContent(ignoreErroneousContent)
		{}
		Term termify(std::unordered_map<A,B> const& m){
			std::vector<Term> ov;
			for(auto& pair : m){
				ov.push_back(List::create(std::vector<Term>({at->termify(pair.first), bt->termify(pair.second)})));
			}
			return List::create(std::move(ov));
		}
		std::unordered_map<A,B> check(Term const& v){
			return checkMap(v, &*at, &*bt, ignoreErroneousContent);
		}
	};
	
	template<typename A, typename B>
	struct MapTermer : Termer<std::unordered_map<A,B>> {
		std::shared_ptr<Termer<A>> at;
		std::shared_ptr<Termer<B>> bt;
		MapTermer(std::shared_ptr<Termer<A>> at, std::shared_ptr<Termer<B>> bt):
			at(at),
			bt(bt)
		{}
		Term termify(std::unordered_map<A,B> const& m){
			std::vector<Term> ov;
			for(auto& pair : m){
				ov.push_back(List::create(std::vector<Term>({at->termify(pair.first), bt->termify(pair.second)})));
			}
			return List::create(std::move(ov));
		}
	};
	
	template<typename A, typename B>
	struct MapChecker : Checker<std::unordered_map<A,B>> {
		std::shared_ptr<Checker<A>> at;
		std::shared_ptr<Checker<B>> bt;
		bool ignoreErroneousContent = false;
		MapChecker(std::shared_ptr<Checker<A>> at, std::shared_ptr<Checker<B>> bt, bool ignoreErroneousContent=false):
			at(at),
			bt(bt),
			ignoreErroneousContent(ignoreErroneousContent)
		{}
		std::unordered_map<A,B> check(Term const& v){
			return checkMap(v, &*at, &*bt);
		}
	};
	
	template<class T>
	std::vector<T> checkTaggedSequence(Term const& v, std::string& tag, Checker<T>* tt){
		if(v.isStr()){
			throw TyperError(v, std::string("expected a sequence starting with \"")+tag+"\"");
		}else{
			std::vector<Term> const& fv = v.l.l;
			if(fv.size() >= 1){
				Term const& tagTerm = fv[0];
				if(tagTerm.isStr()){
					if(tagTerm.s.s != tag){
						throw TyperError(tagTerm, std::string("expected tag to be \"")+tag+"\", but it's \""+tagTerm.s.s+"\"");
					}
				}else{
					throw TyperError(tagTerm, std::string("expected a tag term \"")+tag+"\" here, but it's a list");
				}
				std::vector<T> outv;
				for(int i=1; i<fv.size(); ++i){
					outv.push_back(tt->check(fv[i]));
				}
				return outv;
			}else{
				throw TyperError(v, std::string("expected a sequence starting with \"")+tag+"\", but the sequence is empty");
			}
		}
	}
	template<class T>
	Term termifyTaggedSequence(std::vector<T> const& v, std::string& tag, Termer<T>* tt){
		std::vector<Term> cv;
		cv.reserve(v.size()+1);
		cv.push_back(Stri::create(std::string(tag)));
		for(T const& vt: v){
			cv.push_back(tt->termify(vt));
		}
		return List::create(cv);
	}
	template<typename T>
	struct TaggedSequenceTranslator : Translator<std::vector<T>> {
		std::string tag;
		std::shared_ptr<Translator<T>> tt;
		TaggedSequenceTranslator(
			std::string tag,
			std::shared_ptr<Translator<T>> tt
		):tag(tag),tt(tt){}
		virtual std::vector<T> check(Term const& v){
			return checkTaggedSequence(v,tag,&*tt);
		}
		virtual Term termify(std::vector<T> const& v){
			return termifyTaggedSequence(v,tag,&*tt);
		}
	};
	template<typename T>
	struct TaggedSequenceChecker : Checker<std::vector<T>> {
		std::string tag;
		std::shared_ptr<Checker<T>> tt;
		TaggedSequenceChecker(
			std::string tag,
			std::shared_ptr<Checker<T>> tt
		):tag(tag),tt(tt){}
		virtual std::vector<T> check(Term const& v){
			return checkTaggedSequence(v,tag,&*tt);
		}
	};
	template<typename T>
	struct TaggedSequenceTermer : Termer<std::vector<T>> {
		std::string tag;
		std::shared_ptr<Termer<T>> tt;
		TaggedSequenceTermer(
			std::string tag,
			std::shared_ptr<Termer<T>> tt
		):tag(tag),tt(tt){}
		virtual Term termify(std::vector<T> const& v){
			return termifyTaggedSequence(v,tag,tt);
		}
	};
	

	template <class T>
	T checkTagSlicing(Term const& v, std::string& tag, Checker<T>* inner){
		if(v.isStr()){
			throw TyperError(v, std::string("expected a sequence starting with \"")+tag+"\"");
		}else{
			std::vector<Term> const& fv = v.l.l;
			if(fv.size() >= 1){
				Term const& tagTerm = fv[0];
				if(tagTerm.isStr()){
					if(tagTerm.s.s != tag){
						throw TyperError(tagTerm, std::string("expected tag to be \"")+tag+"\", but it's "+tagTerm.s.s);
					}else{
						std::vector<Term> outv;
						outv.reserve(fv.size()-1);
						for(int i=1; i<fv.size(); ++i){
							outv.push_back(fv[i]);
						}
						return inner->check(List::create(outv, v.l.line, v.l.column + tagTerm.s.s.size())); //caveat, this will give slightly incorrect line numbers for many unicode strings, but I don't know who cares.
					}
				}else{
					throw TyperError(tagTerm, std::string("expected a tag term \"")+tag+"\" here, but it's a list");
				}
			}else{
				throw TyperError(v, std::string("expected a sequence starting with \"")+tag+"\", but the sequence is empty");
			}
		}
	}
	
	template<class T>
	struct TagSlicingChecker : Checker<T> {
		std::string tag;
		std::shared_ptr<Checker<T>> inner;
		TagSlicingChecker(
			std::string tag,
			std::shared_ptr<Checker<T>> inner
		):tag(tag),inner(inner){}
		virtual T check(Term const& v){
			return checkTagSlicing(v,tag,&*inner);
		}
	};
	
	template<class T>
	struct TagSlicing : Translator<T> {
		std::string tag;
		std::shared_ptr<Translator<T>> inner;
		TagSlicing(
			std::string tag,
			std::shared_ptr<Translator<T>> inner
		):tag(tag),inner(inner){}
		virtual T check(Term const& v){
			return checkTagSlicing(v,tag,&*inner);
		}
		virtual Term termify(T const& v){
			Term rt = inner->termify(v);
			std::vector<Term> cv;
			Term tagTerm = Stri::create(std::string(tag));
			if(rt.isStr()){
				throw TyperError(0,0, "couldn't termify, tagSlice expects the content to be a list, but it termed to a string");
				cv.reserve(2);
				cv.push_back(std::move(tagTerm));
				cv.push_back(std::move(rt));
			}else{
				cv.reserve(rt.l.l.size()+1);
				cv.push_back(std::move(tagTerm));
				detail::append(cv, rt.l.l);
			}
			return List::create(std::move(cv));
		}
	};
	
	// This insanity is the first and last thing I ever did with C++'s variadic templates. It was as if they provided the bare minimum functionality with no understanding of what primitives people would need in order to use it elegantly.
	template<int... T>
	struct seq {};

	template<int... N>
	struct count_to{};
	template<int N, int... S>
	struct count_to<N, S...> : count_to<N-1, N-1, S...> {};
	template<int... S>
	struct count_to<0, S...>{
		typedef seq<S...> type;
	};
	template<typename F, typename... TArg>
	auto of_result()-> decltype(std::declval<F>()(std::declval<TArg>()...));
	
	template<typename F, class... Args, int... Indices>
	static inline auto check_terms_through_proper(
		seq<Indices...>,
		std::tuple<Checker<Args>*...>& args,
		List const& s,
		F& f
	)-> decltype(f(std::declval<Args>()...)){
		return f(std::get<Indices>(args)->check(s.l[Indices])...);
	}
	template<typename F, class... Args, int... Indices>
	static auto check_terms_through(std::tuple<Checker<Args>*...>& args, List const& s, F& f)-> decltype(f(std::declval<Args>()...)) {
		return check_terms_through_proper(typename count_to<sizeof...(Args)>::type(), args, s, f);
	}
	template<int... Indices, class... Types>
	static inline auto as_checker_ptrs_proper(
		seq<Indices...>,
		std::tuple<std::shared_ptr<Checker<Types>>...> const& args
	)-> std::tuple<Checker<Types>*...> {
		return std::tuple<Checker<Types>*...>(&*std::get<Indices>(args)...);
	}
	template<class... Types>
	static auto as_checker_ptrs(std::tuple<std::shared_ptr<Checker<Types>>...> const& args)-> std::tuple<Checker<Types>*...> {
		return as_checker_ptrs_proper(typename count_to<sizeof...(Types)>::type(), args);
	}
	template<class F, class... Args>
	static auto checkReduction(Term const& v, F& f, std::tuple<Checker<Args>*...>& checkers)-> decltype(of_result<F,Args...>()) {
		unsigned len = sizeof...(Args);
		if(v.isStr()){
			std::stringstream ss;
			ss << "expected a tuple of length " << len << ", but it's a string term";
			throw TyperError(v, ss.str());
		}else{
			List const& s = v.l;
			if(s.l.size() != len){
				std::stringstream ss;
				ss<<"expected a series of length "<<len<<", but the series was of length "<<s.l.size();
				throw TyperError(s.line, s.column, ss.str());
			}
			return check_terms_through(checkers, s, f);
		}
	}
	
	template<class F, class... Args>
	struct ReductionChecker : Checker<decltype(of_result<F,Args...>())>{
	private:
		std::tuple<std::shared_ptr<Checker<Args>>...> checkers;
		F f;
	public:
		ReductionChecker(std::tuple<std::shared_ptr<Checker<Args>>...> checkers, F f):checkers(checkers),f(f){}
		decltype(of_result<F,Args...>()) check(Term const& v){
			std::tuple<Checker<Args>*...> checkerPtrs = as_checker_ptrs(checkers);
			return checkReduction(v, f, checkerPtrs);
		}
	};
	
	template<int N, int I, class... Args>
	struct ReductionTermingMapThrough{
		inline static void map_through(
			std::tuple<Args...>& res,
			std::tuple<Termer<Args>*...>& termers,
			std::vector<Term>& out
		){
			out.push_back(std::get<I>(termers)->termify(std::get<I>(res)));
			ReductionTermingMapThrough<N,I+1,Args...>::map_through(res,termers,out);
		}
	};
	template<int N, class... Args>
	struct ReductionTermingMapThrough<N,N,Args...>{
		inline static void map_through(
			std::tuple<Args...>& res,
			std::tuple<Termer<Args>*...>& termers,
			std::vector<Term>& out
		){}
	};
	template<class F, class T, class... Args>
	struct ReductionTermer : Termer<T>{
	private:
		auto resolve(std::tuple<Args...>& result)-> std::vector<Term> {
			std::vector<Term> out;
			ReductionTermingMapThrough<sizeof...(Args),0, Args...>::map_through(result, termers, out);
			return out;
		}
		
		std::tuple<std::shared_ptr<Termer<Args>>...> termers;
		F f;
	public:
		ReductionTermer(std::tuple<std::shared_ptr<Checker<Args>>...> termers, F f):termers(termers),f(f){}
		Term termify(T const& v){
			return List::create(resolve(f(v)));
		}
	};
	
	template<class Reduction, class Expansion, class... Types>
	struct ReductionTranslator : Translator<decltype(of_result<Reduction,Types...>())> {
		Reduction rf;
		Expansion ef;
		std::tuple<std::shared_ptr<Translator<Types>>...> translators;
		
		template<int... Indices>
		static inline auto get_as_checkers(
			seq<Indices...>,
			std::tuple<std::shared_ptr<Translator<Types>>...>& args
		)-> std::tuple<Checker<Types>*...> {
			return std::tuple<Checker<Types>*...>(&*std::get<Indices>(args)...);
		}
		static auto as_checkers(std::tuple<std::shared_ptr<Translator<Types>>...>& args)-> std::tuple<Checker<Types>*...> {
			return get_as_checkers(typename count_to<sizeof...(Types)>::type(), args);
		}
		
		template<int... Indices>
		static inline auto get_as_termers(
			seq<Indices...>,
			std::tuple<std::shared_ptr<Translator<Types>>...>& args
		)-> std::tuple<Termer<Types>*...> {
			return std::tuple<Termer<Types>*...>(&*std::get<Indices>(args)...);
		}
		static auto as_termers(std::tuple<std::shared_ptr<Translator<Types>>...>& args)-> std::tuple<Termer<Types>*...> {
			return get_as_termers(typename count_to<sizeof...(Types)>::type(), args);
		}
		
		ReductionTranslator(Reduction rf, Expansion ef, std::tuple<std::shared_ptr<Translator<Types>>...> translators):
			rf(rf), ef(ef), translators(translators)
		{}
		auto translate_and_vectorize(std::tuple<Types...>& result)-> std::vector<Term> {
			std::vector<Term> out;
			auto termers = as_termers(translators);
			ReductionTermingMapThrough<sizeof...(Types),0, Types...>::map_through(result, termers, out);
			return out;
		}
		
		Term termify(decltype(of_result<Reduction,Types...>()) const& v){
			std::tuple<Types...> res = ef(v);
			return List::create(translate_and_vectorize(res));
		}
		decltype(of_result<Reduction,Types...>()) check(Term const& v){
			std::tuple<Checker<Types>*...> ts = as_checkers(translators);
			return checkReduction(v, rf, ts);
		}
	};
	
	
	template<typename F, typename... Ts>
	static std::shared_ptr<Checker<decltype(of_result<F,Ts...>())>> combineCheck(F fun, std::shared_ptr<Checker<Ts>>... mappers){
		return std::shared_ptr<Checker<decltype(of_result<F,Ts...>())>>(new ReductionChecker<F,Ts...>(make_tuple(mappers...), fun));
	}
	
	template<typename Reducer, typename Input, typename... Ts>
	static std::shared_ptr<Termer<Input>> combineTerm(Reducer fr, std::shared_ptr<Termer<Ts>>... mappers){
		return std::shared_ptr<Termer<Input>>(new ReductionTermer<Reducer,Input,Ts...>(make_tuple(mappers...), fr));
	}
	template<class Reduction, class Expansion, class... Ts>
	static std::shared_ptr<Translator<decltype(of_result<Reduction,Ts...>())>> combineTrans(Reduction fr, Expansion fe, std::shared_ptr<Translator<Ts>>... mappers){
		return std::shared_ptr<Translator<decltype(of_result<Reduction,Ts...>())>>(
			new ReductionTranslator<Reduction,Expansion,Ts...>(fr,fe,make_tuple(mappers...))
		);
	}
	template<typename T>
	static std::shared_ptr<Translator<std::vector<T>>> sequenceTrans(
		std::shared_ptr<Translator<T>> tt
	){
		return std::shared_ptr<Translator<std::vector<T>>>(new SequenceTranslator<T>(std::move(tt)));
	}
	template<typename T>
	static std::shared_ptr<Checker<std::vector<T>>> sequenceTrans(
		std::shared_ptr<Checker<T>> tt
	){
		return std::shared_ptr<Checker<std::vector<T>>>(new SequenceChecker<T>(std::move(tt)));
	}
	//functionally equivalent to sliceOffTag(tag, sequenceTrans(tt)).
	template<typename T>
	static std::shared_ptr<Translator<std::vector<T>>> taggedSequence(
		std::string tag,
		std::shared_ptr<Translator<T>> tt
	){
		return std::shared_ptr<Translator<std::vector<T>>>(new TaggedSequenceTranslator<T>(tag, tt));
	}
	template<typename T>
	static std::shared_ptr<Checker<std::vector<T>>> taggedSequence(
		std::string tag,
		std::shared_ptr<Checker<T>> tt
	){
		return std::shared_ptr<Checker<std::vector<T>>>(new TaggedSequenceChecker<T>(tag, tt));
	}
	//requires that the first element in the term be tag, and and passes down a copy of the list one shorter.
	template<typename T>
	static std::shared_ptr<Translator<T>> sliceOffTag(std::string tag, std::shared_ptr<Translator<T>> inner){
		return std::shared_ptr<Translator<T>>(new TagSlicing<T>(tag, inner)); }
	template<typename T>
	static std::shared_ptr<Checker<T>> sliceOffTag(std::string tag, std::shared_ptr<Checker<T>> inner){
		return std::shared_ptr<Checker<T>>(new TagSlicingChecker<T>(tag, inner)); }
	//requires that the first element in the term be `literal`, and passes the second element through to bt.
	template<typename B>
	static std::shared_ptr<Translator<B>> ensureTag(std::string literal, std::shared_ptr<Translator<B>> bt){
		return std::shared_ptr<Translator<B>>(new LiteralEnsureTranslator<B>(literal, bt)); }
	template<typename B>
	static std::shared_ptr<Checker<B>> ensureTag(std::string literal, std::shared_ptr<Checker<B>> bt){
		return std::shared_ptr<Checker<B>>(new LiteralEnsureChecker<B>(literal, bt)); }
	template<typename B>
	static std::shared_ptr<Termer<B>> ensureTag(std::string literal, std::shared_ptr<Termer<B>> bt){
		return std::shared_ptr<Termer<B>>(new LiteralEnsureTermer<B>(literal, bt)); }
	template<typename A, typename B>
	static std::shared_ptr<Translator<std::pair<A,std::vector<B>>>> separateStartAndSequence(std::shared_ptr<Translator<A>> at, std::shared_ptr<Translator<B>> bt){
		return std::shared_ptr<Translator<std::pair<A,std::vector<B>>>>(new StartAndSequence<A,B>(at,bt)); }
	template<typename A, typename B>
	static std::shared_ptr<Translator<std::pair<A,B>>> sliceStartAndList(std::shared_ptr<Translator<A>> at, std::shared_ptr<Translator<B>> bt){
		return std::shared_ptr<Translator<std::pair<A,B>>>(new SliceStartAndListTranslator<A,B>(at,bt)); }
	template<typename A, typename B>
	static std::shared_ptr<Checker<std::pair<A,B>>> sliceStartAndList(std::shared_ptr<Checker<A>> at, std::shared_ptr<Checker<B>> bt){
		return std::shared_ptr<Checker<std::pair<A,B>>>(new SliceStartAndListChecker<A,B>(at,bt)); }
	template<typename A, typename B>
	static std::shared_ptr<Translator<std::pair<A,B>>> pairTrans(std::shared_ptr<Translator<A>> at, std::shared_ptr<Translator<B>> bt){
		return std::shared_ptr<Translator<std::pair<A,B>>>(new PairTranslator<A,B>(at,bt)); }
	template<typename A, typename B>
	static std::shared_ptr<Translator<std::unordered_map<A,B>>> mapConversion(std::shared_ptr<Translator<std::vector<std::pair<A,B>>>> v){
		return std::shared_ptr<Translator<std::unordered_map<A,B>>>(new MapConverter<A,B>(std::move(v))); }
	template<typename A, typename B>
	static std::shared_ptr<Translator<std::unordered_map<A,B>>> mapTrans(std::shared_ptr<Translator<A>> at, std::shared_ptr<Translator<B>> bt){
		return std::shared_ptr<Translator<std::unordered_map<A,B>>>(new MapTranslator<A,B>(at,bt)); }
	template<typename A, typename B>
	static std::shared_ptr<Termer<std::unordered_map<A,B>>> mapTerm(std::shared_ptr<Termer<A>> at, std::shared_ptr<Termer<B>> bt){
		return std::shared_ptr<Termer<std::unordered_map<A,B>>>(new MapTermer<A,B>(at,bt)); }
	template<typename A, typename B>
	static std::shared_ptr<Checker<std::unordered_map<A,B>>> mapCheck(std::shared_ptr<Checker<A>> at, std::shared_ptr<Checker<B>> bt){
		return std::shared_ptr<Checker<std::unordered_map<A,B>>>(new MapChecker<A,B>(at,bt)); }
	static std::shared_ptr<Translator<Term>> identity(){
		return std::shared_ptr<Translator<Term>>(new BlandTranslator()); }
	static std::shared_ptr<Translator<bool>> boolTrans(){
		return std::shared_ptr<Translator<bool>>(new BoolTranslator()); }
	static std::shared_ptr<Checker<bool>> boolCheck(){ return boolTrans(); }
	static std::shared_ptr<Termer<bool>> boolTerm(){ return boolTrans(); }
	static std::shared_ptr<Translator<std::string>> stringTrans(){
		return std::shared_ptr<Translator<std::string>>(new StringTranslator()); }
	static std::shared_ptr<Translator<int>> intTrans(){
		return std::shared_ptr<Translator<int>>(new IntTranslator()); }
	static std::shared_ptr<Checker<std::string>> stringCheck(){
		return std::shared_ptr<Checker<std::string>>(new StringChecker()); }
	static std::shared_ptr<Checker<int>> intCheck(){
		return std::shared_ptr<Checker<int>>(new IntChecker()); }
	static std::shared_ptr<Translator<float>> floatTrans(){
		return std::shared_ptr<Translator<float>>(new FloatTranslator()); }
	static std::shared_ptr<Checker<float>> floatCheck(){
		return std::shared_ptr<Checker<float>>(new FloatChecker()); }
	static std::shared_ptr<Translator<double>> doubleTrans(){
		return std::shared_ptr<Translator<double>>(new DoubleTranslator()); }
	static std::shared_ptr<Checker<double>> doubleCheck(){
		return std::shared_ptr<Checker<double>>(new DoubleChecker()); }
	
	
	template<class T>
	static T findOrDefault(std::vector<Term> const& v, std::shared_ptr<Checker<T>> checker, T d){
		for(Term const& t : v){
			try{
				T ret = checker->check(t);
				return ret;
			}catch(TyperError te){}
		}
		return d;
	}
	
	template<class T>
	static std::shared_ptr<Checker<T>> asChecker(std::shared_ptr<Translator<T>> tt){
		return std::static_pointer_cast<Checker<T>>(tt); }
	template<class T>
	static std::shared_ptr<Termer<T>> asTermer(std::shared_ptr<Translator<T>> tt){
		return std::static_pointer_cast<Termer<T>>(tt); }
}

}//namespace termpose