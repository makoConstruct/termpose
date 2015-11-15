#include "termpose.h"
#include <stdlib.h>
#include <assert.h>
#include <stdint.h>
#include <string.h>
#include <stdio.h>


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



void clearStr(char** c){
	if(*c != "" && *c != NULL){
		free(*c);
		*c = "";
	}
}

void freeIfSome(void* v){ if(v != NULL) free(v); }




#define new(type, identifier) type* identifier = malloc(sizeof(type))
typedef uint32_t u32;
typedef uint8_t u8;
typedef uint8_t bool;
#define true 1
#define false 0
u32 min(u32 a, u32 b){ return a < b ? a : b; }
u32 max(u32 a, u32 b){ return a < b ? b : a; }
u32 ceilPowerTwo(u32 v){
	v -= 1;
	v = v | (v >> 16);
	v = v | (v >> 8);
	v = v | (v >> 4);
	v = v | (v >> 2);
	v = v | (v >> 1);
	return v+1;
}
void* makeCopy(void* in, size_t len){
	void* ret = malloc(len);
	memcpy(ret, in, len);
	return ret;
}

char* strCopy(char* v){ return makeCopy(v, strlen(v)+1); }


//buffers
typedef struct CharBuffer {
	char* v;
	u32 len;
	u32 cap;
} CharBuffer;
void initCharBuffer(CharBuffer* cb, u32 initialLen, u32 initialCapacity){
	cb->v = malloc(initialCapacity*sizeof(char));
	cb->len = initialLen;
	cb->cap = initialCapacity;
}
CharBuffer* newCharBuf(u32 initialLen){
	new(CharBuffer, ret);
	initCharBuffer(ret,initialLen,ceilPowerTwo(initialLen));
	return ret;
}
void reserveCapacityAfterLength(CharBuffer* t, u32 extraCap){
	u32 finalCap = t->len + extraCap;
	if(t->cap < finalCap){
		u32 newCap = ceilPowerTwo(finalCap);
		t->v = realloc(t->v, newCap);
		t->cap = newCap;
	}
}
void addChar(CharBuffer* t, char c){
	u32 tl = t->len;
	if(tl >= t->cap){
		u32 newCap = t->cap*2;
		t->v = realloc(t->v, newCap*sizeof(char));
		t->cap = newCap;
	}
	t->v[tl] = c;
	t->len = tl+1;
}
void addStr(CharBuffer* t, char const* c){
	u32 i = 0;
	while(c[i] != 0){
		addChar(t, c[i]);
		++i;
	}
}
void addU32(CharBuffer* t, u32 n){
	const u32 ULEN = 12;
	reserveCapacityAfterLength(t, ULEN); //ensure we can take the thing no matter how long it is (it will never be longer than ULEN)
	t->len += sprintf(t->v + t->len, "%u", n);
}
void addString(CharBuffer* t, char* c, u32 len){
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
	return realloc(t->v, sizeof(char)*t->len);
}
char* copyLengthedStringAndClear(CharBuffer* t, u32* rlen){ //alert, returns NULL if empty
	u32 tl = t->len;
	*rlen = tl;
	t->len = 0;
	if(tl){
		char* out = malloc((tl)*sizeof(char));
		memcpy(out, t->v, tl*sizeof(char));
		return out;
	}else{
		return NULL;
	}
}
char* copyOutNullTerminatedString(CharBuffer* t){
	char* out = malloc((t->len+1)*sizeof(char));
	memcpy(out, t->v, t->len*sizeof(char));
	out[t->len] = 0;
	t->len = 0;
	return out;
}
void drainCharBuffer(CharBuffer* t){
	free(t->v);
}


