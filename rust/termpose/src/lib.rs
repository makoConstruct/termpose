#![feature(core_intrinsics)]
#![feature(slice_patterns)]
#![feature(custom_derive)]
#![feature(box_patterns)]
#![feature(box_syntax)]
#![feature(convert)]
#![feature(plugin)]
#![feature(libc)]

#![plugin(termpose_codegen)]

use std::ffi::CStr;
use std::ffi::CString;
use std::string::String;
use std::result::{Result};
use std::ops::Deref;
use std::mem::uninitialized;
use std::collections::HashMap;
use std::hash::Hash;

extern crate libc;
use libc::c_void;
use libc::c_char;

#[repr(C)]
struct TermposeTerm{
	tag:u8,
	line:u32,
	column:u32,
	len:u32,
	con:*const c_void,
}

#[repr(C)]
struct TermposeCoding{
	opening_bracket:c_char,
	closing_bracket:c_char,
	pairing_char:c_char,
	indent_is_strict:bool,
	indent:*const c_char,
}

#[repr(C)]
struct TermposeParserError{
	line:u32,
	column:u32,
	msg:*const c_char,
}

#[link(name = "termpose")]
extern "C" {
	fn isStr(v:*const TermposeTerm)-> bool;
	
	// fn parseAsList(unicodeString:*const c_char, errorOut:*mut *mut c_char)-> *mut TermposeTerm; //contents will be contained in a root list even if there is only a single line at root (if you don't want that, see parse()). If there is an error, return value will be null and errorOut will be set to contain the text. If there was no error, errorOut will be set to null. errorOut can be null, if a report is not desired.
	// fn parseWithCoding(unicodeString:*const c_char, errorOut:*mut *mut c_char, coding:*const TermposeCoding)-> *mut TermposeTerm;
	// fn parseLengthedToSeqs(unicodeString:*const c_char, length:u32, errorOut:*mut *mut c_char)-> *mut TermposeTerm;
	fn parseLengthedToSeqsWithCoding(unicodeString:*const c_char, length:u32, errorOut:*mut TermposeParserError, coding:*const TermposeCoding)-> *mut TermposeTerm;
	// fn parse(unicodeString:*const c_char, errorOut:*mut *mut c_char)-> *mut TermposeTerm; //same as above but root line will not be wrapped in a root seqs iff there is only one root element. This is what you want if you know there's only one line in the string you're parsing, but use parseAsList if there could be more(or less) than one, as it is more consistent
	// fn stringifyTerm(t:*const TermposeTerm)-> *mut c_char;
	
	fn destroyTerm(v:*mut TermposeTerm);
	// fn drainTerm(v:*mut TermposeTerm);
	// fn destroyStr(str:*mut c_char);
}

fn mul(s:&str, rhs:u32)-> String {
	let mut ret = String::with_capacity(s.len()*rhs as usize);
	for _ in 0..rhs {
		ret.push_str(s);
	}
	ret
}

unsafe fn copy_vec_from_ptr(dat: *const u8, len: usize) -> Vec<u8> {
    let mut vec = Vec::with_capacity(len);
    ::std::ptr::copy_nonoverlapping(dat, vec.as_mut_ptr(), len);
    vec.set_len(len);
    vec
}

//input must be utf-8
unsafe fn from_parts(v:*const c_char, len:usize)-> String { String::from_utf8_unchecked(copy_vec_from_ptr(v as *const u8, len)) }

unsafe fn extract_first<T>(mut v:Vec<T>)-> T { //OPTIMIZATION; remove moves the contents of the array back one before the array gets dropped, which is a stupid as hell way to implement this operation, but it's literally the only way
	debug_assert!(v.len() >= 1);
	v.remove(0)
}

fn seriously_unreachable()-> ! {
	if cfg!(debug_assertions) { panic!("this code path is not supposed to be reachable") }
	else{ unsafe{std::intrinsics::unreachable()} } }

