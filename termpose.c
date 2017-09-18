#include "termpose.h"

//thanks suckless for this utf-8 stuff:
#define BETWEEN(x, a, b)  ((a) <= (x) && (x) <= (b))
#define UTF_INVALID  0xFFFD
#define UTF_SIZ  4
#define LEN(a)  (sizeof(a) / sizeof(a)[0])
typedef uint32_t Rune;
typedef unsigned char uchar;
static uchar utfbyte[UTF_SIZ + 1] = {0x80,    0, 0xC0, 0xE0, 0xF0};
static uchar utfmask[UTF_SIZ + 1] = {0xC0, 0x80, 0xE0, 0xF0, 0xF8};
static Rune utfmin[UTF_SIZ + 1] = {       0,    0,  0x80,  0x800,  0x10000};
static Rune utfmax[UTF_SIZ + 1] = {0x10FFFF, 0x7F, 0x7FF, 0xFFFF, 0x10FFFF};

size_t utf8decode(char const *c, Rune *u, size_t clen);
Rune utf8decodebyte(char c, size_t *i);
size_t utf8encode(Rune u, char *c);
char utf8encodebyte(Rune u, size_t i);
char * utf8strchr(char *s, Rune u);
size_t utf8validate(Rune *u, size_t i);


size_t utf8decode(char const *c, Rune *u, size_t clen){
	size_t i, j, len, type;
	Rune udecoded;
	*u = UTF_INVALID;
	if (!clen) return 0;
	udecoded = utf8decodebyte(c[0], &len);
	if (!BETWEEN(len, 1, UTF_SIZ))
		return 1;
	for (i = 1, j = 1; i < clen && j < len; ++i, ++j) {
		udecoded = (udecoded << 6) | utf8decodebyte(c[i], &type);
		if (type != 0) return j;
	}
	if (j < len) return 0;
	*u = udecoded;
	utf8validate(u, len);
	return len;
}

Rune utf8decodebyte(char c, size_t *i){
	for (*i = 0; *i < LEN(utfmask); ++(*i)){
		if (((uchar)c & utfmask[*i]) == utfbyte[*i]) return (uchar)c & ~utfmask[*i];
	}
	return 0;
}

size_t utf8encode(Rune u, char *c){
	size_t len, i;
	len = utf8validate(&u, 0);
	if (len > UTF_SIZ) return 0;
	for (i = len - 1; i != 0; --i) {
		c[i] = utf8encodebyte(u, 0);
		u >>= 6;
	}
	c[0] = utf8encodebyte(u, len);
	return len;
}

char utf8encodebyte(Rune u, size_t i){
	return utfbyte[i] | (u & ~utfmask[i]);
}

char * utf8strchr(char *s, Rune u){
	Rune r;
	size_t i, j, len;
	len = strlen(s);
	for (i = 0, j = 0; i < len; i += j) {
		if (!(j = utf8decode(&s[i], &r, len - i))) break;
		if (r == u) return &(s[i]);
	}
	return NULL;
}

size_t utf8validate(Rune *u, size_t i){
	if (!BETWEEN(*u, utfmin[i], utfmax[i]) || BETWEEN(*u, 0xD800, 0xDFFF)) *u = UTF_INVALID;
	for (i = 1; *u > utfmax[i]; ++i) ;
	return i;
}







uint32_t ceilPowerTwo(uint32_t v){
	v -= 1;
	v = v | (v >> 16);
	v = v | (v >> 8);
	v = v | (v >> 4);
	v = v | (v >> 2);
	v = v | (v >> 1);
	return v+1;
}
void* makeCopy(void const* in, size_t len){
	void* ret = malloc(len);
	memcpy(ret, in, len);
	return ret;
}

char* strCopy(char const* v){ return (char*)makeCopy(v, strlen(v)+1); }

void destroyStr(char* str){ free(str); }

void freeIfSome(void* v){ if(v != NULL) free(v); }




//buffers
typedef struct CharBuffer {
	char* v;
	uint32_t len;
	uint32_t cap;
} CharBuffer;
void initCharBuffer(CharBuffer* cb, uint32_t initialLen, uint32_t initialCapacity){
	cb->v = (char*)malloc(initialCapacity*sizeof(char));
	cb->len = initialLen;
	cb->cap = initialCapacity;
}
CharBuffer* newCharBuf(uint32_t initialLen){
	CharBuffer* ret = (CharBuffer*)malloc(sizeof(CharBuffer));
	initCharBuffer(ret,initialLen,ceilPowerTwo(initialLen));
	return ret;
}
void reserveCapacityAfterLength(CharBuffer* t, uint32_t extraCap){
	uint32_t finalCap = t->len + extraCap;
	if(t->cap < finalCap){
		uint32_t newCap = ceilPowerTwo(finalCap);
		t->v = (char*)realloc(t->v, newCap);
		t->cap = newCap;
	}
}
void addChar(CharBuffer* t, char c){
	uint32_t tl = t->len;
	if(tl >= t->cap){
		uint32_t newCap = t->cap*2;
		t->v = (char*)realloc(t->v, newCap*sizeof(char));
		t->cap = newCap;
	}
	t->v[tl] = c;
	t->len = tl+1;
}
void addStr(CharBuffer* t, char const* c){
	uint32_t i = 0;
	while(c[i] != 0){
		addChar(t, c[i]);
		++i;
	}
}
void addU32(CharBuffer* t, uint32_t n){
	const uint32_t ULEN = 12;
	reserveCapacityAfterLength(t, ULEN); //ensure we can take the thing no matter how long it is (it will never be longer than ULEN)
	t->len += sprintf(t->v + t->len, "%u", n);
}
void addString(CharBuffer* t, char* c, uint32_t len){
	reserveCapacityAfterLength(t, len);
	memcpy(t->v + t->len, c, len*sizeof(char));
	t->len += len;
}
void addRune(CharBuffer* t, Rune c){
	reserveCapacityAfterLength(t, UTF_SIZ);
	size_t lengthAdded = utf8encode(c, t->v + t->len);
	t->len += lengthAdded;
	// return lengthAdded; //considered returning this for error detection, but in practice it's better to deal with bad runes when you get them rather than when you process them.
}
void clearCharBuffer(CharBuffer* t){
	t->len = 0;
}
char* rendStr(CharBuffer* t){ //drains the CharBuffer and yeilds its contents, null-terminated
	addChar(t, 0);
	return (char*)realloc(t->v, sizeof(char)*t->len);
}
char* copyLengthedStringAndClear(CharBuffer* t, uint32_t* rlen){ //alert, returns NULL if empty
	uint32_t tl = t->len;
	*rlen = tl;
	t->len = 0;
	if(tl){
		char* out = (char*)malloc((tl)*sizeof(char));
		memcpy(out, t->v, tl*sizeof(char));
		return out;
	}else{
		return NULL;
	}
}
char* copyOutNullTerminatedString(CharBuffer* t){
	char* out = (char*)malloc((t->len+1)*sizeof(char));
	memcpy(out, t->v, t->len*sizeof(char));
	out[t->len] = 0;
	t->len = 0;
	return out;
}
void drainCharBuffer(CharBuffer* t){
	free(t->v);
}


