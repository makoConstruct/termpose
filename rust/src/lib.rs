#![feature(test)]
// #![feature(nll)]
#![feature(core_intrinsics)]

// extern crate string_cache;
// use string_cache::DefaultLeaf as Strin;
use std::mem::{forget, replace, uninitialized};
use std::cmp::{PartialEq};
use std::result::Result;
use std::error::Error;
use std::fmt::{Formatter, Display, Debug};
use std::borrow::{Borrow};
use std::ptr::{null_mut};
// use std::debug_assert;
extern crate ref_slice;
use ref_slice::ref_slice;


// pub trait Wood where Self:Sized {
// 	type Iter:Iterator<Item=Self>;
// 	fn initial_str(& self)-> &str;
// 	fn contents<'a>(&'a self)-> Iter;
// 	fn tail<'a>(&'a self)-> Iter;
// }
// impl<'a, St> PartialEq for &'a Woods<St> {
// 	fn eq(&self, other: &&Woods<St>)-> bool {
// 		self == other
// 	}
// }

impl PartialEq for Wood {
	fn eq(&self, other: &Self)-> bool { //line numbers are not considered important
		match *self {
			Leafv(ref sa)=> {
				match *other {
					Leafv(ref so)=> sa.v == so.v,
					_=> false,
				}
			}
			Branchv(ref la)=> {
				match *other {
					Branchv(ref lo)=> la.v == lo.v,
					_=> false,
				}
			}
		}
	}
}


pub struct Branch {
	pub line: isize,
	pub column: isize,
	pub v: Vec<Wood>,
}
pub struct Leaf {
	pub line: isize,
	pub column: isize,
	pub v: String,
}
pub enum Wood {
	Branchv(Branch),
	Leafv(Leaf),
}
pub use Wood::*;


impl<'a> Into<Wood> for &'a str {
	fn into(self) -> Wood { Leafv(Leaf{ line:-1, column:-1, v:self.to_string() }) }
}
impl<'a> Into<Wood> for Vec<Wood> {
	fn into(self) -> Wood { Branchv(Branch{ line:-1, column:-1, v:self }) }
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

impl Wood {
	pub fn leaf(v:String)-> Wood    { Wood::Leafv(Leaf{line:-1, column:-1, v:v}) }
	pub fn branch(v:Vec<Wood>)-> Wood { Wood::Branchv(Branch{line:-1, column:-1, v:v}) }
	pub fn line_and_col(&self)-> (isize, isize) {
		match *self {
			Wood::Branchv(ref l)=> (l.line, l.column),
			Wood::Leafv(ref a)=> (a.line, a.column),
		}
	}
	pub fn initial_str(&self)-> &str { //if it bottoms out at an empty branch, it returns the empty str
		match *self {
			Branchv(ref v)=> {
				if let Some(ref ss) = v.v.first() {
					ss.initial_str()
				}else{
					""
				}
			}
			Leafv(ref v)=> v.v.as_str(),
		}
	}
	pub fn to_string(&self)-> String {
		let mut ret = String::new();
		self.stringify(&mut ret);
		ret
	}
	pub fn stringify(&self, s:&mut String){
		match *self {
			Branchv(ref v)=> {
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
			Leafv(ref v)=> {
				let needs_quotes = v.v.chars().any(|c|{ c == ' ' || c == '\t' || c == ':' || c == '(' || c == ')' });
				if needs_quotes { s.push('"'); }
				push_escaped(s, v.v.as_str());
				if needs_quotes { s.push('"'); }
			}
		}
	}
	pub fn contents(&self)-> std::slice::Iter<Self> { //if Leaf, returns a slice iter of a single element that is the leaf's str
		match *self.borrow() {
			Branchv(ref v)=> v.v.iter(),
			Leafv(_)=> ref_slice(self).iter(),
		}
	}
	pub fn tail<'b>(&'b self)-> std::slice::Iter<'b, Self> { //if Leaf, returns an empty slice iter
		match *self.borrow() {
			Branchv(ref v)=> tail((*v).v.iter()),
			Leafv(_)=> [].iter(),
		}
	}
	pub fn find(&self, key:&str)-> Option<&Wood> {
		self.contents().find(|el| el.initial_str() == key)
	}
}

#[macro_export]
macro_rules! branch {
	($($inner:expr),* $(,)*)=> {{
		$crate::Branchv($crate::Branch{line:-1, column:-1, v:vec!($($inner.into()),*)})
	}};
}


mod translators; pub use translators::*;
mod parsers; pub use parsers::*;