#[allow(dead_code)] //permitting cstr, which is not visibly used, but is needed to provide ownership for the cstring in TermposeCoding
#[doc = "Codings specify termpose dialects, formats very similar to termpose but perhaps, for instance, they might use []s instead of ()s, or = instead of :. They may also specify style constraints, a Coding might require all indentation to be two spaces, or require it all to be tabs."]
pub struct Coding{
	cstr:CString,
	termpose_coding:TermposeCoding,
}
pub enum BraceStyle{ ROUND, SQUARE, CURLY }
pub enum IndentStyle{ TAB, SPACES(u8) }
impl Coding{
	pub fn new(brace_style:BraceStyle, pairing_char:char, indent_is_strict:bool, indent_style:IndentStyle)-> Coding {
		let (ob, cb) = match brace_style {
			BraceStyle::ROUND=> ('(', ')'),
			BraceStyle::SQUARE=> ('[', ']'),
			BraceStyle::CURLY=> ('{', '}'),
		};
		let indent_string = match indent_style {
			IndentStyle::TAB => "\t".to_string(),
			IndentStyle::SPACES(n)=> {
				if n == 0 {
					panic!("Invalid Termpose Coding. Zero length indentation is not indentation.");
				}else if n > 16 {
					panic!("Invalid Termpose Coding. Too many spaces in your indentation style requirements. You are sick.");
				}else{
					mul(" ",n as u32)
				}
			}
		};
		let cstr = CString::new(indent_string).unwrap();
		let cstrp = cstr.as_ptr();
		Coding{
			cstr:cstr,
			termpose_coding: TermposeCoding{
				opening_bracket:ob as c_char,
				closing_bracket:cb as c_char,
				pairing_char:pairing_char as c_char,
				indent_is_strict:indent_is_strict,
				indent:cstrp, //note, unsafe to move this away from cstr
			}
		}
	}
	#[doc = "The default style (BraceStyle::ROUND, ':', false, IndentStyle::SPACES(2))."]
	pub fn pretty()-> Coding { Coding::new(BraceStyle::ROUND, ':', false, IndentStyle::SPACES(2)) }
	#[doc = "A more practical style using symbols nearer to the home row, that do not require the user to press shift to access (BraceStyle::SQUARE, ';', false, IndentStyle::TAB)."]
	pub fn best()-> Coding { Coding::new(BraceStyle::SQUARE, ';', false, IndentStyle::TAB) }
	#[doc = "A style for the unix cli, needed, as round brackets have a built in meaning in bash (BraceStyle::SQUARE, '=', false, IndentStyle::SPACES(2)."]
	pub fn cli()-> Coding { Coding::new(BraceStyle::SQUARE, '=', false, IndentStyle::SPACES(2)) }
}
impl Default for Coding{
	fn default()-> Coding { Coding::pretty() }
}

struct Terms{ v:*mut TermposeTerm } //just a wrapper for the term pointer result, which drops using the C implementation's disposer
impl Deref for Terms {
	type Target = TermposeTerm;
	fn deref(&self)-> &TermposeTerm { unsafe{&*self.v} }
}
impl Drop for Terms { fn drop(&mut self){ unsafe{destroyTerm(self.v)}; } }

#[derive(Clone)]
pub enum Term{
	List{l:Vec<Term>, line:u32, column:u32},
	Stri{s:String, line:u32, column:u32},
}
pub use Term::*;
pub fn list(l:Vec<Term>)-> Term { List{l:l, line:0,column:0} }
pub fn stri(s:String)-> Term { Stri{s:s, line:0,column:0} }

impl Term{
	unsafe fn from_cterm(v:*const TermposeTerm)-> Term{
		if isStr(v) {
			let vr = &*v;
			Term::Stri{s:from_parts(vr.con as *const c_char, vr.len as usize), line:vr.line, column:vr.column}
		}else{
			let vr = &*v;
			let mut rl = Vec::with_capacity(vr.len as usize);
			let ptr = vr.con as *const TermposeTerm;
			for i in 0..vr.len {
				rl.push(Term::from_cterm(ptr.offset(i as isize)));
			}
			Term::List{l:rl, line:vr.line, column:vr.column}
		}
	}
	pub fn line(&self)-> u32 { match self {  &Stri{line,..}=> line,  &List{line,..}=> line }  }
	pub fn column(&self)-> u32 { match self {  &Stri{column,..}=> column,  &List{column,..}=> column }  }
}

#[derive(PartialEq, Debug)]
pub struct TermposeError{
	line:u32,
	column:u32,
	msg:String,
}
fn termpose_error_from_term(v:&Term, msg:String)-> TermposeError {
	TermposeError{
		line:v.line(),
		column:v.column(),
		msg:msg,
	}
}