void initPrettyCoding(TermposeCoding* tc){
	tc->openingBracket = '(';
	tc->closingBracket = ')';
	tc->pairingChar = ':';
	tc->indent = strCopy("  ");
	tc->indentIsStrict = false;
}
void initBestCoding(TermposeCoding* tc){
	tc->openingBracket = '[';
	tc->closingBracket = ']';
	tc->pairingChar = ';';
	tc->indent = strCopy("\t");
	tc->indentIsStrict = false;
}
void initCliCoding(TermposeCoding* tc){
	tc->openingBracket = '['; //because '(' would need to be escaped in bash
	tc->closingBracket = ']';
	tc->pairingChar = '=';
	tc->indent = strCopy("  ");
	tc->indentIsStrict = false;
}
bool validateCoding(TermposeCoding const* tc){ //more for documentation than anything. If your coding violates this function don't expect things to work right :P
	if(!(
		(tc->openingBracket == '(' && tc->closingBracket == ')') ||
		(tc->openingBracket == '[' && tc->closingBracket == ']') ||
		(tc->openingBracket == '{' && tc->closingBracket == '}')
	)){
		return false;
	}
	
	uint32_t indentl = strlen(tc->indent);
	if(indentl > 0){
		if(indentl > 16){ //nope.
			return false;
		}
		char indentc = tc->indent[0];
		if(indentc == '\t'){
			if(indentl > 1){ //fuck no.
				return false;
			}
		}else if(indentc == ' '){
			for(int i=0; i<indentl; ++i){
				if(tc->indent[i] != ' '){
					return false;
				}
			}
		}else{ //wtf?
			return false;
		}
	}else{
		return false;
	}
	return true;
}
void copyCoding(TermposeCoding* t, TermposeCoding const* s){
	t->openingBracket = s->openingBracket;
	t->closingBracket = s->closingBracket;
	t->pairingChar = s->pairingChar;
	t->indentIsStrict = s->indentIsStrict;
	t->indent = strCopy(s->indent);
}

void initDefaultCoding(TermposeCoding* tc){ initPrettyCoding(tc); }
void drainCoding(TermposeCoding* tc){
	free(tc->indent);
}


typedef struct PtrBuffer{
	void** v;
	uint32_t len;
	uint32_t cap;
} PtrBuffer;
void initPtrBuffer(PtrBuffer* cb, uint32_t initialLen, uint32_t initialCapacity){
	cb->v = (void**)malloc(initialCapacity*sizeof(void*));
	cb->len = initialLen;
	cb->cap = initialCapacity;
}
void initEmptyPtrBuffer(PtrBuffer* cb, uint32_t initialCapacity){
	cb->v = (void**)malloc(initialCapacity*sizeof(void*));
	cb->len = 0;
	cb->cap = initialCapacity;
}
PtrBuffer* newPtrBuf(uint32_t initialLen){
	PtrBuffer* ret = (PtrBuffer*)malloc(sizeof(PtrBuffer));
	initPtrBuffer(ret,initialLen,ceilPowerTwo(initialLen));
	return ret;
}
void addPtr(PtrBuffer* t, void* c){
	uint32_t tl = t->len;
	if(tl >= t->cap){
		uint32_t newCap = t->cap*2;
		t->v = (void**)realloc(t->v, newCap*sizeof(void*));
		t->cap = newCap;
	}
	t->v[tl] = c;
	t->len = tl+1;
}
void* popPtr(PtrBuffer* t){ //note, does not downsize the array, but why would you
	uint32_t lmo = t->len - 1;
	void* ret = t->v[lmo];
	t->len = lmo;
	return ret;
}
void clearPtrBuffer(PtrBuffer* t){
	t->len = 0;
}
void drainPtrBuffer(PtrBuffer* t){
	free(t->v);
}
void* firstPtr(PtrBuffer* t){ return t->v[0]; }
void* lastPtr(PtrBuffer* t){ return t->v[t->len - 1]; }


typedef struct LineDetail{
	uint32_t indentationLength;
	PtrBuffer* tailestTermSequence; //[InterTerm*]
} LineDetail;
typedef struct LineBuffer{
	LineDetail* v;
	uint32_t len;
	uint32_t cap;
} LineBuffer;
void initLineBuffer(LineBuffer* cb, uint32_t initialLen, uint32_t initialCapacity){
	cb->v = (LineDetail*)malloc(initialCapacity*sizeof(LineDetail));
	cb->len = initialLen;
	cb->cap = initialCapacity;
}
LineBuffer* newLineBuf(uint32_t initialLen){
	LineBuffer* ret = (LineBuffer*)malloc(sizeof(LineBuffer));
	initLineBuffer(ret,initialLen,ceilPowerTwo(initialLen));
	return ret;
}
LineDetail* emplaceNewLine(LineBuffer* t, uint32_t indentationLength, PtrBuffer* tailestTermSequence){
	uint32_t tl = t->len;
	if(tl >= t->cap){
		uint32_t newCap = t->cap*2;
		t->v = (LineDetail*)realloc(t->v, newCap*sizeof(LineDetail));
		t->cap = newCap;
	}
	LineDetail* ret = t->v + tl;
	ret->indentationLength = indentationLength;
	ret->tailestTermSequence = tailestTermSequence;
	t->len = tl+1;
	return ret;
}
void popLine(LineBuffer* t){ //note, does not downsize the array, but why would you
	t->len -= 1;
}
void drainLineBuffer(LineBuffer* t){
	free(t->v);
}
LineDetail* firstLine(LineBuffer* t){ return &t->v[0]; }
LineDetail* lastLine(LineBuffer* t){ return &t->v[t->len - 1]; }









