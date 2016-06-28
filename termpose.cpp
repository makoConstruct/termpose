
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

namespace ctermpose{
	#include "termpose.c"
}

namespace termpose{


template <typename T>
std::vector<T> operator+(const std::vector<T> &A, const std::vector<T> &B)
{
	std::vector<T> AB;
	AB.reserve( A.size() + B.size() );
	AB.insert( AB.end(), A.begin(), A.end() );
	AB.insert( AB.end(), B.begin(), B.end() );
	return AB;
}

template <typename T>
std::vector<T> &operator+=(std::vector<T> &A, const std::vector<T> &B)
{
	A.reserve( A.size() + B.size() );
	A.insert( A.end(), B.begin(), B.end() );
	return A;
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
	static void mimicInterTerm(Term* r, ctermpose::InterTerm* v);
	unsigned estimateLength() const;
	bool isStr() const;
	std::string toString() const;
	std::string prettyPrint() const;
	static inline Term parseMultiline(std::string& s);
	static inline Term parse(std::string& s);
	Term();
	Term(Term&& other);
	Term(Term const& other);
	Term(List other);
	Term(Stri other);
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
		errors += other.errors;
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

void Term::mimicInterTerm(Term* r, ctermpose::InterTerm* v){
	if(ctermpose::isInterStri(v)){
		auto is = (ctermpose::InterStri*)v;
		auto rs = &r->s;
		rs->tag = 1;
		rs->line = is->line;
		rs->column = is->column;
		new (& rs->s) std::string(is->v);
	}else{
		auto is = (ctermpose::InterSeqs*)v;
		auto rl = &r->l;
		rl->tag = 0;
		rl->line = is->line;
		rl->column = is->column;
		new (& rl->l) std::vector<Term>();
		std::vector<Term>& rv = rl->l;
		size_t vl = is->s.len;
		rv.reserve(vl);
		auto mimiced = [is](ctermpose::InterTerm* v){
			Term ntb;
			mimicInterTerm(&ntb, v);
			return ntb;
		};
		for(int i=0; i<vl; ++i){
			rv.emplace_back(mimiced((ctermpose::InterTerm*)(is->s.v[i])));
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

inline Term Term::parseMultiline(std::string& unicodeString){ //note, throws ParserError if there was a syntax error
	//redefining stuff so that we can avoid the middle part of the translation from interterm to ctermpose::term to termpose::term. Musing: If I were REALLY serious I'd cut out the interterms alltogether and do a proper C++ port. Hm. I suppose there might be a way of abstracting over the differences between C and C++ and using the same parsing code for both.
	ctermpose::TermposeParsingError errorOut;
	ctermpose::Parser p;
	ctermpose::initParser(&p);
	ctermpose::parseLengthedToSeqsTakingParserRef(&p, unicodeString.c_str(), unicodeString.size(), &errorOut);
	if(!errorOut.msg){
		Term ret;
		List* lv = (List*)&ret;
		lv->tag = 0;
		lv->line = 0;
		lv->column = 0;
		new(&lv->l) std::vector<Term>();
		std::vector<Term>& outVec = lv->l;
		unsigned rarl = p.rootArBuf.len;
		ctermpose::InterTerm** rarv = (ctermpose::InterTerm**)p.rootArBuf.v;
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



namespace parsingDSL{
	template<typename T>
	using rc = std::shared_ptr<T>;
	
	template<typename T> struct dependent_false: std::false_type {};
	template<typename T> struct Checker{
		virtual T check(Term const& v) = 0; //throws TyperError
	};
	template<typename T> struct Termer{
		virtual Term termify(T const& v) = 0;
	};
	template<typename T> struct Translator : Checker<T>, Termer<T>{
	};
	template<typename T> struct BasicTranslator : Translator<T> {
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
	
	template<typename A, typename B>
	struct PairTranslator : Translator<std::pair<A,B>> {
		rc<Translator<A>> at;
		rc<Translator<B>> bt;
		PairTranslator(rc<Translator<A>> at, rc<Translator<B>> bt):at(at),bt(bt) {}
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
		rc<Translator<A>> at;
		rc<Translator<B>> bt;
		bool ignoreErroneousContent;
		StartAndSequence(rc<Translator<A>> at, rc<Translator<B>> bt, bool ignoreErroneousContent = false):at(at),bt(bt),ignoreErroneousContent(ignoreErroneousContent) {}
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
								errors += e.errors;
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
	
	template<typename B>
	struct LiteralEnsurer : Translator<B> {
		std::string aLiteral;
		rc<Translator<std::pair<std::string,B>>> bt;
		LiteralEnsurer(std::string aLiteral, rc<Translator<std::pair<std::string,B>>> bt):aLiteral(aLiteral), bt(bt) {}
		virtual Term termify(B const& b){
			return bt.termify(std::make_pair(aLiteral, b));
		}
		virtual B check(Term const& v){
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
	};
	
	template<typename A, typename B>
	struct MapConverter : Translator<std::unordered_map<A,B>> {
		rc<Translator<std::vector<std::pair<A,B>>>> st;
		MapConverter(rc<Translator<std::vector<std::pair<A,B>>>> st):st(st) {}
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
		for(T&& iv : v){
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
						errors += e.errors;
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
		rc<Termer<T>> innerTermer;
		SequenceTermer(rc<Termer<T>> it):innerTermer(it) {}
		virtual Term termify(std::vector<T> const& v){
			return sequenceTermify(&*innerTermer, v);
		}
	};
	template<typename T> struct SequenceChecker : Checker<std::vector<T>> {
		rc<Checker<T>> innerChecker;
		bool ignoreErroneousContent;
		SequenceChecker(rc<Checker<T>> ic, bool ignoreErroneousContent = false):innerChecker(ic), ignoreErroneousContent(ignoreErroneousContent) {}
		virtual std::vector<T> check(Term const& t){
			return sequenceCheck(&*innerChecker, ignoreErroneousContent, t);
		}
	};
	template<typename T> struct SequenceTranslator : Translator<std::vector<T>> {
		rc<Translator<T>> innerTranslator;
		bool ignoreErroneousContent = false;
		SequenceTranslator(
			rc<Translator<T>> ic,
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
	struct MapTranslator : Translator<std::unordered_map<A,B>> {
		rc<Translator<A>> at;
		rc<Translator<B>> bt;
		bool ignoreErroneousContent = false;
		MapTranslator(rc<Translator<A>> at, rc<Translator<B>> bt, bool ignoreErroneousContent=false):
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
							A a = at->check(it.l.l[0]);
							try{
								B b = bt->check(it.l.l[1]);
								if(errors.size() == 0){
									results.insert({std::move(a), std::move(b)});
								}
							}catch(TyperError e){
								if(!ignoreErroneousContent){
									errors += e.errors;
								}
							}
						}catch(TyperError e){
							if(!ignoreErroneousContent){
								errors += e.errors;
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
		rc<Translator<T>> tt;
		TaggedSequenceTranslator(
			std::string tag,
			rc<Translator<T>> tt
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
		rc<Checker<T>> tt;
		TaggedSequenceChecker(
			std::string tag,
			rc<Checker<T>> tt
		):tag(tag),tt(tt){}
		virtual std::vector<T> check(Term const& v){
			return checkTaggedSequence(v,tag,&*tt);
		}
	};
	template<typename T>
	struct TaggedSequenceTermer : Termer<std::vector<T>> {
		std::string tag;
		rc<Termer<T>> tt;
		TaggedSequenceTermer(
			std::string tag,
			rc<Termer<T>> tt
		):tag(tag),tt(tt){}
		virtual Term termify(std::vector<T> const& v){
			return termifyTaggedSequence(v,tag,tt);
		}
	};
	
	// struct TagSlicing : Translator<List> {
	// 	std::string tag;
	// 	TagSlicing(
	// 		std::string tag
	// 	):tag(tag){}
	// 	virtual List check(Term const& v){
	// 		if(v.isStr()){
	// 			throw TyperError(v, std::string("expected a sequence starting with \"")+tag+"\"");
	// 		}else{
	// 			std::vector<Term> const& fv = v.l.l;
	// 			if(fv.size() >= 1){
	// 				Term const& tagTerm = fv[0];
	// 				if(tagTerm.isStr()){
	// 					if(tagTerm.s.s != tag){
	// 						throw TyperError(tagTerm, std::string("expected tag to be \"")+tag+"\", but it's "+tagTerm.s.s);
	// 					}else{
	// 						std::vector<Term> outv;
	// 						for(int i=1; i<fv.size(); ++i){
	// 							outv.push_back(fv[i]);
	// 						}
	// 						return List::create(outv, v.s.line + tagTerm.s.s.size(), v.s.column); //caveat, this will give slightly incorrect line numbers for many unicode strings, but I don't know who cares.
	// 					}
	// 				}else{
	// 					throw TyperError(tagTerm, std::string("expected a tag term \"")+tag+"\" here, but it's a list");
	// 				}
	// 			}else{
	// 				throw TyperError(v, std::string("expected a sequence starting with \"")+tag+"\", but the sequence is empty");
	// 			}
	// 		}
	// 	}
	// 	virtual Term termify(List const& v){
	// 		std::vector<Term> cv;
	// 		cv.reserve(v.l.size()+1);
	// 		cv.push_back(Stri::create(std::string(tag)));
	// 		cv += v.l;
	// 		return List::create(cv, v.line, v.column);
	// 	}
	// };
	
	
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
						for(int i=1; i<fv.size(); ++i){
							outv.push_back(fv[i]);
						}
						return inner->check(List::create(outv, v.s.line + tagTerm.s.s.size(), v.s.column)); //caveat, this will give slightly incorrect line numbers for many unicode strings, but I don't know who cares.
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
		rc<Checker<T>> inner;
		TagSlicingChecker(
			std::string tag,
			rc<Checker<T>> inner
		):tag(tag),inner(inner){}
		virtual T check(Term const& v){
			return checkTagSlicing(v,tag,&*inner);
		}
	};
	
	template<class T>
	struct TagSlicing : Translator<T> {
		std::string tag;
		rc<Translator<T>> inner;
		TagSlicing(
			std::string tag,
			rc<Translator<T>> inner
		):tag(tag),inner(inner){}
		virtual List check(Term const& v){
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
				cv += rt.l.l;
			}
			return List::create(std::move(cv));
		}
	};
	
	// I'm not sure I'll ever finish the following functionality. It seems completely impossible to get the return type of a lambda. Anything that would like to will utterly confound gcc. Clean simple functional APIs are impossible.
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

	template<class F, class... Args>
	struct ReductionChecker : Checker<decltype(of_result<F,Args...>())>{
	private:
		template<int... Indices>
		static inline auto map_args_into(
			seq<Indices...>,
			std::tuple<rc<Checker<Args>>...>& args,
			List const& s,
			F& f
		)-> decltype(f(std::declval<Args>()...)){
			return f(std::get<Indices>(args)->check(s.l[Indices])...);
		}
		static auto apply(std::tuple<rc<Checker<Args>>...>& args, List const& s, F& f)-> decltype(f(std::declval<Args>()...)) {
			return map_args_into(typename count_to<sizeof...(Args)>::type(), args, s, f);
		}
		
		std::tuple<rc<Checker<Args>>...> checkers;
		F f;
	public:
		ReductionChecker(std::tuple<rc<Checker<Args>>...> checkers, F f):checkers(checkers),f(f){}
		decltype(of_result<F,Args...>()) check(Term const& v){
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
				return apply(checkers, s, f);
			}
		}
	};
	
	template<typename F, typename... Ts>
	static rc<Checker<decltype(of_result<F,Ts...>())>> combine(F fun, rc<Checker<Ts>>... mappers){
		return rc<Checker<decltype(of_result<F,Ts...>())>>(new ReductionChecker<F,Ts...>(make_tuple(mappers...), fun));
	}
	template<typename T>
	static rc<Translator<std::vector<T>>> sequenceTrans(
		rc<Translator<T>> tt
	){
		return rc<Translator<std::vector<T>>>(new SequenceTranslator<T>(std::move(tt)));
	}
	template<typename T>
	static rc<Checker<std::vector<T>>> sequenceTrans(
		rc<Checker<T>> tt
	){
		return rc<Checker<std::vector<T>>>(new SequenceChecker<T>(std::move(tt)));
	}
	template<typename T>
	static rc<Translator<std::vector<T>>> taggedSequence(
		std::string tag,
		rc<Translator<T>> tt
	){
		return rc<Translator<std::vector<T>>>(new TaggedSequenceTranslator<T>(tag, tt));
	}
	template<typename T>
	static rc<Checker<std::vector<T>>> taggedSequence(
		std::string tag,
		rc<Checker<T>> tt
	){
		return rc<Checker<std::vector<T>>>(new TaggedSequenceChecker<T>(tag, tt));
	}
	template<typename T>
	static rc<Translator<T>> sliceOffTag(std::string tag, rc<Translator<T>> inner){
		return rc<Translator<T>>(new TagSlicing<T>(tag, inner)); }
	template<typename T>
	static rc<Checker<T>> sliceOffTag(std::string tag, rc<Checker<T>> inner){
		return rc<Checker<T>>(new TagSlicingChecker<T>(tag, inner)); }
	template<typename B>
	static rc<Translator<B>> ensureTag(std::string const& literal, rc<Translator<std::pair<std::string,B>>> bt){
		return new rc<Translator<B>>(LiteralEnsurer<B>(literal, bt)); }
	template<typename A, typename B>
	static rc<Translator<std::pair<A,std::vector<B>>>> separateStartAndList(rc<Translator<A>> at, rc<Translator<B>> bt){
		return rc<Translator<std::pair<A,std::vector<B>>>>(new StartAndSequence<A,B>(at,bt)); }
	template<typename A, typename B>
	static rc<Translator<std::pair<A,B>>> pairTrans(rc<Translator<A>> at, rc<Translator<B>> bt){
		return rc<Translator<std::pair<A,B>>>(new PairTranslator<A,B>(at,bt)); }
	template<typename A, typename B>
	static rc<Translator<std::unordered_map<A,B>>> mapConversion(rc<Translator<std::vector<std::pair<A,B>>>> v){
		return rc<Translator<std::unordered_map<A,B>>>(new MapConverter<A,B>(std::move(v))); }
	template<typename A, typename B>
	static rc<Translator<std::unordered_map<A,B>>> mapTrans(rc<Translator<A>> at, rc<Translator<B>> bt){
		return rc<Translator<std::unordered_map<A,B>>>(new MapTranslator<A,B>(at,bt)); }
	static rc<Translator<Term>> identity(){
		return rc<Translator<Term>>(new BlandTranslator()); }
	static rc<Translator<bool>> boolTrans(){
		return rc<Translator<bool>>(new BoolTranslator()); }
	static rc<Checker<bool>> boolCheck(){ return boolTrans(); }
	static rc<Termer<bool>> boolTerm(){ return boolTrans(); }
	static rc<Translator<std::string>> stringTrans(){
		return rc<Translator<std::string>>(new StringTranslator()); }
	static rc<Translator<int>> intTrans(){
		return rc<Translator<int>>(new IntTranslator()); }
	static rc<Checker<std::string>> stringCheck(){
		return rc<Checker<std::string>>(new StringChecker()); }
	static rc<Checker<int>> intCheck(){
		return rc<Checker<int>>(new IntChecker()); }
	static rc<Translator<float>> floatTrans(){
		return rc<Translator<float>>(new FloatTranslator()); }
	
}


}//namespace termpose