fn parse_to_seqs(s:&str, coding:&Coding)-> Result<Terms,TermposeError> {
	let mut err_out:TermposeParserError = unsafe{uninitialized()}; //gets initialized by parseLengthedToSeqs
	let res:*mut TermposeTerm = unsafe{ parseLengthedToSeqsWithCoding(s.as_ptr() as *const c_char, s.len() as u32, &mut err_out, &coding.termpose_coding) };
	if err_out.msg.is_null() {
		Ok(Terms{v:res})
	}else{
		Err(TermposeError{
			line:err_out.line,
			column:err_out.column,
			msg:unsafe{CStr::from_ptr(err_out.msg)}.to_str().unwrap().to_string()
		})
	}
}

fn needs_escape(v:&String, coding:&Coding)-> bool {
	v.as_bytes().iter().any(|cu|{
		let ob = coding.termpose_coding.opening_bracket as u8 as char;
		let pc = coding.termpose_coding.pairing_char as u8 as char;
		let cb = coding.termpose_coding.closing_bracket as u8 as char;
		let c = *cu as char;
			match c {
			' '=> true,
			'\t'=> true,
			'\n'=> true,
			'\r'=> true,
			c if c == ob => true,
			c if c == pc => true,
			c if c == cb => true,
			_=> false,
		}
	})
}

fn write_escaped_if_needed(out:&mut String, v:&String, coding:&Coding){
	if needs_escape(v, coding) {
		out.push('"');
		for cu in v.as_bytes() {
			let c = *cu as char;
			match c {
				'"'=> { out.push('\\'); out.push('"'); },
				'\t'=> { out.push('\\'); out.push('\t'); },
				'\n'=> { out.push('\\'); out.push('\n'); },
				'\r'=> { out.push('\\'); out.push('\r'); },
				_ => out.push(c),
			}
		}
		out.push('"');
	}else{
		out.push_str(v);
	}
}

impl Term{
	pub fn is_list(&self)-> bool {
		match *self { List{..}=> true, Stri{..}=> false }
	}
	#[doc = "Parses, putting every term found at root level(indentation = 0) into a root List term, even if there is only one or no results."]
	pub fn parse_multiline(s:&str)-> Result<Term,TermposeError> {
		Term::parse_multiline_with_coding(s, &Coding::default())
	}
	pub fn parse_multiline_with_coding(s:&str, coding:&Coding)-> Result<Term,TermposeError> {
		parse_to_seqs(s, coding).map(|terms| unsafe{Term::from_cterm(terms.v)})
	}
	pub fn parse_with_coding(s:&str, c:&Coding)-> Result<Term,TermposeError> {
		Term::parse_multiline_with_coding(s, c).map(|term|{ //TODO this is one of those egregious piles of abjection that I had to write before rust was good. Use proper variant types and better borrow scoping to fix this.
			let should_extract = match term {
				List{ref l, ..}=>
					if l.len() == 1 {
						true
					}else{
						false
					},
				Stri{ .. }=> seriously_unreachable(),
			};
			if should_extract {
				match term {
					List{l, ..}=> unsafe{extract_first(l)},
					Stri{..}=> seriously_unreachable(),
				}
			}else{
				term
			}
		})
	}
	#[doc = "This may be preferable to parse_multiline when you are sure that the input text will have just one term at root, in which case it will not wrap that term in a root list. If the text has zero or more than one, though, it will behave as parse_multiline, which may catch you by surprise in some cases. parse_multiline is provided to avoid this in situations where it would constitute an irregularity."]
	pub fn parse(s:&str)-> Result<Term,TermposeError> {
		Term::parse_with_coding(s, &Coding::default())
	}
	fn stringifying(&self, out_stream:&mut String, coding:&Coding){
		match *self {
			Stri{ref s, ..}=> write_escaped_if_needed(out_stream, s, coding),
			List{ref l, ..}=>{
				out_stream.push(coding.termpose_coding.opening_bracket as u8 as char);
				if let Some(ref first) = l.first() {
					first.stringifying(out_stream, coding);
					for el in l.iter().skip(1) {
						out_stream.push(' ');
						el.stringifying(out_stream, coding);
					}
				}
				out_stream.push(coding.termpose_coding.closing_bracket as u8 as char);
			}
		}
	}
	pub fn to_string_with_coding(&self, coding:&Coding)-> String {
		let mut stream = String::with_capacity(16);
		self.stringifying(&mut stream, &coding);
		stream
	}
}
impl ToString for Term{
	fn to_string(&self)-> String {
		let coding = Coding::default();
		self.to_string_with_coding(&coding)
	}
}