void initSeqs(Term_t* ret, Term_t* s, uint32_t nTerms, uint32_t line, uint32_t column){
	ret->tag = 0;
	ret->len = nTerms;
	ret->line = line;
	ret->column = column;
	ret->con = s;
}
Term_t* newSeqs(Term_t* s, uint32_t nTerms, uint32_t line, uint32_t column){
	Term_t* ret = (Term_t*)malloc(sizeof(Term_t));
	initSeqs(ret, s, nTerms, line, column);
	return ret;
}
Term_t* newStri(char* v, uint32_t length, uint32_t line, uint32_t column){
	Term_t* ret = (Term_t*)malloc(sizeof(Term_t));
	ret->tag = 1;
	ret->len = length;
	ret->line = line;
	ret->column = column;
	ret->str = v;
	return ret;
}
uint8_t isStr(Term_t const* v){ return *(uint8_t*)v; } 
void drainTerm(Term_t* v){
	Term_t* sv = v->con; //doesn't so much assume that this is a list term as it just doesn't matter for the free that comes later
	if(!isStr(v)){
		uint32_t sl = v->len;
		for(int i=0; i<sl; ++i){
			drainTerm(sv+i);
		}
	}
	free(sv);
}
void destroyTerm(Term_t* v){
	drainTerm(v);
	free(v);
}

//term stringification:
bool escapeIsNeeded(char* v, size_t len, TermposeCoding const* coding){
	for(int i=0; i<len; ++i){
		char c = v[i];
		switch(c){
			case ' ': return true; break;
			case '\t': return true; break;
			case '\n': return true; break;
			case '\r': return true; break;
		}
		if(c == coding->openingBracket) return true;
		else if(c == coding->pairingChar) return true;
		else if(c == coding->closingBracket) return true;
	}
	return false;
}
void escapeSymbol(CharBuffer* outUnicode, char* in, uint32_t len, TermposeCoding const* coding){
	for(int i=0; i<len; ++i){
		char c = in[i];
		switch(c){
			case '\\':
				addChar(outUnicode, '\\');
				addChar(outUnicode, '\\');
			break;
			case '"':
				addChar(outUnicode, '\\');
				addChar(outUnicode, '"');
			break;
			case '\t':
				addChar(outUnicode, '\\');
				addChar(outUnicode, 't');
			break;
			case '\n':
				addChar(outUnicode, '\\');
				addChar(outUnicode, 'n');
			break;
			case '\r':
				addChar(outUnicode, '\\');
				addChar(outUnicode, 'r');
			break;
			default:
				addChar(outUnicode, c);
		}
	}
}
void stringifyingStri(CharBuffer* cb, Term_t const* v, TermposeCoding const* coding){
	uint32_t vl = v->len;
	char* s = v->str;
	if(escapeIsNeeded(s, vl, coding)){
		addChar(cb, '"');
		escapeSymbol(cb, s, vl, coding);
		addChar(cb, '"');
	}else{
		addString(cb, s, vl);
	}
}
void stringifyingTerm(CharBuffer* cb, Term_t const* t, TermposeCoding const* coding);
void stringifyingSeqs(CharBuffer* cb, Term_t const* v, TermposeCoding const* coding){
	addChar(cb, coding->openingBracket);
	uint32_t tl = v->len;
	Term_t* ts = v->con;
	if(tl >= 1){
		stringifyingTerm(cb, ts, coding);
	}
	for(int i=1; i<tl; ++i){
		addChar(cb, ' ');
		stringifyingTerm(cb, ts+i, coding);
	}
	addChar(cb, coding->closingBracket);
}
void stringifyingTerm(CharBuffer* cb, Term_t const* t, TermposeCoding const* coding){
	if(isStr(t)){
		stringifyingStri(cb, t, coding);
	}else{
		stringifyingSeqs(cb, t, coding);
	}
}
char* stringifyTermWithCoding(Term_t const* t, TermposeCoding const* coding){ //returns null if one of the strings contained \invalid unicode.
	CharBuffer cb;
	initCharBuffer(&cb, 0, 128);
	stringifyingTerm(&cb, t, coding);
	return rendStr(&cb);
}
char* stringifyTerm(Term_t const* t){ //returns null if one of the strings contained invalid unicode.
	TermposeCoding coding;
	initDefaultCoding(&coding);
	return stringifyTermWithCoding(t, &coding);
}


// struct ParsingException{
// 	char* msg;
// 	uint32_t line;
// 	uint32_t column;
// };

typedef struct InterSeqs{ //allocated into InterTerms
	uint8_t tag/*=0*/;
	uint32_t line;
	uint32_t column;
	PtrBuffer s;
} InterSeqs;
typedef struct InterStri{ //allocated into InterTerms
	uint8_t tag/*=1*/;
	uint32_t line;
	uint32_t column;
	char* v;
} InterStri;

