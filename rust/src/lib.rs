#![feature(test)]
// #![feature(nll)]
#![feature(drain_filter)]
#![feature(core_intrinsics)]

// extern crate string_cache;
// use string_cache::DefaultLeaf as Strin;
use std::str::FromStr;
use std::cmp::{PartialEq};
use std::result::Result;
use std::error::Error;
use std::fmt::{Formatter, Display, Debug};
use std::mem::forget;
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

/// Line numbers aren't checked in equality comparisons
impl PartialEq for Wood {
	fn eq(&self, other: &Self)-> bool {
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
	pub fn line(&self)-> isize { match *self { Leafv(ref s)=> s.line, Branchv(ref s)=> s.line } }
	pub fn col(&self)-> isize { match *self { Leafv(ref s)=> s.column, Branchv(ref s)=> s.column } }
	pub fn line_and_col(&self)-> (isize, isize) {
		match *self {
			Wood::Branchv(ref l)=> (l.line, l.column),
			Wood::Leafv(ref a)=> (a.line, a.column),
		}
	}
	/// Seeks the earliest string in the tree by looking at the first element of each branch recursively until it hits a leaf. (If it runs into an empty list, returns the empty string.
	/// I recommend this whenever you want to use the first string as a discriminator, or whenever you want to get leaf str contents in general.
	/// This abstracts over similar structures in a way that I consider generally desirable. I would go as far as to say that the more obvious, less abstract way of getting initial string should be Considered Harmful.
	/// A few motivating examples:
	/// If you wanted to add a feature to a programming language that allows you to add a special tag to an invocation, you want to put the tag inside the invocation's ast node but you don't want it to be confused for a parameter, this pattern enables:
	/// ((f tag(special_invoke_inline)) a b)
	/// If the syntax of a sexp language had more structure to it than usual:
	/// ((if condition) then...) would still get easily picked up as an 'if' node.
	/// Annotations are a good example, more generally, if you're refactoring and you decide you want to add an extra field to what was previously a leaf, this pattern enables you to make that change, confident that your code will still read its string content in the same way
	/// (list key:value "some prose") -> (list key:value ("some prose" modifier:italicise))
	pub fn initial_str(&self)-> &str {
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
	
	pub fn strip_comments_escape(&mut self, comment_str:&str, comment_escape_str:&str) {
		match *self {
			Branchv(ref mut b)=> {
				b.v.drain_filter(|i|{
					match *i {
						Branchv(ref mut b)=> {
							if b.v.len() > 0 {
								match b.v[0] {
									Leafv(Leaf{ref v, ..})=> {
										if v == comment_str {
											return true;
										}
									}
									_=> {}
								}
							}
						}
						_=> {}
					}
					i.strip_comments_escape(comment_str, comment_escape_str);
					false
				});
			}
			Leafv(ref mut l)=> {
				if &l.v == comment_escape_str {
					l.v = comment_str.into();
				}
			}
		}
	}
	
	/// Strips any branches with first element of comment_str. If you need to produce a leaf that is equivalent to comment_str.
	/// If you need the wood to contain a leaf that is the comment_str, you can escape it with a backslash.
	/// This is actually a highly flawed way of providing commenting, because this will also strip out any serialization of a list of strings where the first element happens to equal the `comment_str`. That's a really subtle error, that violates a lot of expectations.
	/// You could get around it by escaping your wood so that any strs that resemble comment tags wont read that way, but it's a bit awkward and sometimes wont really work.
	pub fn strip_comments(&mut self, comment_str:&str){
		let escstr = format!("\\{}", comment_str);
		self.strip_comments_escape(comment_str, &escstr);
	}
	
	/// if Leaf, returns a slice iter containing just this, else Branch, iterates over branch contents
	pub fn contents(&self)-> std::slice::Iter<Self> {
		match *self.borrow() {
			Branchv(ref v)=> v.v.iter(),
			Leafv(_)=> ref_slice(self).iter(),
		}
	}
	/// returns the first wood, or if it's a leaf, itself
	pub fn head(&self)-> Result<&Wood, Box<WoodError>> {
		self.contents().next().ok_or_else(|| Box::new(WoodError::new(self, "there shouldn't be an empty list here".to_string())))
	}
	/// returns the second wood within this one, if it is a list wood, if there is a second wood
	pub fn second(&self)-> Result<&Wood, Box<WoodError>> {
		self.tail().next().ok_or_else(|| Box::new(WoodError::new(self, "a second wood was supposed to be present".to_string())))
	}
	/// if Leaf, returns an empty slice iter, if Branch, returns contents after the first element
	pub fn tail<'b>(&'b self)-> std::slice::Iter<'b, Self> {
		match *self.borrow() {
			Branchv(ref v)=> tail((*v).v.iter()),
			Leafv(_)=> [].iter(),
		}
	}
	/// `self.contents().find(|el| el.initial_str() == key)`
	pub fn seek<'a, 'b>(&'a self, key:&'b str)-> Option<&'a Wood> {
		self.contents().find(|el| el.initial_str() == key)
	}
	
	/// returns the first child term with initial_str == key, or if none is found, an error
	pub fn find<'a, 'b>(&'a self, key:&'b str)-> Result<&'a Wood, Box<WoodError>> {
		self.seek(key).ok_or_else(|| Box::new(WoodError::new(
			self,
			format!("could not find child with key \"{}\"", key),
		)))
	}
	/// find(self, key).and_then(|v| v.second())
	pub fn find_val<'a, 'b>(&'a self, key:&'b str)-> Result<&'a Wood, Box<WoodError>> {
		self.find(key).and_then(|v| v.second())
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
	fn dewoodify(&self, v:&Wood) -> Result<T, Box<WoodError>>;
}