//begin Translation API/DSL

//nassumes v has length > 0. Panics in debug mode, unknown behavior in release.
fn seriously_get_tail<T>(v:&[T])-> &[T] {
	if let Some((_, tail)) = v.split_first() {
		tail
	}else{
		seriously_unreachable()
	}
}

fn serious_unwrap<T>(v:Option<T>)-> T {
	match v {
		Some(r)=> r,
		None=> seriously_unreachable(),
	}
}

fn err_at<T>(line:u32, column:u32, s:String)-> Result<T,Vec<TermposeError>> {
	Err(vec!(TermposeError{
		line:line,
		column:column,
		msg:s
	}))
}

fn errs<T>(v:&Term, s:String)-> Result<T,Vec<TermposeError>> {
	err_at(v.line(), v.column(), s)
}

///takes any number of Vec<TermposeError> and returns a Vec<TermposeError>

#[doc = "Needed for error reporting in reads (EG: \"expected Vec<${T::name()}>, found a Stri Term instead\"). You can implement a trivial Named by calling `NamedFor!(type);`."]
pub trait Named{  fn name()->String;  }
macro_rules! NamedFor {
	($name:ident) => (
		impl Named for $name {
			fn name()-> String { stringify!($name).to_string() }
		}
	)
}


pub trait Readable : Sized + Named { //needs to be Sized to use Self (and to fit in with collection Readables), needs to be Named for error reporting
	fn check(v:&Term)-> Result<Self, Vec<TermposeError>>;
	fn parse_and_check(v:&str)-> Result<Self,Vec<TermposeError>> {
		match Term::parse(v) {
			Ok(ref t)=> Self::check(t),
			Err(tpe)=> Err(vec![tpe]),
		}
	}
}
pub fn check<T:Readable>(v:&Term)-> Result<T,Vec<TermposeError>> { T::check(v) }
pub fn parse_and_check<T:Readable>(s:&str)-> Result<T,Vec<TermposeError>> { T::parse_and_check(s) }

pub trait Writable {
	fn to_term(&self)-> Term;}
pub fn termify<T:Writable>(v:&T)-> Term { v.to_term() }

pub trait Translatable : Readable + Writable {}


pub trait Reader<T>{
	fn check(&self, v:&Term)-> Result<T,Vec<TermposeError>>;}
pub trait Writer<T>{
	fn termify(&self, v:&T)-> Term;}
pub trait Translator<T> : Reader<T> + Writer<T> {}

impl<T:Readable> Reader<T> for () {
	fn check(&self, v:&Term)-> Result<T,Vec<TermposeError>> { T::check(v) }}
impl<T:Writable> Writer<T> for () {
	fn termify(&self, v:&T)-> Term { v.to_term() }}
impl<T:Translatable> Translator<T> for () {}
#[doc = "Naturally, translators can be automatically generated for default-translatable types."]
pub fn translator_for<T:Translatable>()-> Box<Translator<T>> { box () }
pub fn reader_for<T:Readable>()-> Box<Reader<T>> { box () }
pub fn writer_for<T:Writable>()-> Box<Writer<T>> { box () }




NamedFor!(bool);
impl Readable for bool {
	fn check(v:&Term)-> Result<bool,Vec<TermposeError>>{
		match v {
			&Stri{ref s, ..}=> match s.as_str() {
				"true" | "⊤" => Ok(true),
				"false" | "⊥" => Ok(false),
				_ => errs(v, format!("expected bool, found \"{}\"", s)),
			},
			_=> errs(v, "expected bool, found a list term".to_string()),
		}
	}
}
impl Writable for bool{
	fn to_term(&self)-> Term {
		Stri{s:self.to_string(), line:0,column:0}
	}
}
impl Translatable for bool {}

NamedFor!(i32);
impl Readable for i32 {
	fn check(v:&Term)-> Result<i32, Vec<TermposeError>>{
		match v {
			&Stri{ref s, ..}=> match i32::from_str_radix(s,10) {
				Ok(i) => Ok(i),
				Err(_) => errs(v, format!("expected i32, found \"{}\", which could not be parsed as i32.", s)) },
			_=> errs(v, "expected i32, found a list term".to_string()), }}}