typedef union InterTerm{ InterSeqs seqs; InterStri stri; } InterTerm;
bool isInterStri(InterTerm* v){ return *(bool*)v; }
void initInterSeqs(InterTerm* ret, uint32_t line, uint32_t column){
	ret->seqs.tag = 0;
	ret->seqs.line = line;
	ret->seqs.column = column;
	initPtrBuffer(&ret->seqs.s,0,4);
}
InterSeqs* newInterSeqs(uint32_t line, uint32_t column){
	InterTerm* ret = (InterTerm*)malloc(sizeof(InterTerm));
	initInterSeqs(ret,line,column);
	return (InterSeqs*)ret;
}
InterStri* newInterStri(uint32_t line, uint32_t column, char* v){
	InterTerm* ret = (InterTerm*)malloc(sizeof(InterTerm));
	ret->stri.tag = 1;
	ret->stri.line = line;
	ret->stri.column = column;
	ret->stri.v = v;
	return &ret->stri;
}
void drainInterStri(InterStri* t){
	free(t->v);
}
void drainInterTerm(InterTerm* t);
void drainInterSeqs(InterSeqs* t){
	PtrBuffer* ts = &t->s;
	uint32_t tsl = ts->len;
	InterTerm** tsv = (InterTerm**)ts->v;
	for(int i=0; i<tsl; ++i){
		InterTerm* it = tsv[i];
		drainInterTerm(it);
		free(it);
	}
	drainPtrBuffer(ts);
}
void drainInterTerm(InterTerm* t){
	if(isInterStri(t)){
		drainInterStri(&t->stri);
	}else{
		drainInterSeqs(&t->seqs);
	}
}

struct Parser;

// parser mode. PF is short for "processing funnel"
typedef void (*PF)(struct Parser*, bool, Rune);

typedef struct Parser{
	CharBuffer stringBuffer;
	char* previousIndentation;  uint32_t previousIndentationLength;
	char* salientIndentation;  uint32_t salientIndentationLength;
	bool hasFoundLine;
	TermposeCoding coding;
	PtrBuffer rootArBuf; //[InterTerm*]
	PtrBuffer parenTermStack; //[InterSeqs*]
	uint32_t line;
	uint32_t column;
	uint32_t index;
	InterTerm* lastAttachedTerm; //the most prominent last term, not necessarily the last attached. If a paren just closed, it's the one whose paren just closed.
	PF currentMode;
	PtrBuffer modes; //[PF]
	TermposeParsingError error;
	LineBuffer indentStack; //indentation length, tailestTermSequence. It is enough to keep track of length. We can derive ∀(a,b∈Line.Indentation) a.length = b.length → a = b from the prefix checking that is done before adding to the stack
	CharBuffer multilineStringIndentBuffer;
	char* multilineStringsIndent;  uint32_t multilineStringsIndentLength;
	PtrBuffer* containsImmediateNext; //[InterTerm*]
} Parser;


void updateStateAboutInterTermMove(Parser* p, InterTerm* to, InterTerm* from);
void dropSeqLayerIfSole(Parser* p, InterSeqs* pt){
	assert(pt->tag == 0);
	PtrBuffer* arb = &pt->s;
	if(arb->len == 1){
		InterTerm* soleEl = (InterTerm*)arb->v[0];
		drainPtrBuffer(arb);
		memcpy(pt, soleEl, sizeof(InterTerm));
		free(soleEl);
		updateStateAboutInterTermMove(p, (InterTerm*)pt, soleEl);
	}
}
void exudeSeqs(Parser* p, InterTerm* pi){ //pi becomes interSeqs, its previous content is copied out and linked as its new first element.
	assert( pi->seqs.line == pi->stri.line && pi->seqs.column == pi->stri.column ); //making sure that ti doesn't matter which interpretation of the union type we use.
	uint32_t l = pi->seqs.line;
	uint32_t c = pi->seqs.column;
	InterTerm* nit = (InterTerm*)malloc(sizeof(InterTerm));
	memcpy(nit, pi, sizeof(InterTerm));
	initInterSeqs(pi, l,c);
	addPtr(&pi->seqs.s, nit);
	updateStateAboutInterTermMove(p, nit, pi);
}


void termMimicInterterm(Term_t* t, InterTerm* v){
	if(isInterStri(v)){
		t->tag = 1;
		char* tsv = v->stri.v;
		uint32_t stl = strlen(tsv);
		t->str = (char*)makeCopy(tsv, (stl+1)*sizeof(char));
		t->len = stl;
		t->line = v->stri.line;
		t->column = v->stri.column;
	}else{
		t->tag = 0;
		uint32_t slen = v->seqs.s.len;
		Term_t* innerTerms = (Term_t*)malloc(slen*sizeof(Term_t));
		t->con = innerTerms;
		t->len = slen;
		t->line = v->seqs.line;
		t->column = v->seqs.column;
		InterTerm** sarr = (InterTerm**)v->seqs.s.v;
		for(int i=0; i<slen; ++i){
			termMimicInterterm(innerTerms+i, sarr[i]);
		}
	}
}

void initParserWithCoding(Parser* p, TermposeCoding const* coding){
	initCharBuffer(&p->stringBuffer,0,256);
	p->previousIndentation = NULL;  p->previousIndentationLength = 0;
	p->salientIndentation = NULL;  p->salientIndentationLength = 0;
	p->hasFoundLine = false;
	copyCoding(&p->coding, coding);
	initPtrBuffer(&p->rootArBuf,0,32);
	initPtrBuffer(&p->parenTermStack,0,32);
	p->line = 1;
	p->column = 1;
	p->index = 0;
	p->lastAttachedTerm = NULL;
	p->currentMode = NULL;
	initPtrBuffer(&p->modes,0,24);
	p->error.msg = NULL;
	initLineBuffer(&p->indentStack,0,32);
	emplaceNewLine(&p->indentStack, 0, &p->rootArBuf);
	initCharBuffer(&p->multilineStringIndentBuffer,0,128);
	p->multilineStringsIndent = NULL;  p->multilineStringsIndentLength = 0;
	p->containsImmediateNext = NULL;
}
void initParser(Parser* p){
	TermposeCoding coding;
	initDefaultCoding(&coding);
	initParserWithCoding(p, &coding);
}
void drainParser(Parser* p){
	drainCharBuffer(&p->stringBuffer);
	freeIfSome(p->previousIndentation);  p->previousIndentationLength = 0;
	freeIfSome(p->salientIndentation);  p->salientIndentationLength = 0;
	p->hasFoundLine = false;
	drainCoding(&p->coding);
	//destroy the interterms
	PtrBuffer* rab = &p->rootArBuf;
	uint32_t rlen = rab->len;
	for(int i=0; i<rlen; ++i){
		InterTerm* it = (InterTerm*)rab->v[i];
		drainInterTerm(it);
		free(it);
	}
	drainPtrBuffer(rab);
	drainPtrBuffer(&p->parenTermStack);
	drainPtrBuffer(&p->modes);
	drainLineBuffer(&p->indentStack);
	drainCharBuffer(&p->multilineStringIndentBuffer);
	freeIfSome(p->multilineStringsIndent);  p->multilineStringsIndentLength = 0;
	p->containsImmediateNext = NULL;
}

