#![feature(core_intrinsics)]
#![feature(slice_patterns)]
#![feature(box_patterns)]
#![feature(libc)]

use std::ffi::CStr;
use std::ffi::CString;
use std::string::String;
use std::result::{Result};
use std::ops::Deref;
use std::mem::uninitialized;

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

#[link(name = "termpose")]
extern "C" {
	fn isStr(v:*const TermposeTerm)-> bool;
	
	// fn parseAsList(unicodeString:*const c_char, errorOut:*mut *mut c_char)-> *mut TermposeTerm; //contents will be contained in a root list even if there is only a single line at root (if you don't want that, see parse()). If there is an error, return value will be null and errorOut will be set to contain the text. If there was no error, errorOut will be set to null. errorOut can be null, if a report is not desired.
	// fn parseWithCoding(unicodeString:*const c_char, errorOut:*mut *mut c_char, coding:*const TermposeCoding)-> *mut TermposeTerm;
	// fn parseLengthedToSeqs(unicodeString:*const c_char, length:u32, errorOut:*mut *mut c_char)-> *mut TermposeTerm;
	fn parseLengthedToSeqsWithCoding(unicodeString:*const c_char, length:u32, errorOut:*mut *mut c_char, coding:*const TermposeCoding)-> *mut TermposeTerm;
	// fn parse(unicodeString:*const c_char, errorOut:*mut *mut c_char)-> *mut TermposeTerm; //same as above but root line will not be wrapped in a root seqs iff there is only one root element. This is what you want if you know there's only one line in the string you're parsing, but use parseAsList if there could be more(or less) than one, as it is more consistent
	// fn stringifyTerm(t:*const TermposeTerm)-> *mut c_char;
	
	fn destroyTerm(v:*mut TermposeTerm);
	// fn drainTerm(v:*mut TermposeTerm);
	fn destroyStr(str:*mut c_char);
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

pub enum Term{
	List{l:Vec<Term>, line:u32, column:u32},
	Stri{s:String, line:u32, column:u32},
}
pub use Term::*;

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
}

fn parse_to_seqs(s:&String, coding:&Coding)-> Result<Terms,String> {
	let mut err_out:*mut c_char = unsafe{uninitialized()}; //gets initialized by parseLengthedToSeqs
	let res:*mut TermposeTerm = unsafe{ parseLengthedToSeqsWithCoding(s.as_ptr() as *const c_char, s.len() as u32, &mut err_out, &coding.termpose_coding) };
	let ret:Result<Terms,String>;
	if err_out.is_null() {
		ret = Ok(Terms{v:res});
	}else{
		ret = Err(format!("parseError: {}", unsafe{CStr::from_ptr(err_out)}.to_str().unwrap()));
		unsafe{destroyStr(err_out)};
	}
	ret
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
	pub fn parse_multiline(s:&String)-> Result<Term,String> {
		Term::parse_multiline_with_coding(s, &Coding::default())
	}
	pub fn parse_multiline_with_coding(s:&String, coding:&Coding)-> Result<Term,String> {
		parse_to_seqs(s, coding).map(|terms|{
			unsafe{Term::from_cterm(terms.v)}
		})
	}
	pub fn parse_with_coding(s:&String, c:&Coding)-> Result<Term,String> {
		Term::parse_multiline_with_coding(s, c).map(|term|{
			let should_extract = match term {
				List{ref l, ..}=>
					if l.len() == 1 {
						true
					}else{
						false
					},
				Stri{ .. }=> unsafe{std::intrinsics::unreachable()},
			};
			if should_extract {
				match term {
					List{l, ..}=> unsafe{extract_first(l)},
					Stri{..}=> unsafe{std::intrinsics::unreachable()},
				}
			}else{
				term
			}
		})
	}
	#[doc = "This may be preferable to parse_multiline when you are sure that the input text will have just one term at root, in which case it will not wrap that term in a root list. If the text has zero or more than one, though, it will behave as parse_multiline, which may catch you by surprise. parse_multiline is provided to avoid this potential irregularity."]
	pub fn parse(s:&String)-> Result<Term,String> {
		Term::parse_with_coding(s, &Coding::default())
	}
	fn stringifying(&self, out_stream:&mut String, coding:&Coding){ //TODO this is one of those egregious piles of abjection that I had to write before rust was good. Use proper variant types and better borrow scoping to fix this.
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




#[cfg(test)]
mod tests {
	use super::{Term, Coding};
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
}