impl Writable for i32 {
	fn to_term(&self)-> Term { Term::Stri{s:self.to_string(), line:0, column:0} }}
impl Translatable for i32 {}
NamedFor!(u32);
impl Readable for u32 {
	fn check(v:&Term)-> Result<u32, Vec<TermposeError>>{
		match v {
			&Stri{ref s, ..}=> match u32::from_str_radix(s,10) {
				Ok(i) => Ok(i),
				Err(_) => errs(v, format!("expected u32, found \"{}\", which could not be parsed as u32.", s)) },
			_=> errs(v, "expected u32, found a list term".to_string()), }}}
impl Writable for u32 {
	fn to_term(&self)-> Term { Term::Stri{s:self.to_string(), line:0, column:0} }}
impl Translatable for u32 {}
NamedFor!(i64);
impl Readable for i64 {
	fn check(v:&Term)-> Result<i64, Vec<TermposeError>>{
		match v {
			&Stri{ref s, ..}=> match i64::from_str_radix(s,10) {
				Ok(i) => Ok(i),
				Err(_) => errs(v, format!("expected i64, found \"{}\", which could not be parsed as i64.", s)) },
			_=> errs(v, "expected i64, found a list term".to_string()), }}}
impl Writable for i64 {
	fn to_term(&self)-> Term { Term::Stri{s:self.to_string(), line:0, column:0} }}
impl Translatable for i64 {}
NamedFor!(u64);
impl Readable for u64 {
	fn check(v:&Term)-> Result<u64, Vec<TermposeError>>{
		match v {
			&Stri{ref s, ..}=> match u64::from_str_radix(s,10) {
				Ok(i) => Ok(i),
				Err(_) => errs(v, format!("expected u64, found \"{}\", which could not be parsed as u64.", s)) },
			_=> errs(v, "expected u64, found a list term".to_string()), }}}
impl Writable for u64 {
	fn to_term(&self)-> Term { Term::Stri{s:self.to_string(), line:0, column:0} }}
impl Translatable for u64 {}

NamedFor!(String);
impl Readable for String {
	fn check(v:&Term)-> Result<String, Vec<TermposeError>>{
		match v {
			&Stri{ref s, ..}=> Ok(s.clone()),
			_=> errs(v, "expected String, found a list term".to_string()),
		}
	}
}
impl Writable for String {
	fn to_term(&self)-> Term {
		Stri{s:self.clone(), line:0,column:0}
	}
}
impl Translatable for String {}

NamedFor!(Term);
impl Readable for Term {
	fn check(v:&Term)-> Result<Term, Vec<TermposeError>>{
		Ok(v.clone())
	}
}
impl Writable for Term {
	fn to_term(&self)-> Term {
		self.clone()
	}
}
impl Translatable for Term {}


impl<T:Named> Named for Vec<T>{fn name()->String { format!("Vec<{}>",T::name()) }}
#[inline(always)]
fn check_vec<T:Named, R:Reader<T>+?Sized>(v:&Term, r:&R, drops_nonmatches:bool)-> Result<Vec<T>,Vec<TermposeError>> {
	match v {
		&Stri{..}=> errs(v,format!("expected {}, found Stri Term.", Vec::<T>::name())),
		&List{ref l, ..}=> {
			if drops_nonmatches {
				let mut results = Vec::with_capacity(l.len());
				for t in l {
					match r.check(t) {
						Ok(vt)=> { results.push(vt); },
						Err(_)=> {},
					}
				}
				Ok(results)
			}else{
				let mut errors = Vec::new();
				let mut results = Vec::with_capacity(l.len());
				for t in l {
					match r.check(t) {
						Ok(vt)=> if errors.len() == 0 { results.push(vt); }, //else we don't need it
						Err(mut ers)=> errors.append(&mut ers),
					}
				}
				if errors.len() > 0 {
					Err(errors)
				}else{
					Ok(results)
				}
			}
		}
	}
}
impl<T:Readable> Readable for Vec<T> {
	fn check(v:&Term)-> Result<Vec<T>,Vec<TermposeError>> {
		check_vec(v, reader_for::<T>().as_ref(), false)
	}
}
impl<T:Writable> Writable for Vec<T> {
	fn to_term(&self)-> Term {
		List{l:self.iter().map(|t| t.to_term()).collect(), line:0,column:0}
	}
}
pub struct VecReader<T>{
	drops_nonmatches:bool,
	r:Box<Reader<T>>,
}
impl<T:Named> Reader<Vec<T>> for VecReader<T>{
	fn check(&self, v:&Term)-> Result<Vec<T>,Vec<TermposeError>> {
		check_vec(v, &*self.r, self.drops_nonmatches)
	}
}