//general notes:
//the parser consists of a loop that pumps chars from the input string into whatever the current Mode is. Modes jump from one to the next according to cues, sometimes forwarding the cue onto the new mode before it gets any input from the input loop. There is a mode stack, but it is rarely used. Modes are essentially just a (Bool, Char) => Unit (named 'PF', short for Processing Funnel(JJ it's legacy from when they were Partial Functions)) where the Bool is on whenn the input has reached the end of the file. CR LFs ("\r\n"s) are filtered to a single LF (This does mean that a windows formatted file will have unix line endings when parsing multiline strings, that is not a significant issue, ignoring the '\r's will probably save more pain than it causes and the escape sequence \r is available if they're explicitly desired).
//terms are attached through PointsInterTerms, so that the term itself can change, update its PointsInterTerm, and still remain attached. I had tried delaying the attachment of a term until it was fully formed, but this turned out to be a terrible way of doing things.

// key aspects of global state to regard:
// stringBuffer:StringBuilder    where indentation and symbols are collected and cut off
// indentStack:ArrayBuffer[(Int, ArrayBuffer[InterTerm])]    encodes the levels of indentation we've traversed and the parent container for each level
// parenStack:ArrayBuffer[ArrayBuffer[InterTerm]]    encodes the levels of parens we've traversed and the container for each level. The first entry is the root line, which has no parens.


void updateStateAboutInterTermMove(Parser* p, InterTerm* to, InterTerm* from){ //there are only a few key references which need to be updated when a move occurs
	if(p->lastAttachedTerm == from) p->lastAttachedTerm = to;
	if(p->containsImmediateNext == &from->seqs.s) p->containsImmediateNext = &to->seqs.s;
	InterTerm** ptsv = (InterTerm**)p->parenTermStack.v;
	uint32_t ptsl = p->parenTermStack.len;
	for(int i=0; i<ptsl; ++i){
		if(ptsv[i] == from){
			ptsv[i] = to;
			break;
		}
	}
}
void transition(Parser* p, PF nm){ p->currentMode = nm; }
void pushMode(Parser* p, PF nm){
	addPtr(&p->modes, (void*)p->currentMode);
	transition(p, nm);
}
InterSeqs* interSq(Parser* p){ return newInterSeqs(p->line, p->column); }
InterStri* interSt(Parser* p, char* c){ return newInterStri(p->line, p->column, c); }
void popMode(Parser* p){
	p->currentMode = (PF)popPtr(&p->modes);
}
void lodgeErrorAt(Parser* p, uint32_t l, uint32_t c, char const* message){
	p->error.line = l;
	p->error.column = c;
	p->error.msg = message;
}
void lodgeError(Parser* p, char const* message){
	lodgeErrorAt(p, p->line, p->column, message);
}
PtrBuffer* receiveForemostRecepticle(Parser* p){ //note side effects
	if(p->containsImmediateNext != NULL){
		PtrBuffer* ret = p->containsImmediateNext;
		p->containsImmediateNext = NULL;
		return ret;
	}else{
		if(p->parenTermStack.len == 0){
			InterSeqs* rootParenLevel = interSq(p);
			addPtr(&p->parenTermStack, rootParenLevel);
			addPtr(lastLine(&p->indentStack)->tailestTermSequence, rootParenLevel);
		}
		return &((InterTerm*)lastPtr(&p->parenTermStack))->seqs.s;
	}
}
void attach(Parser* p, InterTerm* t){
	addPtr(receiveForemostRecepticle(p), t);
	p->lastAttachedTerm = t;
}
InterStri* finishTakingSymbolAndAttach(Parser* p){
	InterStri* newSt = interSt(p, copyOutNullTerminatedString(&p->stringBuffer));
	attach(p, (InterTerm*)newSt);
	return newSt;
}
void finishTakingIndentationAndAdjustLineAttachment(Parser* p){
	//Iff there is no indented content or that indented content falls within an inner paren(iff the parenTermStack is longer than one), and the line only has one item at root, the root element in the parenstack should drop a seq layer so that it is just that element. In all other case, leave it as a sequence.
	
	freeIfSome(p->previousIndentation);
	p->previousIndentation =       p->salientIndentation;
	p->previousIndentationLength = p->salientIndentationLength;
	p->salientIndentation = copyLengthedStringAndClear(
		&p->stringBuffer,
		&p->salientIndentationLength
	);
	
	if(p->hasFoundLine){
		if(p->salientIndentationLength > p->previousIndentationLength){
			//only in this case  p->salientIndentationLength > p->previousIndentationLength && p->parenTermStack.len == 0 && p->containsImmediateNext == NULL && multiple entries on the root line || multiple entries on the root line,  is an extra root line seqs needed
			if( strncmp(
				p->previousIndentation,
				p->salientIndentation,
				p->previousIndentationLength
			) != 0 ){ //if the shorter (previousIndentationLength) does not prefix the longer
				lodgeErrorAt(p, p->line - p->salientIndentationLength, p->column, "inconsistent indentation at");
				return;
			}
			//ensure that the indent is valid for this parse's coding
			if(p->coding.indentIsStrict){
				char const* indent = p->coding.indent;
				uint32_t indentl = strlen(indent);
				char indentc = indent[0];
				//require correct characters
				for(int i=p->previousIndentationLength; i<p->salientIndentationLength; ++i){
					if(p->salientIndentation[i] != indentc){
						if(indentc == '\t'){
							lodgeError(p, "No spaces allowed in indentation");
						}else{
							lodgeError(p, "No tabs allowed in indentation");
						}
						return;
					}
				}
				//require correct length
				if(p->salientIndentationLength - p->previousIndentationLength != indentl){
					char msg[128];
					if(indentl > 1){
						char const* indentCharName = (indentc == ' ')? "spaces" : "tabs";
						sprintf(msg, "an indentation style requirement applies. Indentation must consist of %d %s.", indentl, indentCharName);
					}else{
						char const* indentCharName = (indentc == ' ')? "space" : "tab";
						sprintf(msg, "an indentation style requirement applies. Indentation must consist of one %s.", indentCharName);
					}
					lodgeError(p, msg);
					return;
				}
			}
			
			if(p->parenTermStack.len > 1 || p->containsImmediateNext != NULL){
				dropSeqLayerIfSole(p, (InterSeqs*)firstPtr(&p->parenTermStack)); //if this line is in some way open to the next and there's only one root element, then it will not be the head term of a seqs with the indented content, drop the seqs that was going to be that.
			}
			
			emplaceNewLine(&p->indentStack, p->salientIndentationLength, receiveForemostRecepticle(p));
		}else{
			dropSeqLayerIfSole(p, (InterSeqs*)firstPtr(&p->parenTermStack));
			
			p->containsImmediateNext = NULL;
			if(p->salientIndentationLength < p->previousIndentationLength){
				if(strncmp(
					p->previousIndentation,
					p->salientIndentation,
					p->salientIndentationLength
				) != 0){ //then previousIndentation does not start with salientIndentation
					lodgeError(p,"inconsistent indentation");
				}
				//pop to enclosing scope
				while(lastLine(&p->indentStack)->indentationLength > p->salientIndentationLength){
					// if(lastLine(p->indentStack)->indentationLength < p->salientIndentationLength){ //I don't know what this was supposed to do, but it is not doing it. Leaving in case a related bug pops up, it might be a clue
					// 	lodgeErrorAt(line, column, "inconsistent indentation, sibling elements have different indentation")
					// }
					popLine(&p->indentStack);
				}
			}
			
			//ensure that indent adheres to the coding
			if(p->coding.indentIsStrict){
				char const* indent = p->coding.indent;
				uint32_t indentl = strlen(indent);
				char indentc = indent[0];
				if((p->previousIndentationLength - p->salientIndentationLength) % indentl){
					char msg[128];
					if(indentl > 1){
						char const* indentCharName = (indentc == ' ')? "spaces" : "tabs";
						sprintf(msg, "an indentation style requirement applies. Each indent must consist of %d %s.", indentl, indentCharName);
					}else{
						char const* indentCharName = (indentc == ' ')? "space" : "tab";
						sprintf(msg, "an indentation style requirement applies. Each indent must consist of one %s.", indentCharName);
					}
					lodgeError(p, msg);
					return;
				}
			}
		}
		clearPtrBuffer(&p->parenTermStack);
	}else{
		p->hasFoundLine = true;
	}
}
void closeParen(Parser* p){
	if(p->parenTermStack.len <= 1){ //not supposed to be <= 0, the bottom level is the root line, and must be retained
		lodgeErrorAt(p, p->line, p->column, "unbalanced paren");
	}
	p->containsImmediateNext = NULL;
	popPtr(&p->parenTermStack);
	p->lastAttachedTerm = (InterTerm*)lastPtr(&p->parenTermStack);
}
InterStri* receiveFinishedSymbol(Parser* p){
	return interSt(p, copyOutNullTerminatedString(&p->stringBuffer));
}

