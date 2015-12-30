
#include <iostream>
#include <sstream>
#include <string>
#include <stdexcept>
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


struct Error{
	unsigned line, column;
	std::string msg;
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
struct TyperError:public TermposeError{
	std::vector<termpose::Error> errors;
	TyperError(Term& t, std::string msg):TermposeError("TyperError"){}
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
	char* errorOut = nullptr;
	ctermpose::Parser p;
	ctermpose::initParser(&p);
	ctermpose::parseLengthedToSeqsTakingParserRef(&p, unicodeString.c_str(), unicodeString.size(), &errorOut);
	if(!errorOut){
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
		std::string msg(errorOut);
		ctermpose::destroyStr(errorOut);
		throw ParserError(0,0,msg);
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
	struct Error{
		unsigned line;
		unsigned column;
		std::string msg;
		Error(unsigned line, unsigned column, std::string msg): line(line), column(column), msg(msg) {}
	};
	template<typename T> struct Checker{
		T check(Term& v); //throws TyperError
	};
	template<typename T> struct Termer{
		virtual Term termify(T& v);
	};
	template<typename T> struct Translator : Checker<T>, Termer<T>{
	};
	
	template<> struct Termer<bool>{ virtual Term termify(bool& b){
			return Stri::create(b?"true":"false");
	}};
	template<> struct Checker<bool>{ virtual bool check(Term& v){
		if(v.isStr()){
			std::string& st = v.s.s;
			if(st == "true" || st == "⊤") return true;
			else if(st == "false" || st == "⊥") return false;
			else throw TyperError(v, "expected a bool here");
		}else{
			throw TyperError(v, "expected a bool here");
		}
	}};
	
	template<> struct Termer<int>{ virtual Term termify(int& b){
			std::stringstream ss;
			ss << b;
			return Stri::create(ss.str());
	}};
	template<> struct Checker<int>{ virtual int check(Term& v){
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
	}};
	
	template<typename T> struct Termer<std::vector<T>> {
		Termer<T> innerTermer;
		Termer(Termer<T> it):innerTermer(it) {}
		Termer(){}
		virtual Term termify(std::vector<T>& v){
			std::vector<Term> ov;
			for(T& iv : v){
				ov.push_back(innerTermer.termify(iv));
			}
			return List::create(std::move(ov));
		}
	};
	template<typename T> struct Checker<std::vector<T>> {
		Checker<T> innerChecker;
		bool ignoreErroneousContent;
		Checker(Checker<T> ic, bool ignoreErroneousContent = false):innerChecker(ic), ignoreErroneousContent(ignoreErroneousContent) {}
		Checker(bool ignoreErroneousContent = false): ignoreErroneousContent(ignoreErroneousContent) {}
		virtual T check(Term& t){
			if(t.isStr()){
				return throw TyperError(t, "expected a sequence here");
			}else{
				List& tl = (List&)t;
				std::vector<T> results;
				std::vector<Error> errors;
				for(Term& it : tl.l){
					try{
						T res = innerChecker.check(it);
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
					throw TyperError(move(errors));
				}else{
					return results;
				}
			}
		}
	};
	
	
	//the following could have been a magical function that takes a type and automatically composes a method for doing the translation, but apparently BoolTermer:Termer<bool> does not count as a bool specialization of Termer, so it wont work unless I'm willing to define things differently.
	// template<typename T> Term termify(T& v){
	// 	static_assert(!is_abstract<Termer<T>>::value, "no Termer specialization has been defined for the given type");
	// 	Termer<T> vt;
	// 	return vt.termify
	// }
	
	static std::unique_ptr<Translator<bool>> boolTranslater(){ return std::unique_ptr<Translator<bool>>(new Translator<bool>()); }
	static std::unique_ptr<Checker<bool>> boolChecker(){ return boolTranslater(); }
	static std::unique_ptr<Termer<bool>> boolTermer(){ return boolTranslater(); }
	
}


}//namespace termpose