#[derive(PartialEq,Copy,Clone)]
pub enum DuplicateHandling{ DROP, OVERWRITE, ERROR }
impl<K:Named,V:Named> Named for HashMap<K,V>{fn name()->String { format!("HashMap<{},{}>",K::name(),V::name()) }}
#[inline(always)]
fn check_hashmap<K:Eq+Hash+Named, V:Eq+Hash+Named, KR:Reader<K>+?Sized, VR:Reader<V>+?Sized>(v:&Term, kr:&KR, vr:&VR, dh:DuplicateHandling)-> Result<HashMap<K,V>,Vec<TermposeError>> {
	match v {
		&Stri{..}=> errs(v,format!("expected {}, found Stri Term.", HashMap::<K,V>::name())),
		&List{ref l, ..}=> {
			let mut errors:Vec<TermposeError> = Vec::new();
			let mut results = HashMap::with_capacity(l.len());
			for t in l {
				match t {
					&List{ref l, line, column}=> if l.len() == 2 {
						let mut krc = kr.check(&l[0]);
						let mut vrc = vr.check(&l[0]);
						let mut decent = errors.len() == 0;
						match krc {
							Ok(ref k)=>
								if dh == DuplicateHandling::ERROR && results.contains_key(k) {
									errors.push(TermposeError{line:line, column:column, msg:"duplicate entry in map".to_string()});
									decent = false;
								},
							Err(ref mut ers)=> {
								errors.append(ers);
								decent = false;
							}
						}
						match vrc {
							Ok(_)=> {},
							Err(ref mut ers)=> {
								errors.append(ers);
								decent = false;
							}
						}
						if decent {
							if let (Ok(k), Ok(v)) = (krc, vrc) {
								match dh {
									DuplicateHandling::DROP => {},
									DuplicateHandling::OVERWRITE | DuplicateHandling::ERROR =>{
										results.insert(k, v);
									}
								}
							}else{
								seriously_unreachable()
							}
						}
					}else{
						errors.push(termpose_error_from_term(t, format!("expected ({}, {}) pair, found a List Term with length {}", K::name(), V::name(), l.len())));
					},
					_=> errors.push(termpose_error_from_term(t, format!("expected ({}, {}) pair, found Stri Term", K::name(), V::name()))),
				}
			}
			if errors.len() > 0 {
				Err(errors)
			}else{
				Ok(results)
			}
		}
	}
}
impl<K:Readable+Eq+Hash+Named,V:Readable+Eq+Hash+Named> Readable for HashMap<K,V> {
	fn check(v:&Term)-> Result<HashMap<K,V>,Vec<TermposeError>> {
		check_hashmap(v, reader_for::<K>().as_ref(), reader_for::<V>().as_ref(), DuplicateHandling::ERROR)
	}
}
impl<K:Writable+Eq+Hash,V:Writable+Eq+Hash> Writable for HashMap<K,V> {
	fn to_term(&self)-> Term {
		list(self.iter().map(|(k,v)|{
			list(vec![k.to_term(), v.to_term()])
		}).collect())
	}
}
pub struct HashMapReader<K:Eq+Hash+Named,V:Eq+Hash+Named>{
	kr:Box<Reader<K>>,
	vr:Box<Reader<V>>,
	duplicate_handling:DuplicateHandling,
}
impl<K:Eq+Hash+Named,V:Eq+Hash+Named> Reader<HashMap<K,V>> for HashMapReader<K,V> {
	fn check(&self, v:&Term)-> Result<HashMap<K,V>,Vec<TermposeError>> {
		check_hashmap(v, self.kr.as_ref(), self.vr.as_ref(), self.duplicate_handling)
	}
}


pub struct OptionalReader<T>{r:Box<Reader<T>>}
impl<T> Reader<Option<T>> for OptionalReader<T> {
	fn check(&self, v:&Term)-> Result<Option<T>,Vec<TermposeError>> {
		match self.r.check(v) {
			Ok(o)=> Ok(Some(o)),
			Err(_)=> Ok(None),
		}
	}
}