typedef struct PtrBuffer{
	void** v;
	u32 len;
	u32 cap;
} PtrBuffer;
void initPtrBuffer(PtrBuffer* cb, u32 initialLen, u32 initialCapacity){
	cb->v = malloc(initialCapacity*sizeof(void*));
	cb->len = initialLen;
	cb->cap = initialCapacity;
}
void initEmptyPtrBuffer(PtrBuffer* cb, u32 initialCapacity){
	cb->v = malloc(initialCapacity*sizeof(void*));
	cb->len = 0;
	cb->cap = initialCapacity;
}
PtrBuffer* newPtrBuf(u32 initialLen){
	new(PtrBuffer, ret);
	initPtrBuffer(ret,initialLen,ceilPowerTwo(initialLen));
	return ret;
}
void addPtr(PtrBuffer* t, void* c){
	u32 tl = t->len;
	if(tl >= t->cap){
		u32 newCap = t->cap*2;
		t->v = realloc(t->v, newCap*sizeof(void*));
		t->cap = newCap;
	}
	t->v[tl] = c;
	t->len = tl+1;
}
void* popPtr(PtrBuffer* t){ //note, does not downsize the array, but why would you
	u32 lmo = t->len - 1;
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
	u32 indentationLength;
	PtrBuffer* tailestTermSequence; //[InterTerm*]
} LineDetail;
typedef struct LineBuffer{
	LineDetail* v;
	u32 len;
	u32 cap;
} LineBuffer;
void initLineBuffer(LineBuffer* cb, u32 initialLen, u32 initialCapacity){
	cb->v = malloc(initialCapacity*sizeof(LineDetail));
	cb->len = initialLen;
	cb->cap = initialCapacity;
}
LineBuffer* newLineBuf(u32 initialLen){
	new(LineBuffer, ret);
	initLineBuffer(ret,initialLen,ceilPowerTwo(initialLen));
	return ret;
}
LineDetail* emplaceNewLine(LineBuffer* t, u32 indentationLength, PtrBuffer* tailestTermSequence){
	u32 tl = t->len;
	if(tl >= t->cap){
		u32 newCap = t->cap*2;
		t->v = realloc(t->v, newCap*sizeof(LineDetail));
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









void initSeqs(Term* ret, Term* s, uint32_t nTerms, uint32_t line, uint32_t column){
	ret->tag = 0;
	ret->len = nTerms;
	ret->line = line;
	ret->column = column;
	ret->con = s;
}
Term* newSeqs(Term* s, uint32_t nTerms, uint32_t line, uint32_t column){
	new(Term, ret);
	initSeqs(ret, s, nTerms, line, column);
	return ret;
}
Term* newStri(char* v, uint32_t length, uint32_t line, uint32_t column){
	new(Term, ret);
	ret->tag = 1;
	ret->len = length;
	ret->line = line;
	ret->column = column;
	ret->str = v;
	return ret;
}
uint8_t isStri(Term* v){ return *(uint8_t*)v; } 
void drainTerm(Term* v){
	Term* sv = v->con; //doesn't so much assume that this is a list term as it just doesn't matter for the free that comes later
	if(!isStri(v)){
		uint32_t sl = v->len;
		for(int i=0; i<sl; ++i){
			drainTerm(sv+i);
		}
	}
	free(sv);
}
void destroyTerm(Term* v){
	drainTerm(v);
	free(v);
}

//term stringification:
bool escapeIsNeeded(char* v, size_t len){
	for(int i=0; i<len; ++i){
		switch(v[i]){
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
void escapeSymbol(CharBuffer* outUnicode, char* in, u32 len){
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
void stringifyingStri(CharBuffer* cb, Term* v){
	uint32_t vl = v->len;
	char* s = v->str;
	if(escapeIsNeeded(s, vl)){
		addChar(cb, '"');
		escapeSymbol(cb, s, vl);
		addChar(cb, '"');
	}else{
		addString(cb, s, vl);
	}
}
void stringifyingTerm(CharBuffer* cb, Term* t);
void stringifyingSeqs(CharBuffer* cb, Term* v){
	addChar(cb, '(');
	uint32_t tl = v->len;
	Term* ts = v->con;
	if(tl >= 1){
		stringifyingTerm(cb, ts);
	}
	for(int i=1; i<tl; ++i){
		addChar(cb, ' ');
		stringifyingTerm(cb, ts+i);
	}
	addChar(cb, ')');
}
void stringifyingTerm(CharBuffer* cb, Term* t){
	if(isStri(t)){
		stringifyingStri(cb, t);
	}else{
		stringifyingSeqs(cb, t);
	}
}
char* stringifyTerm(Term* t){ //returns null if one of the strings contained invalid unicode.
	CharBuffer cb;
	initCharBuffer(&cb, 0, 128);
	stringifyingTerm(&cb, t);
	return rendStr(&cb);
}


// struct ParsingException{
// 	char* msg;
// 	u32 line;
// 	u32 column;
// };

typedef struct InterSeqs{ //allocated into InterTerms
	u8 tag/*=0*/;
	u32 line;
	u32 column;
	PtrBuffer s;
} InterSeqs;
typedef struct InterStri{ //allocated into InterTerms
	u8 tag/*=1*/;
	u32 line;
	u32 column;
	char* v;
} InterStri;


typedef union InterTerm{ InterSeqs seqs; InterStri stri; } InterTerm;
bool isInterStri(InterTerm* v){ return *(bool*)v; }
void initInterSeqs(InterTerm* ret, u32 line, u32 column){
	ret->seqs.tag = 0;
	ret->seqs.line = line;
	ret->seqs.column = column;
	initPtrBuffer(&ret->seqs.s,0,4);
}
InterSeqs* newInterSeqs(u32 line, u32 column){
	InterTerm* ret = malloc(sizeof(InterTerm));
	initInterSeqs(ret,line,column);
	return (InterSeqs*)ret;
}
InterStri* newInterStri(u32 line, u32 column, char* v){
	InterTerm* ret = malloc(sizeof(InterTerm));
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
	u32 tsl = ts->len;
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



typedef struct Parser Parser;
void updateStateAboutInterTermMove(Parser* p, InterTerm* to, InterTerm* from);
void dropSeqLayerIfSole(Parser* p, InterSeqs* pt){
	assert(pt->tag == 0);
	PtrBuffer* arb = &pt->s;
	if(arb->len == 1){
		InterTerm* soleEl = arb->v[0];
		drainPtrBuffer(arb);
		memcpy(pt, soleEl, sizeof(InterTerm));
		free(soleEl);
		updateStateAboutInterTermMove(p, (InterTerm*)pt, soleEl);
	}
}
void exudeSeqs(Parser* p, InterTerm* pi){ //pi becomes interSeqs, its previous content is copied out and linked as its new first element.
	assert( pi->seqs.line == pi->stri.line && pi->seqs.column == pi->stri.column ); //it should not matter which interpretation of the union type we use.
	u32 l = pi->seqs.line;
	u32 c = pi->seqs.column;
	InterTerm* nit = malloc(sizeof(InterTerm));
	memcpy(nit, pi, sizeof(InterTerm));
	initInterSeqs(pi, l,c);
	addPtr(&pi->seqs.s, nit);
	updateStateAboutInterTermMove(p, nit, pi);
}


void termMimicInterterm(Term* t, InterTerm* v){
	if(isInterStri(v)){
		t->tag = 1;
		char* tsv = v->stri.v;
		u32 stl = strlen(tsv);
		t->str = makeCopy(tsv, (stl+1)*sizeof(char));
		t->len = stl;
		t->line = v->stri.line;
		t->column = v->stri.column;
	}else{
		t->tag = 0;
		u32 slen = v->seqs.s.len;
		Term* innerTerms = malloc(slen*sizeof(Term));
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



// parser mode. PF is short for "processing funnel"
typedef struct Parser Parser;
typedef void (*PF)(Parser*, bool, Rune);


typedef struct Parser{
	CharBuffer stringBuffer;
	char* previousIndentation;  u32 previousIndentationLength;
	char* salientIndentation;  u32 salientIndentationLength;
	bool hasFoundLine;
	PtrBuffer rootArBuf; //[InterTerm*]
	PtrBuffer parenTermStack; //[InterSeqs*]
	u32 line;
	u32 column;
	u32 index;
	InterTerm* lastAttachedTerm;
	PF currentMode;
	PtrBuffer modes; //[PF]
	char* error;
	LineBuffer indentStack; //indentation length, tailestTermSequence. It is enough to keep track of length. We can derive ∀(a,b∈Line.Indentation) a.length = b.length → a = b from the prefix checking that is done before adding to the stack
	CharBuffer multilineStringIndentBuffer;
	char* multilineStringsIndent;  u32 multilineStringsIndentLength;
	PtrBuffer* containsImmediateNext; //[InterTerm*]
} Parser;
void initParser(Parser* p){
	initCharBuffer(&p->stringBuffer,0,256);
	p->previousIndentation = NULL;  p->previousIndentationLength = 0;
	p->salientIndentation = NULL;  p->salientIndentationLength = 0;
	p->hasFoundLine = false;
	initPtrBuffer(&p->rootArBuf,0,32);
	initPtrBuffer(&p->parenTermStack,0,32);
	p->line = 0;
	p->column = 0;
	p->index = 0;
	p->lastAttachedTerm = NULL;
	p->currentMode = NULL;
	initPtrBuffer(&p->modes,0,24);
	p->error = NULL;
	initLineBuffer(&p->indentStack,0,32);
	emplaceNewLine(&p->indentStack, 0, &p->rootArBuf);
	initCharBuffer(&p->multilineStringIndentBuffer,0,128);
	p->multilineStringsIndent = NULL;  p->multilineStringsIndentLength = 0;
	p->containsImmediateNext = NULL;
}
void drainParser(Parser* p){
	drainCharBuffer(&p->stringBuffer);
	freeIfSome(p->previousIndentation);  p->previousIndentationLength = 0;
	freeIfSome(p->salientIndentation);  p->salientIndentationLength = 0;
	p->hasFoundLine = false;
	//destroy the interterms
	PtrBuffer* rab = &p->rootArBuf;
	u32 rlen = rab->len;
	for(int i=0; i<rlen; ++i){
		InterTerm* it = rab->v[i];
		drainInterTerm(it);
		free(it);
	}
	drainPtrBuffer(rab);
	drainPtrBuffer(&p->parenTermStack);
	drainPtrBuffer(&p->modes);
	freeIfSome(p->error);
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
	u32 ptsl = p->parenTermStack.len;
	for(int i=0; i<ptsl; ++i){
		if(ptsv[i] == from){
			ptsv[i] = to;
			break;
		}
	}
}
void transition(Parser* p, PF nm){ p->currentMode = nm; }
void pushMode(Parser* p, PF nm){
	addPtr(&p->modes, p->currentMode);
	transition(p, nm);
}
InterSeqs* interSq(Parser* p){ return newInterSeqs(p->line, p->column); }
InterStri* interSt(Parser* p, char* c){ return newInterStri(p->line, p->column, c); }
void popMode(Parser* p){
	p->currentMode = popPtr(&p->modes);
}
void lodgeErrorAt(Parser* p, u32 l, u32 c, char const* message){
	CharBuffer cb;
	initCharBuffer(&cb, 0, 32);
	addStr(&cb, "line:");
	addU32(&cb, l);
	addStr(&cb, " column:");
	addU32(&cb, c);
	addStr(&cb, ". ");
	addStr(&cb, message);
	p->error = rendStr(&cb);
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
			//
			if( strncmp(
				p->previousIndentation,
				p->salientIndentation,
				p->previousIndentationLength
			) != 0 ){ //if the shorter (previousIndentationLength) does not prefix the longer
				lodgeErrorAt(p, p->line - p->salientIndentationLength, p->column, "inconsistent indentation at");
				return;
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
	}else switch(c){
		case '\n':
			clearCharBuffer(&p->stringBuffer);
		break;
		case ':':
		case '(':
			finishTakingIndentationAndAdjustLineAttachment(p);
			transition(p, seekingTerm);
			seekingTerm(p,false,c);
		break;
		case ')':
			lodgeError(p, "nothing to close");
		break;
		case '"':
			finishTakingIndentationAndAdjustLineAttachment(p);
			transition(p, buildingQuotedSymbol);
		break;
		case ' ':
		case '\t':
			addRune(&p->stringBuffer, c);
		break;
		default:
			finishTakingIndentationAndAdjustLineAttachment(p);
			transition(p, buildingSymbol);
			buildingSymbol(p,false,c);
	}
}
void seekingTerm(Parser* p, bool fileEnd, Rune c){ 
	if(fileEnd){
		finishTakingIndentationAndAdjustLineAttachment(p);
	}else{
		InterSeqs* newSq;
		switch(c){
		case '(':
			newSq = interSq(p);
			attach(p, (InterTerm*)newSq);
			addPtr(&p->parenTermStack, newSq);
		break;
		case ')':
			closeParen(p);
			transition(p, immediatelyAfterTerm);
		break;
		case ':':
			newSq = interSq(p);
			attach(p, (InterTerm*)newSq);
			p->containsImmediateNext = &newSq->s;
		break;
		case '\n':
			transition(p, eatingIndentation);
		break;
		case ' ': case '\t': break;
		case '"':
			transition(p, buildingQuotedSymbol);
		break;
		default:
			transition(p, buildingSymbol);
			buildingSymbol(p,false, c);
		}
	}
}
void immediatelyAfterTerm(Parser* p, bool fileEnd, Rune c){
	if(fileEnd){
		finishTakingIndentationAndAdjustLineAttachment(p);
	}else{
		InterTerm* newLevel;
		switch(c){
		case '(':
			newLevel = p->lastAttachedTerm;
			exudeSeqs(p, newLevel);
			addPtr(&p->parenTermStack, newLevel);
			transition(p, seekingTerm);
		break;
		case ')':
			closeParen(p);
		break;
		case ':':
			newLevel = p->lastAttachedTerm;
			exudeSeqs(p, newLevel);
			p->containsImmediateNext = &newLevel->seqs.s;
			transition(p, seekingTerm);
		break;
		case '\n':
			transition(p, eatingIndentation);
		break;
		case ' ':
		case '\t':
			transition(p, seekingTerm);
		break;
		case '"':
			newLevel = p->lastAttachedTerm;
			exudeSeqs(p, newLevel);
			p->containsImmediateNext = &newLevel->seqs.s;
			transition(p, buildingQuotedSymbol);
		break;
		default:
			lodgeError(p, "You have to put a space here. Yes I know the fact that I can say that means I could just pretend there's a space there and let you go ahead, but I wont be doing that, as I am an incorrigible formatting nazi.");
		}
	}
}
void buildingSymbol(Parser* p, bool fileEnd, Rune c){
	if(fileEnd){
		finishTakingSymbolAndAttach(p);
		finishTakingIndentationAndAdjustLineAttachment(p);
	}else switch(c){
		case ' ':
		case '\t':
			finishTakingSymbolAndAttach(p);
			transition(p, seekingTerm);
		break;
		case ':':
		case '\n':
		case '(':
		case ')':
			finishTakingSymbolAndAttach(p);
			transition(p, immediatelyAfterTerm);
			immediatelyAfterTerm(p,false,c);
		break;
		case '"':
			finishTakingSymbolAndAttach(p);
			transition(p, immediatelyAfterTerm);
			immediatelyAfterTerm(p,false,c);
		break;
		default:
			addRune(&p->stringBuffer, c);
		break;
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
		u32 indentAsItWasLength;
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

void parseLengthedToSeqsTakingParserRef(Parser* p, char const* unicodeString, unsigned length, Term* termOut, char** errorOut){ //contents will be contained in a root seqs even if there is only a single line at root. Parser aught to be reset, but wont be for now. termOut will not be initialized if there was an error. If there was an error, it will be written of in errorOut, and if there wasn't, errorOut will be set to null.
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
		if(p->error) goto error;
		p->index += 1;
		if(c == '\n'){
			p->line += 1;
			p->column = 0;
		}else{
			p->column += 1;
		}
	}
	p->currentMode(p,true,0);
	
	if(p->error){
		error:
		if(errorOut != NULL){
			*errorOut = strCopy(p->error);
		}
		return;
	}else{
		//build term from interterm
		u32 rarl = p->rootArBuf.len;
		Term* outArr = malloc(rarl*sizeof(Term));
		InterTerm** rarv = (InterTerm**)p->rootArBuf.v;
		for(int i=0; i<rarl; ++i){
			termMimicInterterm(outArr+i, rarv[i]);
		}
		*errorOut = NULL;
		initSeqs(termOut, outArr, rarl, 0,0);
	}
}


void parseLengthedToPrealocatedSeqs(char const * unicodeString, unsigned length, Term* termOut, char** errorOut){
	Parser p;
	initParser(&p);
	parseLengthedToSeqsTakingParserRef(&p, unicodeString, length, termOut, errorOut);
	drainParser(&p);
}

Term* parseLengthedToSeqs(char const * unicodeString, unsigned length, char** errorOut){
	Term* t = malloc(sizeof(Term));
	parseLengthedToPrealocatedSeqs(unicodeString, length, t, errorOut);
	if(*errorOut){
		free(t);
		return NULL;
	}else{
		return t;
	}
}

Term* parseAsList(char const* unicodeString, char** errorOut){
	return parseLengthedToSeqs(unicodeString, strlen(unicodeString), errorOut);
}

Term* parse(char const* unicodeString, char** errorOut){
	Term* ret = parseAsList(unicodeString, errorOut);
	if(ret->len == 1){
		Term* truRet = ret->con;
		free(ret);
		return truRet;
	}else{
		return ret;
	}
}