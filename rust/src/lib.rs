#![feature(test)]
// #![feature(nll)]
#![feature(unreachable)]

//termpose. A library you can do sexpy things with

// extern crate string_cache;
// use string_cache::DefaultAtom as Strin;
use std::mem::{forget, replace, uninitialized, unreachable};
use std::cmp::{PartialEq};
use std::result::Result;
use std::error::Error;
use std::fmt::{Formatter, Display, Debug};
use std::borrow::{Borrow};
use std::ptr::{null_mut};
// use std::debug_assert;
extern crate ref_slice;
use ref_slice::ref_slice;


// pub trait Term where Self:Sized {
// 	fn initial_string(& self)-> &str;
// 	fn contents<'a>(&'a self)-> std::slice::Iter<'a, Self>;
// 	fn tail<'a>(&'a self)-> std::slice::Iter<'a, Self>;
// }



// impl<'a, St> PartialEq for &'a Terms<St> {
// 	fn eq(&self, other: &&Terms<St>)-> bool {
// 		self == other
// 	}
// }

impl PartialEq for Term {
	fn eq(&self, other: &Self)-> bool { //line numbers are not considered important
		match *self {
			Atomv(ref sa)=> {
				match *other {
					Atomv(ref so)=> sa.v == so.v,
					_=> false,
				}
			}
			Listv(ref la)=> {
				match *other {
					Listv(ref lo)=> la.v == lo.v,
					_=> false,
				}
			}
		}
	}
}


pub struct List {
	pub line: isize,
	pub column: isize,
	pub v: Vec<Term>,
}
pub struct Atom {
	pub line: isize,
	pub column: isize,
	pub v: String,
}
pub enum Term {
	Listv(List),
	Atomv(Atom),
}
pub use Term::*;


#[macro_export]
macro_rules! list {
	($(e),*) => ( Term::Listv(List{line:-1, column:-1, v:vec!($(($e).into()),*)}) )
}

impl<'a> Into<Term> for &'a str {
	fn into(self) -> Term { Atomv(Atom{ line:-1, column:-1, v:self.to_string() }) }
}
impl<'a> Into<Term> for Vec<Term> {
	fn into(self) -> Term { Listv(List{ line:-1, column:-1, v:self }) }
}

fn tail<I:Iterator>(mut v:I)-> I {
	v.next();
	v
}

// fn stringify_with_separators<T:Deref<Target=str>, I:Iterator<Item=T>>(out_buf:&mut String, separator:&str, mut i:I){
// 	if let Some(ref first) = i.next() {
// 		out_buf.push_str(&**first);
// 		while let Some(ref nexto) = i.next() {
// 			out_buf.push_str(separator);
// 			out_buf.push_str(&**nexto);
// 		}
// 	}
// }

fn push_escaped(take:&mut String, give:&str){
	for c in give.chars() {
		match c {
			'\n'=> { take.push('\\'); take.push('n'); }
			'\t'=> { take.push('\\'); take.push('t'); }
			'"'=> { take.push('\\'); take.push('"'); }
			_=> { take.push(c); }
		}
	}
}

impl Term {
	pub fn atom(v:String)-> Term    { Term::Atomv(Atom{line:-1, column:-1, v:v}) }
	pub fn list(v:Vec<Term>)-> Term { Term::Listv(List{line:-1, column:-1, v:v}) }
	pub fn line_and_col(&self)-> (isize, isize) {
		match *self {
			Term::Listv(ref l)=> (l.line, l.column),
			Term::Atomv(ref a)=> (a.line, a.column),
		}
	}
	pub fn initial_string(&self)-> &str { //if it bottoms out at an empty list, it returns the empty str
		match *self {
			Listv(ref v)=> {
				if let Some(ref ss) = v.v.first() {
					ss.initial_string()
				}else{
					""
				}
			}
			Atomv(ref v)=> v.v.as_str(),
		}
	}
	pub fn to_string(&self)-> String {
		let mut ret = String::new();
		self.stringify(&mut ret);
		ret
	}
	pub fn stringify(&self, s:&mut String){
		match *self {
			Listv(ref v)=> {
				s.push('(');
				//spaces go between
				let mut i = v.v.iter();
				if let Some(ref first) = i.next() {
					first.stringify(s);
					while let Some(ref nexto) = i.next() {
						s.push(' ');
						nexto.stringify(s);
					}
				}
				s.push(')');
			}
			Atomv(ref v)=> {
				let needs_quotes = v.v.chars().any(|c|{ c == ' ' || c == '\t' || c == ':' || c == '(' || c == ')' });
				if needs_quotes { s.push('"'); }
				push_escaped(s, v.v.as_str());
				if needs_quotes { s.push('"'); }
			}
		}
	}
	pub fn contents(&self)-> std::slice::Iter<Self> { //if Atom, returns a slice iter of a single element that is the atom's str
		match *self.borrow() {
			Listv(ref v)=> v.v.iter(),
			Atomv(_)=> ref_slice(self).iter(),
		}
	}
	pub fn tail<'b>(&'b self)-> std::slice::Iter<'b, Self> { //if Atom, returns an empty slice iter
		match *self.borrow() {
			Listv(ref v)=> tail((*v).v.iter()),
			Atomv(_)=> [].iter(),
		}
	}
	pub fn find(&self, key:&str)-> Option<&Term> {
		self.contents().find(|el| el.initial_string() == key)
	}
}

#[macro_export]
macro_rules! list {
	($($inner:expr),*)=> {{
		Listv(List{line:-1, column:-1, v:vec!($($inner.into()),*)})
	}};
}


mod translators; pub use translators::*;
mod parsers; pub use parsers::*;