// struct FunctionReader<T>{(r:Reader<T>)...}

///#[derive(StructTranslatable)] to generate a default implementation for a struct with Translatable members. StructTranslatable requires Named, but Named also has autoderivation.
pub trait StructTranslatable : Sized+Named {
	///if `named`, expects the term to be headed with the name of the struct. If `annotated`, expects each member of the struct to be a name:value pair. If `no_unknown_fields`, expects only the known fields to be present in the struct.
	fn check_struct(v:&Term, named:bool, annotated:bool, no_unknown_fields:bool)-> Result<Self,Vec<TermposeError>>;
	///if `named`, produces a term headed with the name of the struct. If `annotated`, the termification of each member of the struct will be paired with its name.
	fn termify_struct(&self, named:bool, annotated:bool)-> Term;
}
impl<T:StructTranslatable+Named> Readable for T {
	fn check(v:&Term)-> Result<Self,Vec<TermposeError>> { Self::check_struct(v,true,true,true) } }
impl<T:StructTranslatable> Writable for T {
	fn to_term(&self)-> Term { self.termify_struct(true,true) } }
impl<T:StructTranslatable+Named> Translatable for T {}
pub struct StructTranslator {
	named:bool, annotated:bool, no_unknown_fields:bool
}
impl<T:StructTranslatable> Reader<T> for StructTranslator {
	fn check(&self, v:&Term)-> Result<T,Vec<TermposeError>> { T::check_struct(v, self.named, self.annotated, self.no_unknown_fields) }}
impl<T:StructTranslatable> Writer<T> for StructTranslator {
	fn termify(&self, v:&T)-> Term { v.termify_struct(self.named, self.annotated) }}
impl<T:StructTranslatable> Translator<T> for StructTranslator {}
pub fn struct_translator<T:StructTranslatable>(named:bool, annotated:bool, no_unknown_fields:bool)-> Box<Translator<T>> {
	box StructTranslator{named:named, annotated:annotated, no_unknown_fields:no_unknown_fields}
}

struct AnnotatedTranslator<T>{annotation:String, r:Box<Translator<T>>}
// trait AnnotatedTranslatable<T:Translatable>
fn annotated_read<T:Named, R:Reader<T>+?Sized>(v:&Term, annotation:&str, r:&R)-> Result<T,Vec<TermposeError>> {
	let err = |found:&str| errs(v, format!("expected (\"{}\", {}), {}", annotation, T::name(), found));
	match *v {
		List{ref l, ..}=> if l.len() == 2 {
			match unsafe{l.get_unchecked(0)} {
				&Stri{ref s, line, column}=> if s == annotation {
					r.check(unsafe{l.get_unchecked(1)})
				} else {
					err_at(line, column, format!("expected literal \"{}\", found \"{}\"", annotation, s))
				},
				_=> err("but the term in the place of the literal was not a Stri term")
			}
		}else{
			err("but the List term was the wrong length")
		},
		_ => err("found Stri term"),
	}
}
fn annotated_write<T, W:Writer<T>+?Sized>(v:&T, annotation:&str, r:&W)-> Term {
	list(vec!(stri(annotation.to_string()), r.termify(v)))
}
impl<T:Named> Reader<T> for AnnotatedTranslator<T> {
	fn check(&self, v:&Term)-> Result<T,Vec<TermposeError>> { annotated_read(v, &self.annotation, self.r.as_ref()) }}
impl<T> Writer<T> for AnnotatedTranslator<T> {
	fn termify(&self, v:&T)-> Term { annotated_write(v, &self.annotation, self.r.as_ref()) }}