void eatingIndentation(Parser* p, bool fileEnd, Rune c);
void seekingTerm(Parser* p, bool fileEnd, Rune c);
void immediatelyAfterTerm(Parser* p, bool fileEnd, Rune c);
void buildingSymbol(Parser* p, bool fileEnd, Rune c);
void buildingQuotedSymbol(Parser* p, bool fileEnd, Rune c);
void takingEscape(Parser* p, bool fileEnd, Rune c);
void multiLineFirstLine(Parser* p, bool fileEnd, Rune c);
void multiLineTakingIndent(Parser* p, bool fileEnd, Rune c);
void multiLineTakingText(Parser* p, bool fileEnd, Rune c);

void eatingIndentation(Parser* p, bool fileEnd, Rune c){
	if(fileEnd){
		clearCharBuffer(&p->stringBuffer);
		finishTakingIndentationAndAdjustLineAttachment(p);
	}else if(c == '\n'){
		clearCharBuffer(&p->stringBuffer);
	}else if(
		c == p->coding.pairingChar ||
		c == p->coding.openingBracket
	){
		finishTakingIndentationAndAdjustLineAttachment(p);
		transition(p, seekingTerm);
		seekingTerm(p,false,c);
	}else if(c == p->coding.closingBracket){
		lodgeError(p, "nothing to close");
	}else if(c == '"'){
		finishTakingIndentationAndAdjustLineAttachment(p);
		transition(p, buildingQuotedSymbol);
	}else if(
		c == ' ' ||
		c == '\t'
	){
		addRune(&p->stringBuffer, c);
	}else{
		finishTakingIndentationAndAdjustLineAttachment(p);
		transition(p, buildingSymbol);
		buildingSymbol(p,false,c);
	}
}
void seekingTerm(Parser* p, bool fileEnd, Rune c){ 
	if(fileEnd){
		finishTakingIndentationAndAdjustLineAttachment(p);
	}else{
		if(c == p->coding.openingBracket){
			InterSeqs* newSq = interSq(p);
			attach(p, (InterTerm*)newSq);
			addPtr(&p->parenTermStack, newSq);
		}else if(c == p->coding.closingBracket){
			closeParen(p);
			transition(p, immediatelyAfterTerm);
		}else if(c == p->coding.pairingChar){
			InterSeqs* newSq = interSq(p);
			attach(p, (InterTerm*)newSq);
			p->containsImmediateNext = &newSq->s;
		}else if(c == '\n'){
			transition(p, eatingIndentation);
		}else if(c == ' ' || c == '\t'){
		}else if(c == '"'){
			transition(p, buildingQuotedSymbol);
		}else{
			transition(p, buildingSymbol);
			buildingSymbol(p,false, c);
		}
	}
}
void immediatelyAfterTerm(Parser* p, bool fileEnd, Rune c){
	if(fileEnd){
		finishTakingIndentationAndAdjustLineAttachment(p);
	}else{
		if(c == p->coding.openingBracket){
			InterTerm* newLevel = p->lastAttachedTerm;
			exudeSeqs(p, newLevel);
			addPtr(&p->parenTermStack, newLevel);
			transition(p, seekingTerm);
		}else if(c == p->coding.closingBracket){
			closeParen(p);
		}else if(c == p->coding.pairingChar){
			InterTerm* newLevel = p->lastAttachedTerm;
			exudeSeqs(p, newLevel);
			p->containsImmediateNext = &newLevel->seqs.s;
			transition(p, seekingTerm);
		}else if(c == '\n'){
			transition(p, eatingIndentation);
		}else if(c == ' ' || c == '\t'){
			transition(p, seekingTerm);
		}else if(c == '"'){
			InterTerm* newLevel = p->lastAttachedTerm;
			exudeSeqs(p, newLevel);
			p->containsImmediateNext = &newLevel->seqs.s;
			transition(p, buildingQuotedSymbol);
		}else{
			lodgeError(p, "You have to put a space here. Yes I know the fact that I can say that means I could just pretend there's a space there and let you go ahead, but I wont be doing that, as I am an incorrigible formatting nazi.");
		}
	}
}
void buildingSymbol(Parser* p, bool fileEnd, Rune c){
	if(fileEnd){
		finishTakingSymbolAndAttach(p);
		finishTakingIndentationAndAdjustLineAttachment(p);
	}else{
		if(c == ' ' || c == '\t'){
			finishTakingSymbolAndAttach(p);
			transition(p, seekingTerm);
		}else if(c == p->coding.pairingChar || c == '\n' || c == p->coding.openingBracket || c == p->coding.closingBracket){
			finishTakingSymbolAndAttach(p);
			transition(p, immediatelyAfterTerm);
			immediatelyAfterTerm(p,false,c);
		}else if(c == '"'){
			finishTakingSymbolAndAttach(p);
			transition(p, immediatelyAfterTerm);
			immediatelyAfterTerm(p,false,c);
		}else{
			addRune(&p->stringBuffer, c);
		}
	}
}
void buildingQuotedSymbol(Parser* p, bool fileEnd, Rune c){
	if(fileEnd){
		finishTakingSymbolAndAttach(p);
		finishTakingIndentationAndAdjustLineAttachment(p);
	}else switch(c){
		case '"':
			finishTakingSymbolAndAttach(p);
			transition(p, immediatelyAfterTerm);
		break;
		case '\\':
			pushMode(p, takingEscape);
		break;
		case '\n':
			if(p->stringBuffer.len == 0){
				transition(p, multiLineFirstLine);
			}else{
				finishTakingSymbolAndAttach(p);
				transition(p, eatingIndentation);
			}
		break;
		default:
			addRune(&p->stringBuffer, c);
		break;
	}
}
void takingEscape(Parser* p, bool fileEnd, Rune c){
	if(fileEnd){
		lodgeError(p, "invalid escape sequence, no one can escape the end of the file");
	}else{
		switch(c){
			// case 'h': addRune(&p->stringBuffer, '☃'); break; //TODO, properly implement snowman escaping to bring in line with the spec
			case 'n': addRune(&p->stringBuffer, '\n'); break;
			case 'r': addRune(&p->stringBuffer, '\r'); break;
			case 't': addRune(&p->stringBuffer, '\t'); break;
			default:  addRune(&p->stringBuffer, c); break;
		}
		popMode(p);
	}
}
void multiLineFirstLine(Parser* p, bool fileEnd, Rune c){
	if(fileEnd){
		finishTakingSymbolAndAttach(p);
		finishTakingIndentationAndAdjustLineAttachment(p);
	}else switch(c){
		case ' ':
		case '\t':
			addRune(&p->multilineStringIndentBuffer, c);
		break;
		default:
			p->multilineStringsIndent = copyLengthedStringAndClear(
				&p->multilineStringIndentBuffer,
				&p->multilineStringsIndentLength
			);
			if(p->multilineStringsIndentLength > p->salientIndentationLength){
				if(strncmp(p->multilineStringsIndent, p->salientIndentation, p->salientIndentationLength) == 0){
					transition(p, multiLineTakingText);
					multiLineTakingText(p,false,c);
				}else{
					lodgeError(p, "inconsistent indentation");
				}
			}else{
				finishTakingSymbolAndAttach(p);
				//transfer control to eatingIndentation
				clearCharBuffer(&p->stringBuffer);
				addString(&p->stringBuffer, p->multilineStringsIndent, p->multilineStringsIndentLength);
				freeIfSome(p->multilineStringsIndent);
				transition(p, eatingIndentation);
				eatingIndentation(p,false,c);
			}
			clearCharBuffer(&p->multilineStringIndentBuffer);
	}
}
void multiLineTakingIndent(Parser* p, bool fileEnd, Rune c){
	if(fileEnd){
		finishTakingSymbolAndAttach(p);
		finishTakingIndentationAndAdjustLineAttachment(p);
	}else if(c == ' ' || c == '\t'){
		addRune(&p->multilineStringIndentBuffer, c);
		if(p->multilineStringIndentBuffer.len == p->multilineStringsIndentLength){ //then we're through with the indent
			if(strncmp(p->multilineStringIndentBuffer.v, p->multilineStringsIndent, p->multilineStringsIndentLength) == 0){
				//now we know that it continues, we can insert the endline from the previous line
				addChar(&p->stringBuffer, '\n');
				transition(p, multiLineTakingText);
			}else{
				lodgeError(p, "inconsistent indentation");
			}
			clearCharBuffer(&p->multilineStringIndentBuffer);
		}
	}else if(c == '\n'){
		clearCharBuffer(&p->multilineStringIndentBuffer); //ignores whitespace lines
	}else{
		char* indentAsItWas;
		uint32_t indentAsItWasLength;
		indentAsItWas = copyLengthedStringAndClear(&p->multilineStringIndentBuffer, &indentAsItWasLength);
		clearCharBuffer(&p->multilineStringIndentBuffer);
		//assert(indentAsItWasLength < multilineStringsIndentLength)
		if(strncmp(p->multilineStringsIndent, indentAsItWas, indentAsItWasLength) == 0){
			//breaking out, transfer control to eatingIndentation
			finishTakingSymbolAndAttach(p);
			clearCharBuffer(&p->stringBuffer);
			addStr(&p->stringBuffer, indentAsItWas);
			transition(p, eatingIndentation);
			eatingIndentation(p,false,c);
		}else{
			lodgeError(p, "inconsistent indentation");
		}
		freeIfSome(indentAsItWas);
	}
}
void multiLineTakingText(Parser* p, bool fileEnd, Rune c){
	if(fileEnd){
		finishTakingSymbolAndAttach(p);
		finishTakingIndentationAndAdjustLineAttachment(p);
	}else switch(c){
		case '\n':
			// stringBuffer += '\n'   will not add the newline until we're sure the multiline string is continuing
			transition(p, multiLineTakingIndent);
		break;
		default:
			addRune(&p->stringBuffer, c);
	}
}

