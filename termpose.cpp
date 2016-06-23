
#include <iostream>
#include <sstream>
#include <string>
#include <stdexcept>
#include <algorithm>
#include <vector>
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
	static void putErr(std::stringstream& ss, Error& e){
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
	static Term create(std::string&& str);
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
	static Term create(std::vector<Term>&& ov);
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
	Term(Term&& other);
	Term(Term const& other);
	~Term();
private:
	Term(){}
	void baseLineStringify(std::stringstream& ss) const;
	void stringify(std::stringstream& ss) const;
	void prettyPrinting(std::stringstream& sb, unsigned depth, std::string indent, std::string lineEndings, unsigned lineWidth) const;
	static void parseLengthedToSeqs(Term* termOut, std::string& unicodeString, char** errorOut);
	friend Stri;
	friend List;
};

struct TyperError:public TermposeError{
	std::vector<termpose::Error> errors;
	TyperError(Term& t, std::string msg): TyperError(t.l.line,t.l.column,msg){}
	TyperError(std::vector<Error> errors): errors(errors), TermposeError("TyperError"){}
	TyperError(unsigned line, unsigned column, std::string msg):errors({Error{line, column, msg}}), TermposeError("TyperError"){}
	void suck(TyperError& other){
		errors += other.errors;
	}
	virtual char const* what(){
		std::stringstream ss;
		ss<<"TyperError"<<'\n';
		for(Error& e : errors) putErr(ss, e);
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
Term::~Term(){
	if(isStr()) ((Stri*)this)->~Stri();
	else ((List*)this)->~List();
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
Term Stri::create(std::string&& str){
	Term ret;
	Stri* rv = (Stri*)&ret;
	*rv = Stri{1,0,0,str};
	return ret;
}
Term List::create(std::vector<Term>&& ov){
	Term ret;
	List* rv = (List*)&ret;
	*rv = List{0,0,0,ov};
	return ret;
}



namespace parsingDSL{
	
	template<typename T> struct dependent_false: std::false_type {};
	template<typename T> struct Checker{
		virtual T check(Term& v){throw "not implemented";} //throws TyperError
	};
	template<typename T> struct Termer{
		virtual Term termify(T& v){throw "not implemented";}
	};
	template<typename T> struct Translator : Checker<T>, Termer<T>{
	};
	template<typename T> struct BasicTranslator : Translator<T> {
		Checker<T> ck;
		Termer<T> tk;
		virtual T check(Term& v){ return ck.check(v); }
		virtual Term termify(T& v){ return tk.termify(v); }
	};
	
	Term termifyBool(bool& v){
		return Stri::create(v?"true":"false");
	}
	bool checkBool(Term& v){
		if(v.isStr()){
			std::string& st = v.s.s;
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
		virtual Term termify(bool& b){
			return termifyBool(b);
		}
		virtual bool check(Term& v){
			return checkBool(v);
		}
	};
	
	Term termString(std::string& b){
		std::stringstream ss;
		ss << b;
		return Stri::create(ss.str());
	}
	std::string checkString(Term& v){
		if(v.isStr()){
			return v.s.s;
		}else{
			throw TyperError(v, "expected a string here");
		}
	}
	struct StringTermer : Termer<std::string>{ virtual Term termify(std::string& b){
		return termString(b);
	}};
	struct StringChecker : Checker<std::string>{ virtual std::string check(Term& v){
		return checkString(v);
	}};
	struct StringTranslator : Translator<std::string> {
		std::string check(Term& v){ return checkString(v); }
		Term termify(std::string& v){ return termString(v); }
	};
	
	struct BlandTermer : Termer<Term>{ virtual Term termify(Term& b){ return b; }};
	struct BlandChecker : Checker<Term>{ virtual Term check(Term& v){ return v; }};
	struct BlandTranslator : Translator<Term> {
		Term termify(Term& b){ return b; }
		Term check(Term& v){ return v; }
	};
	
	Term termInt(int& b){
		std::stringstream ss;
		ss << b;
		return Stri::create(ss.str());
	}
	int checkInt(Term& v){
		if(v.isStr()){
			std::string& st = v.s.s;
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
	struct IntTermer : Termer<int>{ virtual Term termify(int& b){ return termInt(b); } };
	struct IntChecker : Checker<int>{ virtual int check(Term& v){ return checkInt(v); } };
	struct IntTranslator : Translator<int> {
		virtual Term termify(int& b){ return termInt(b); }
		virtual int check(Term& v){ return checkInt(v); }
	};
	
	
	Term termifyFloat(float& b){
		std::stringstream ss;
		ss << b;
		return Stri::create(ss.str());
	}
	float checkFloat(Term& v){
		if(v.isStr()){
			std::string& st = v.s.s;
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
	struct FloatTermer : Termer<float>{ virtual Term termify(float& b){ return termifyFloat(b); } };
	struct FloatChecker : Checker<float>{ virtual float check(Term& v){ return checkFloat(v); } };
	struct FloatTranslator : Translator<float> {
		virtual Term termify(float& b){ return termifyFloat(b); }
		virtual float check(Term& v){ return checkFloat(v); }
	};
	
	template<typename T>
	Term sequenceTermify(
		Termer<T>* innerTermer,
		std::vector<T>& v
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
		Term& t
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
		std::shared_ptr<Termer<T>> innerTermer;
		SequenceTermer(std::shared_ptr<Termer<T>> it):innerTermer(it) {}
		virtual Term termify(std::vector<T>& v){
			return sequenceTermify(&*innerTermer, v);
		}
	};
	template<typename T> struct SequenceChecker : Checker<std::vector<T>> {
		std::shared_ptr<Checker<T>> innerChecker;
		bool ignoreErroneousContent;
		SequenceChecker(std::shared_ptr<Checker<T>> ic, bool ignoreErroneousContent = false):innerChecker(ic), ignoreErroneousContent(ignoreErroneousContent) {}
		virtual std::vector<T> check(Term& t){
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
		virtual Term termify(std::vector<T>& v){
			return sequenceTermify(&*innerTranslator, v);
		}
		virtual std::vector<T> check(Term& t){
			return sequenceCheck(&*innerTranslator, ignoreErroneousContent, t);
		}
	};
	
	
	template<typename T>
	struct TaggedSequenceTranslator : Translator<std::vector<T>> {
		std::string tag;
		std::shared_ptr<Translator<T>> tt;
		TaggedSequenceTranslator(
			std::string tag,
			std::shared_ptr<Translator<T>> tt
		):tag(tag),tt(tt){}
		virtual std::vector<T> check(Term& v){
			if(v.isStr()){
				throw TyperError(v, std::string("expected a sequence starting with \"")+tag+"\"");
			}else{
				if(v.l.l.size() >= 1){
					std::vector<T> outv;
					for(int i=1; i<v.l.l.size(); ++i){
						outv.push_back(tt->check(outv[i]));
					}
					return outv;
				}else{
					throw TyperError(v, std::string("expected a sequence starting with \"")+tag+"\". Sequence was empty");
				}
			}
		}
		virtual Term termify(std::vector<T>& v){
			std::vector<T> cv;
			cv.reserve(v.size()+1);
			cv.push_back(Stri::create(std::string(tag)));
			for(T& vt: vt){
				cv.push_back(tt.termify(vt));
			}
			return List::create(cv);
		}
	};
	
	// template<typename... T, Out>
	// struct ReductionChecker : Checker<Out>{
	// 	std::function<Out (...T)> f;
	// 	ReductionChecker(std::function<Out (...T)> f):f(f){}
	// 	Out check(Term& v){
	// 		unsigned len = sizeof...(T);
	// 		if(v.isStri()){
	// 			stringstream ss;
	// 			ss << "expected a tuple of length " << len << ". It was a string term.";
	// 			throw new TyperError(t, ss.str());
	// 		}
			
	// 	}
	// };
	
	template<typename T>
	static std::shared_ptr<Translator<std::vector<T>>> sequence(
		std::shared_ptr<Translator<T>> tt
	){
		return std::shared_ptr<Translator<std::vector<T>>>(new SequenceTranslator<T>(std::move(tt)));
	}
	template<typename T>
	static std::shared_ptr<Translator<std::vector<T>>> taggedSequence(
		std::string tag,
		std::shared_ptr<Translator<T>> tt
	){
		return std::shared_ptr<Translator<std::vector<T>>>(new TaggedSequenceTranslator<T>(tag, tt));
	}
	static std::shared_ptr<Translator<bool>> boolTranslator(){
		return std::shared_ptr<Translator<bool>>(new BoolTranslator()); }
	static std::shared_ptr<Checker<bool>> boolChecker(){ return boolTranslator(); }
	static std::shared_ptr<Termer<bool>> boolTermer(){ return boolTranslator(); }
	static std::shared_ptr<Translator<std::string>> stringTranslator(){
		return std::shared_ptr<Translator<std::string>>(new StringTranslator()); }
	static std::shared_ptr<Translator<int>> intTranslater(){
		return std::shared_ptr<Translator<int>>(new IntTranslator()); }
	static std::shared_ptr<Translator<float>> floatTranslator(){
		return std::shared_ptr<Translator<float>>(new FloatTranslator()); }
	
}


}//namespace termpose