#[doc = "
example:
~~~
#[derive(StructTranslatable)]
struct Hom{  name:String, birthyear:u32, favourite_color:String  }
struct_named_annotated::<Hom>().parse_and_check(\"hom name:lucy birthyear:1992 favourite_color:lime\").is_ok()
~~~"]

// #[doc = "~~~
// example: named_struct!(Hom)::parse_and_check(\"hom lucy 1992 lime\").is_ok()
// ~~~"]
// macro_rules! named_struct {
	
// }
// #[doc = "~~~
// example: tuple_struct!(Hom)::parse_and_check(\"lucy 1992 lime\").is_ok()
// ~~~"]
// macro_rules! tuple_struct {
	
// }


pub mod translation_dsl {
	use super::*;
	use std::collections::HashMap;
	use std::hash::Hash;
	pub fn bool()-> Box<Translator<bool>> { translator_for() }
	pub fn u32()-> Box<Translator<u32>> { translator_for() }
	pub fn u64()-> Box<Translator<u64>> { translator_for() }
	pub fn i32()-> Box<Translator<i32>> { translator_for() }
	pub fn i64()-> Box<Translator<i64>> { translator_for() }
	pub fn term()-> Box<Translator<Term>> { translator_for() }
	pub fn string()-> Box<Translator<String>> { translator_for() }
	#[doc = "Identity function, equivalent to clone()"]
	pub fn vec<T:Named+'static>(r:Box<Reader<T>>)-> Box<Reader<Vec<T>>> {
		Box::new(VecReader{r:r, drops_nonmatches:false}) }
	#[doc = "Ignores elements that don't match `r`. As such, always returns a result."]
	pub fn vec_permissive<T:Named+'static>(r:Box<Reader<T>>)-> Box<Reader<Vec<T>>> {
		Box::new(VecReader{r:r, drops_nonmatches:true }) }
	#[doc = "Duplicate keys are considered an error"]
	pub fn hashmap<K:Eq+Hash+Named+'static, V:Eq+Hash+Named+'static>(kr:Box<Reader<K>>,vr:Box<Reader<V>>)-> Box<Reader<HashMap<K,V>>> { 
		box HashMapReader{kr:kr, vr:vr, duplicate_handling:DuplicateHandling::ERROR} }
	pub fn hashmap_ignore_duplicates<K:Eq+Hash+Named+'static, V:Eq+Hash+Named+'static>(kr:Box<Reader<K>>,vr:Box<Reader<V>>)-> Box<Reader<HashMap<K,V>>> {
		box HashMapReader{kr:kr, vr:vr, duplicate_handling:DuplicateHandling::DROP} }
	pub fn hashmap_duplicates_overwrite<K:Eq+Hash+Named+'static, V:Eq+Hash+Named+'static>(kr:Box<Reader<K>>,vr:Box<Reader<V>>)-> Box<Reader<HashMap<K,V>>> {
		box HashMapReader{kr:kr, vr:vr, duplicate_handling:DuplicateHandling::OVERWRITE} }
	#[doc = "An optional reader never fails to return a result. If input is not matched by `r`, result will be None."]
	pub fn optional<T:'static>(r:Box<Reader<T>>)-> Box<Reader<Option<T>>> { box OptionalReader{r:r} }
}


#[cfg(test)]
mod tests {
	use super::*;
	fn equiv(intext:&str, result:&str){
		assert_eq!(Term::parse_multiline(&intext.to_string()).map(|t|{t.to_string()}), Ok(result.to_string()));
	}
	#[test]
	fn basic(){
		equiv("a b c", "((a b c))");
		equiv(
	"a b:
	 c
	 d", "((a (b c d)))");
		equiv("avoculous \"smedl o bod()\"", "((avoculous \"smedl o bod()\"))");
	}

	#[test]
	fn coding(){
		let equiv_with_coding = |intext:&str, coding:&Coding, result:&str|{
			assert_eq!(Term::parse_multiline_with_coding(&intext.to_string(), coding).map(|t|{t.to_string()}), Ok(result.to_string()));
		};
		let dc = Coding::cli();
		equiv_with_coding(
			"l[termpose] add[no more[any] hab(djdj)aka] defy=gods", &dc,
			"(((l termpose) (add no (more any) \"hab(djdj)aka\") (defy gods)))"
		);
	}
	
	#[test]
	fn translation(){
		assert_eq!(
			parse_and_check("happy"),
			Ok("happy".to_string())
		);
		assert_eq!(
			parse_and_check("happy happy gold boy"),
			Ok(vec!["happy".to_string(), "happy".to_string(), "gold".to_string(), "boy".to_string()])
		);
	}
	
	#[derive(Named, StructTranslatable)]
	struct Hom{
		name:String, birth_year:u32,
	}
	
	// #[test]
	// fn struct_translation(){
	// 	let molly = Hom{name:"molly".to_string(), birth_year:1992};
	// 	assert_eq!(
	// 		molly.termify_struct(true,true).to_string(),
	// 		"(Hom (name molly) (birth_year 1992))"
	// 	);
	// }
}