void parseLengthedToSeqsTakingParserRef(Parser* p, char const* unicodeString, unsigned length, TermposeParsingError* errorOut){ //contents will be contained in a root seqs even if there is only a single line at root. Parser aught to be reset, but wont be for now. If there was an error, it will be written of in errorOut, and if there wasn't, errorOut will be set to null. If there was no error, the parsed struct will reside in p, as interterms in the rootArBuf.
	//pump characters into the mode of the parser until the read head has been graduated to the end
	transition(p, eatingIndentation);
	size_t i=0;
	while(i < length){
		Rune c;
		i += utf8decode(unicodeString+i, &c, length-i);
		if(c == UTF_INVALID){
			lodgeError(p, "not valid unicode");
			goto error;
		}
		if(c == '\r'){ //handle windows' deviant line endings
			c = '\n';
			if(i < length && unicodeString[i] == '\n'){ //if the \r has a \n following it, don't register that
				i += 1;
			}
		}
		p->currentMode(p,false,c);
		if(p->error.msg) goto error;
		p->index += 1;
		if(c == '\n'){
			p->line += 1;
			p->column = 0;
		}else{
			p->column += 1;
		}
	}
	p->currentMode(p,true,0);
	
	if(p->error.msg){
		error:
		if(errorOut != NULL){
			*errorOut = p->error;
		}
	}else{
		if(errorOut != NULL){
			errorOut->msg = NULL;
		}
	}
}