#[derive(Debug)]
pub struct WoodError{
	pub line:isize,
	pub column:isize,
	pub msg:String,
	pub cause:Option<Box<dyn Error>>,
}
impl Display for WoodError {
	fn fmt(&self, f: &mut Formatter) -> std::fmt::Result {
		Debug::fmt(self, f)
	}
}
impl WoodError {
	pub fn new(source: &Wood, msg:String) -> Self {
		let (line, column) = source.line_and_col();
		Self{ line, column, msg, cause:None }
	}
	pub fn new_with_cause(source:&Wood, msg:String, cause:Box<dyn Error>) -> Self {
		let (line, column) = source.line_and_col();
		WoodError{ line, column, msg, cause:Some(cause) }
	}
}
impl Error for WoodError {
	fn cause(&self) -> Option<&dyn Error> { self.cause.as_ref().map(|e| e.as_ref()) }
}


pub trait Woodable {
	fn woodify(&self) -> Wood;
}
pub trait Dewoodable {
	fn dewoodify(v:&Wood) -> Result<Self, Box<WoodError>> where Self:Sized;
}

pub fn woodify<T>(v:&T) -> Wood where T: Woodable {
	v.woodify()
}
pub fn dewoodify<T>(v:&Wood) -> Result<T, Box<WoodError>> where T: Dewoodable {
	T::dewoodify(v)
}


/// parse_termpose(v).and_then(dewoodify)
pub fn deserialize<T>(v:&str) -> Result<T, Box<WoodError>> where T : Dewoodable {
	parse_termpose(v).and_then(|w| dewoodify(&w))
}
/// woodify(v).to_string()
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
			fn dewoodify(v:&Wood) -> Result<$Type, Box<WoodError>> {
				$Type::from_str(v.initial_str()).map_err(|er|{
					Box::new(WoodError::new_with_cause(v, format!("couldn't parse {}", stringify!($name)), Box::new(er)))
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
	fn dewoodify(v:&Wood) -> Result<Self, Box<WoodError>> {
		match v.initial_str() {
			"true" | "⊤" | "yes" => {
				Ok(true)
			},
			"false" | "⟂" | "no" => {
				Ok(false)
			},
			_=> Err(Box::new(WoodError::new(v, "expected a bool here".into())))
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
	fn dewoodify(v:&Wood) -> Result<Self, Box<WoodError>> {
		match *v {
			Leafv(ref a)=> Ok(a.v.clone()),
			Branchv(_)=> Err(Box::new(WoodError::new(v, "sought string, found branch".into()))),
		}
	}
}

pub fn woodify_seq_into<'a, InnerTran, T, I>(inner:&InnerTran, v:I, output:&mut Vec<Wood>)
	where InnerTran: Wooder<T>, I:Iterator<Item=&'a T>, T:'a
{
	for vi in v { output.push(inner.woodify(vi)); }
}
pub fn dewoodify_seq_into<'a, InnerTran, T, I>(inner:&InnerTran, v:I, output:&mut Vec<T>) -> Result<(), Box<WoodError>>
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
		woodify_seq_into(&wooder::Iden, self.iter(), &mut ret);
		ret.into()
	}
}
impl<T> Dewoodable for Vec<T> where T:Dewoodable {
	fn dewoodify(v:&Wood) -> Result<Vec<T>, Box<WoodError>> {
		let mut ret = Vec::new();
		dewoodify_seq_into(&wooder::Iden, v.contents(), &mut ret)?;
		Ok(ret)
	}
}





mod parsers; pub use parsers::*;

pub mod wooder;



#[cfg(test)]
mod tests {
	extern crate test;
	use super::*;
	
	#[test]
	fn test_comment_stripping() {
		let mut w = parse_multiline_termpose("

#\"this function cannot be called with a false object criterion
fn process_ishm_protocol_handling object criterion
	if criterion(object)
		#\"then it returns good
		return good
	else
		return angered
	return secret_egg

").unwrap();
		w.strip_comments("#");
		
		let should_be = parse_multiline_termpose("

fn process_ishm_protocol_handling object criterion
	if criterion(object)
		return good
	else
		return angered
	return secret_egg

").unwrap();
		
		assert_eq!(&w, &should_be);
	}
}