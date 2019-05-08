#![feature(test)]
// #![feature(nll)]
#![feature(core_intrinsics)]

// extern crate string_cache;
// use string_cache::DefaultLeaf as Strin;
use std::str::FromStr;
use std::mem::{forget, replace, uninitialized};
use std::cmp::{PartialEq};
use std::result::Result;
use std::error::Error;
use std::fmt::{Formatter, Display, Debug};
use std::borrow::{Borrow};
use std::ptr::{null_mut};
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


#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Branch {
	pub line: isize,
	pub column: isize,
	pub v: Vec<Wood>,
}
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Leaf {
	pub line: isize,
	pub column: isize,
	pub v: String,
}
#[derive(Debug, Clone, Eq)]
pub enum Wood {
	Branchv(Branch),
	Leafv(Leaf),
}
pub use Wood::*;


impl Into<Wood> for String {
	fn into(self) -> Wood { Leafv(Leaf{ line:-1, column:-1, v:self }) }
}
impl<'a> Into<Wood> for &'a str {
	fn into(self) -> Wood { self.to_string().into() }
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
	pub fn is_leaf(&self)-> bool { match self { &Leafv(_)=> true, _=> false } }
	pub fn is_branch(&self)-> bool { match self { &Branchv(_)=> true, _=> false } }
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
		to_woodslist(self)
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



pub trait Wooder<T> {
	fn woodify(&self, v:&T) -> Wood;
}
pub trait Dewooder<T> {
	fn dewoodify(&self, v:&Wood) -> Result<T, DewoodifyError>;
}

#[derive(Debug)]
pub struct DewoodifyError{
	pub line:isize,
	pub column:isize,
	pub msg:String,
	pub cause:Option<Box<Error>>,
}
impl Display for DewoodifyError {
	fn fmt(&self, f: &mut Formatter) -> Result<(), std::fmt::Error> {
		Debug::fmt(self, f)
	}
}
impl DewoodifyError {
	pub fn new(source: &Wood, msg:String) -> Self {
		let (line, column) = source.line_and_col();
		Self{ line, column, msg, cause:None }
	}
	pub fn new_with_cause(source:&Wood, msg:String, cause:Option<Box<Error>>) -> Self {
		let (line, column) = source.line_and_col();
		DewoodifyError{ line, column, msg, cause }
	}
}

impl Error for DewoodifyError {
	fn description(&self) -> &str { self.msg.as_str() }
	fn cause(&self) -> Option<&Error> { self.cause.as_ref().map(|e| e.as_ref()) }
}
pub trait Woodable {
	fn woodify(&self) -> Wood;
}
pub trait Dewoodable {
	fn dewoodify(&Wood) -> Result<Self, DewoodifyError> where Self:Sized;
}

pub fn woodify<T>(v:&T) -> Wood where T: Woodable {
	v.woodify()
}
pub fn dewoodify<T>(v:&Wood) -> Result<T, DewoodifyError> where T: Dewoodable {
	T::dewoodify(v)
}

#[derive(Debug)]
pub enum WoodError{
	ParserError(PositionedError),
	DewoodifyError(DewoodifyError),
}

/// An extremely simple but probably not very useful function that maps straight from string to data without providing any opportunity for customization
pub fn deserialize<T>(v:&str) -> Result<T, WoodError> where T : Dewoodable {
	match parse_termpose(v) {
		Ok(t)=> dewoodify(&t).map_err(|e| WoodError::DewoodifyError(e)),
		Err(e)=> Err(WoodError::ParserError(e)),
	}
}
/// An extremely simple but probably not very useful function that maps straight from data to string without providing any opportunity for customization
pub fn serialize<T>(v:&T) -> String where T: Woodable {
	woodify(v).to_string()
}

macro_rules! do_basic_stringifying_woodable_for {
	($Type:ident) => (
		impl Woodable for $Type {
			fn woodify(&self) -> Wood { self.to_string().into() }
		}
	)
}
macro_rules! do_basic_destringifying_dewoodable_for {
	($Type:ident) => (
		impl Dewoodable for $Type {
			fn dewoodify(v:&Wood) -> Result<$Type, DewoodifyError> {
				$Type::from_str(v.initial_str()).map_err(|er|{
					DewoodifyError::new_with_cause(v, format!("couldn't parse {}", stringify!($name)), Some(Box::new(er)))
				})
			}
		}
	)
}

do_basic_stringifying_woodable_for!(char);
do_basic_destringifying_dewoodable_for!(char);
do_basic_stringifying_woodable_for!(u32);
do_basic_destringifying_dewoodable_for!(u32);
do_basic_stringifying_woodable_for!(u64);
do_basic_destringifying_dewoodable_for!(u64);
do_basic_stringifying_woodable_for!(i32);
do_basic_destringifying_dewoodable_for!(i32);
do_basic_stringifying_woodable_for!(i64);
do_basic_destringifying_dewoodable_for!(i64);
do_basic_stringifying_woodable_for!(f32);
do_basic_destringifying_dewoodable_for!(f32);
do_basic_stringifying_woodable_for!(f64);
do_basic_destringifying_dewoodable_for!(f64);
do_basic_stringifying_woodable_for!(isize);
do_basic_destringifying_dewoodable_for!(isize);
do_basic_stringifying_woodable_for!(usize);
do_basic_destringifying_dewoodable_for!(usize);

do_basic_stringifying_woodable_for!(bool);
impl Dewoodable for bool {
	fn dewoodify(v:&Wood) -> Result<Self, DewoodifyError> {
		match v.initial_str() {
			"true" | "⊤" | "yes" => {
				Ok(true)
			},
			"false" | "⟂" | "no" => {
				Ok(false)
			},
			_=> Err(DewoodifyError::new_with_cause(v, "expected a bool here".into(), None))
		}
	}
}

// impl<I, T> Woodable for I where I: Iterator<Item=T>, T:Woodable {
	
// }

impl Woodable for String {
	fn woodify(&self) -> Wood {
		self.as_str().into()
	}
}
impl Dewoodable for String {
	fn dewoodify(v:&Wood) -> Result<Self, DewoodifyError> {
		match *v {
			Leafv(ref a)=> Ok(a.v.clone()),
			Branchv(_)=> Err(DewoodifyError::new_with_cause(v, "sought string, found branch".into(), None)),
		}
	}
}

pub fn woodify_seq_into<'a, InnerTran, T, I>(inner:&InnerTran, v:I, output:&mut Vec<Wood>)
	where InnerTran: Wooder<T>, I:Iterator<Item=&'a T>, T:'a
{
	for vi in v { output.push(inner.woodify(vi)); }
}
pub fn dewoodify_seq_into<'a, InnerTran, T, I>(inner:&InnerTran, v:I, output:&mut Vec<T>) -> Result<(), DewoodifyError>
	where InnerTran: Dewooder<T>, I:Iterator<Item=&'a Wood>
{
	// let errors = Vec::new();
	for vi in v {
		match inner.dewoodify(vi) {
			Ok(vii)=> output.push(vii),
			Err(e)=> return Err(e),
		}
	}
	Ok(())
	// if errors.len() > 0 {
	// 	let msgs = String::new();
	// 	for e in errors {
	// 		msgs.push(format!("{}\n"))
	// 	}
	// }
}


impl<T> Woodable for Vec<T> where T:Woodable {
	fn woodify(&self) -> Wood {
		let mut ret = Vec::new();
		woodify_seq_into(&wooder::Central, self.iter(), &mut ret);
		ret.into()
	}
}
impl<T> Dewoodable for Vec<T> where T:Dewoodable {
	fn dewoodify(v:&Wood) -> Result<Vec<T>, DewoodifyError> {
		let mut ret = Vec::new();
		try!(dewoodify_seq_into(&wooder::Central, v.contents(), &mut ret));
		Ok(ret)
	}
}





mod parsers; pub use parsers::*;

pub mod wooder;