Term_t* parseLengthedToSeqsWithCoding(char const * unicodeString, uint32_t length, TermposeParsingError* errorOut, TermposeCoding const* coding){
	Parser p;
	initParserWithCoding(&p, coding);
	parseLengthedToSeqsTakingParserRef(&p, unicodeString, length, errorOut);
	if(errorOut->msg){
		drainParser(&p);
		return NULL;
	}else{
		Term_t* t = (Term_t*)malloc(sizeof(Term_t));
		uint32_t rarl = p.rootArBuf.len;
		Term_t* outArr = (Term_t*)malloc(rarl*sizeof(Term_t));
		InterTerm** rarv = (InterTerm**)p.rootArBuf.v;
		for(int i=0; i<rarl; ++i){
			termMimicInterterm(outArr+i, rarv[i]);
		}
		errorOut->msg = NULL;
		initSeqs(t, outArr, rarl, 0,0);
		drainParser(&p);
		return t;
	}
}
Term_t* parseLengthedToSeqs(char const * unicodeString, uint32_t length, TermposeParsingError* errorOut){
	TermposeCoding tc;
	initDefaultCoding(&tc);
	Term_t* ret = parseLengthedToSeqsWithCoding(unicodeString, length, errorOut, &tc);
	drainCoding(&tc);
	return ret;
}

Term_t* parseAsListWithCoding(char const* unicodeString, TermposeParsingError* errorOut, TermposeCoding const* coding){
	return parseLengthedToSeqsWithCoding(unicodeString, strlen(unicodeString), errorOut, coding);
}
Term_t* parseAsList(char const* unicodeString, TermposeParsingError* errorOut){
	return parseLengthedToSeqs(unicodeString, strlen(unicodeString), errorOut);
}

Term_t* parse(char const* unicodeString, TermposeParsingError* errorOut){
	Term_t* ret = parseAsList(unicodeString, errorOut);
	if(ret->len == 1){
		Term_t* truRet = ret->con;
		free(ret);
		return truRet;
	}else{
		return ret;
	}
}

Term_t* parseWithCoding(char const* unicodeString, TermposeParsingError* errorOut, TermposeCoding const* tc){
	Parser p;
	initParser(&p);
	parseLengthedToSeqsTakingParserRef(&p, unicodeString, strlen(unicodeString), errorOut);
	if(errorOut->msg){
		drainParser(&p);
		return NULL;
	}else{
		Term_t* t = (Term_t*)malloc(sizeof(Term_t));
		uint32_t rarl = p.rootArBuf.len;
		Term_t* outArr = (Term_t*)malloc(rarl*sizeof(Term_t));
		InterTerm** rarv = (InterTerm**)p.rootArBuf.v;
		for(int i=0; i<rarl; ++i){
			termMimicInterterm(outArr+i, rarv[i]);
		}
		errorOut->msg = NULL;
		initSeqs(t, outArr, rarl, 0,0);
		drainParser(&p);
		return t;
	}
}