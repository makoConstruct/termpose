
use super::*;
use std::str::FromStr;
use std::collections::HashMap;
use std::cmp::Eq;
use std::hash::Hash;
use std::iter::FromIterator;

pub trait Termer<T> : Clone {
	fn termify(&self, v:&T) -> Term;
}
pub trait Untermer<T> : Clone {
	fn untermify(&self, v:&Term) -> Result<T, UntermifyError>;
}

#[derive(Debug)]
pub struct UntermifyError{
	pub line:isize,
	pub column:isize,
	pub msg:String,
	pub cause:Option<Box<Error>>,
}
impl Display for UntermifyError {
	fn fmt(&self, f: &mut Formatter) -> Result<(), std::fmt::Error> {
		Debug::fmt(self, f)
	}
}
fn mk_untermify_error(problem:&Term, msg:String, cause:Option<Box<Error>>)-> UntermifyError {
	let (line, column) = problem.line_and_col();
	UntermifyError{ line, column, msg, cause }
}

impl Error for UntermifyError {
	fn description(&self) -> &str { self.msg.as_str() }
	fn cause(&self) -> Option<&Error> { self.cause.as_ref().map(|e| e.as_ref()) }
}

pub trait Bitermer<T> : Termer<T> + Untermer<T> {} //bidirectional termer and untermer


pub trait Termable {
	fn termify(&self)-> Term;
}
pub trait Untermable {
	fn untermify(&Term)-> Result<Self, UntermifyError> where Self:Sized;
}

#[derive(Clone)]
pub struct DefaultBitermer();
impl<T> Bitermer<T> for DefaultBitermer where T:Termable + Untermable {}
impl<T> Termer<T> for DefaultBitermer where T:Termable {
	fn termify(&self, v:&T) -> Term { v.termify() }
}
impl<T> Untermer<T> for DefaultBitermer where T:Untermable {
	fn untermify(&self, v:&Term) -> Result<T, UntermifyError> { T::untermify(v) }
}


#[derive(Copy, Clone)]
pub struct IsizeBi();
impl Bitermer<isize> for IsizeBi {}
impl Termer<isize> for IsizeBi {
	fn termify(&self, v:&isize) -> Term { v.to_string().as_str().into() }
}
impl Untermer<isize> for IsizeBi {
	fn untermify(&self, v:&Term) -> Result<isize, UntermifyError> {
		isize::from_str(v.initial_string()).map_err(|er|{
			mk_untermify_error(v, "couldn't parse isize".into(), Some(Box::new(er)))
		})
	}
}


#[derive(Copy, Clone)]
pub struct UsizeBi();
impl Bitermer<usize> for UsizeBi {}
impl Termer<usize> for UsizeBi {
	fn termify(&self, v:&usize) -> Term { v.to_string().as_str().into() }
}
impl Untermer<usize> for UsizeBi {
	fn untermify(&self, v:&Term) -> Result<usize, UntermifyError> {
		usize::from_str(v.initial_string()).map_err(|er|{
			mk_untermify_error(v, "couldn't parse usize".into(), Some(Box::new(er)))
		})
	}
}


#[derive(Copy, Clone)]
pub struct BoolBi();
impl Bitermer<bool> for BoolBi {}
impl Termer<bool> for BoolBi {
	fn termify(&self, v:&bool) -> Term { v.to_string().as_str().into() }
}
impl Untermer<bool> for BoolBi {
	fn untermify(&self, v:&Term) -> Result<bool, UntermifyError> {
		match v.initial_string() {
			"true" | "⊤" | "yes" => {
				Ok(true)
			},
			"false" | "⟂" | "no" => {
				Ok(false)
			},
			_=> Err(mk_untermify_error(v, "expected a bool here".into(), None))
		}
	}
}


#[derive(Copy, Clone)]
pub struct StringBi();
impl<'a> Bitermer<String> for StringBi {}
impl<'a> Termer<String> for StringBi {
	fn termify(&self, v:&String) -> Term { v.as_str().into() }
}
impl<'a> Untermer<String> for StringBi {
	fn untermify(&self, v:&Term) -> Result<String, UntermifyError> {
		match *v {
			Atomv(ref a)=> Ok(a.v.clone()),
			Listv(_)=> Err(mk_untermify_error(v, "sought string, found list".into(), None)),
		}
	}
}

fn termify_seq_into<'a, InnerTran, T, I>(inner:&InnerTran, v:I, output:&mut Vec<Term>)
	where InnerTran: Termer<T>, I:Iterator<Item=&'a T>, T:'a
{
	for vi in v { output.push(inner.termify(vi)); }
}
fn untermify_seq_into<'a, InnerTran, T, I>(inner:&InnerTran, v:I, output:&mut Vec<T>)-> Result<(), UntermifyError>
	where InnerTran: Untermer<T>, I:Iterator<Item=&'a Term>
{
	// let errors = Vec::new();
	for vi in v {
		match inner.untermify(vi) {
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


#[derive(Copy, Clone)]
pub struct SequenceTran<SubTran>(SubTran);
impl<T, SubTran> Bitermer<Vec<T>> for SequenceTran<SubTran> where SubTran:Bitermer<T> {}
impl<T, SubTran> Termer<Vec<T>> for SequenceTran<SubTran> where SubTran:Termer<T> {
	fn termify(&self, v:&Vec<T>) -> Term {
		let mut ret = Vec::new();
		termify_seq_into(&self.0, v.iter(), &mut ret);
		ret.into()
	}
}
impl<T, SubTran> Untermer<Vec<T>> for SequenceTran<SubTran> where SubTran:Untermer<T> {
	fn untermify(&self, v:&Term) -> Result<Vec<T>, UntermifyError> {
		let mut ret = Vec::new();
		try!(untermify_seq_into(&self.0, v.contents(), &mut ret));
		Ok(ret)
	}
}

#[derive(Copy, Clone)]
pub struct TaggedSequenceTran<'a, SubTran>(&'a str, SubTran);
impl<'a, T, SubTran> Bitermer<Vec<T>> for TaggedSequenceTran<'a, SubTran> where SubTran:Bitermer<T> {}
impl<'a, T, SubTran> Termer<Vec<T>> for TaggedSequenceTran<'a, SubTran> where SubTran:Termer<T> {
	fn termify(&self, v:&Vec<T>) -> Term {
		let mut ret = Vec::new();
		ret.push(self.0.into());
		termify_seq_into(&self.1, v.iter(), &mut ret);
		ret.into()
	}
}

fn ensure_tag<'b>(v:&'b Term, tag:&str)-> Result<std::slice::Iter<'b, Term>, UntermifyError> {
	let mut i = v.contents();
	if let Some(name_term) = i.next() {
		match *name_term {
			Atomv(ref at)=>{
				let name = at.v.as_str();
				if name == tag {
					Ok(i)
				}else{
					Err(mk_untermify_error(name_term, format!("expected \"{}\" here, but instead there was \"{}\"", tag, name), None))
				}
			},
			_=> {
				Err(mk_untermify_error(name_term, format!("expected \"{}\" here, but instead there was a list term", tag), None))
			}
		}
	}else{
		Err(mk_untermify_error(v, format!("expected \"{}\" at beginning, but the term was empty", tag), None))
	}
}

impl<'a, T, SubTran> Untermer<Vec<T>> for TaggedSequenceTran<'a, SubTran> where SubTran:Untermer<T> {
	fn untermify(&self, v:&Term) -> Result<Vec<T>, UntermifyError> {
		let mut ret = Vec::new();
		let it = ensure_tag(v, &self.0)?;
		untermify_seq_into(&self.1, it, &mut ret)?;
		Ok(ret)
	}
}


fn untermify_pair<K, V, KeyTran, ValTran>(kt:&KeyTran, vt:&ValTran, v:&Term) -> Result<(K,V), UntermifyError>
	where KeyTran:Untermer<K>, ValTran:Untermer<V>
{
	match *v {
		Listv(ref lc)=> {
			if lc.v.len() == 2 {
				unsafe{
					let k = kt.untermify(lc.v.get_unchecked(0))?; // safe: we just checked the length
					let v = vt.untermify(lc.v.get_unchecked(1))?; //
					Ok((k, v))
				}
			}else{
				Err(mk_untermify_error(v, format!("expected a pair, two elements, but the list here has {}", lc.v.len()), None))
			}
		}
		Atomv(_)=> {
			Err(mk_untermify_error(v, "expected a pair, but the term here is an atom".into(), None))
		}
	}
}

#[derive(Copy, Clone)]
pub struct PairBi<KeyTran, ValTran>(KeyTran, ValTran);
impl<K, V, KeyTran, ValTran> Bitermer<(K, V)> for PairBi<KeyTran, ValTran> where KeyTran:Bitermer<K>, ValTran:Bitermer<V> {}
impl<K, V, KeyTran, ValTran> Termer<(K, V)> for PairBi<KeyTran, ValTran> where KeyTran:Termer<K>, ValTran:Termer<V> {
	fn termify(&self, v:&(K,V)) -> Term {
		let kt = self.0.termify(&v.0);
		let vt = self.1.termify(&v.1);
		list!(kt, vt).into()
	}
}
impl<K, V, KeyTran, ValTran> Untermer<(K, V)> for PairBi<KeyTran, ValTran> where KeyTran:Untermer<K>, ValTran:Untermer<V> {
	fn untermify(&self, v:&Term) -> Result<(K,V), UntermifyError> {
		untermify_pair(&self.0, &self.1, v)
	}
}

fn termify_map<'a, K, V, KeyTermer, ValTermer, I>(ktr:&KeyTermer, vtr:&ValTermer, i:I, o:&mut Vec<Term>)
	where
		KeyTermer: Termer<K>,
		ValTermer: Termer<V>,
		I: Iterator<Item=(&'a K, &'a V)>,
		K: 'a,
		V: 'a,
{
	for (kr, vr) in i {
		o.push(list!(ktr.termify(kr), vtr.termify(vr)).into())
	}
}

fn untermify_map<'a, K, V, KeyTran, ValTran, I>(ktr:&KeyTran, vtr:&ValTran, i:I, o:&mut Vec<(K, V)>)-> Result<(), UntermifyError>
	where
		KeyTran: Untermer<K>,
		ValTran: Untermer<V>,
		I: Iterator<Item=&'a Term>,
{
	for v in i {
		o.push(untermify_pair(ktr, vtr, v)?);
	}
	Ok(())
}

#[derive(Clone)]
pub struct HashMapBi<KeyTran, ValTran>(KeyTran, ValTran);
impl<K, V, KeyTran, ValTran> Bitermer<HashMap<K, V>> for HashMapBi<KeyTran, ValTran>
	where
		KeyTran:Bitermer<K>, ValTran:Bitermer<V>,
		K: Eq + Hash,
		V: Eq + Hash,
{}
impl<K, V, KeyTran, ValTran> Termer<HashMap<K, V>> for HashMapBi<KeyTran, ValTran>
	where
		KeyTran:Termer<K>, ValTran:Termer<V>,
		K: Eq + Hash,
		V: Eq + Hash,
{
	fn termify(&self, v:&HashMap<K, V>)-> Term {
		let mut ret = Vec::new();
		termify_map(&self.0, &self.1, v.iter(), &mut ret);
		ret.into()
	}
}
impl<K, V, KeyTran, ValTran> Untermer<HashMap<K, V>> for HashMapBi<KeyTran, ValTran>
	where
		KeyTran:Untermer<K>, ValTran:Untermer<V>,
		K: Eq + Hash,
		V: Eq + Hash,
{
	fn untermify(&self, v:&Term) -> Result<HashMap<K,V>, UntermifyError> {
		let mut ret = Vec::new();
		untermify_map(&self.0, &self.1, v.contents(), &mut ret)?;
		Ok(HashMap::from_iter(ret.into_iter()))
	}
}

#[derive(Clone)]
pub struct TaggedHashMapBi<'a, KeyTran, ValTran>(&'a str, KeyTran, ValTran);
impl<'a, K, V, KeyTran, ValTran> Bitermer<HashMap<K, V>> for TaggedHashMapBi<'a, KeyTran, ValTran>
	where
		KeyTran:Bitermer<K>, ValTran:Bitermer<V>,
		K: Eq + Hash,
		V: Eq + Hash,
{}
impl<'a, K, V, KeyTran, ValTran> Termer<HashMap<K, V>> for TaggedHashMapBi<'a, KeyTran, ValTran>
	where
		KeyTran:Termer<K>, ValTran:Termer<V>,
		K: Eq + Hash,
		V: Eq + Hash,
{
	fn termify(&self, v:&HashMap<K, V>)-> Term {
		let mut ret = Vec::new();
		ret.push(self.0.into());
		termify_map(&self.1, &self.2, v.iter(), &mut ret);
		ret.into()
	}
}
impl<'a, K, V, KeyTran, ValTran> Untermer<HashMap<K, V>> for TaggedHashMapBi<'a, KeyTran, ValTran>
	where
		KeyTran:Untermer<K>, ValTran:Untermer<V>,
		K: Eq + Hash,
		V: Eq + Hash,
{
	fn untermify(&self, v:&Term) -> Result<HashMap<K,V>, UntermifyError> {
		let mut ret = Vec::new();
		let it = ensure_tag(v, self.0)?;
		untermify_map(&self.1, &self.2, it, &mut ret)?;
		Ok(HashMap::from_iter(ret.into_iter()))
	}
}


#[cfg(test)]
mod tests {
	use super::*;
	
	#[test]
	fn idempotent_int() {
		assert!(90isize == IsizeBi().untermify(&IsizeBi().termify(&90isize)).unwrap());
	}
	
	#[test]
	fn tricky_list_parse() {
		let listo = list!(list!("tricky", "list"), list!("parse"));
		let tranner:SequenceTran<SequenceTran<StringBi>> = SequenceTran(SequenceTran(StringBi()));
		let lv:Vec<Vec<String>> = tranner.untermify(&listo).unwrap();
		assert!(lv.len() == 2);
		assert!(lv[0].len() == 2);
		assert!(lv[0][0].len() == 6);
		assert!(tranner.termify(&lv).to_string() == "((tricky list) (parse))");
	}
	
	#[test]
	fn do_hash_map() {
		let t = parse("a:b c:d d:e e:f").unwrap();
		assert!(TaggedHashMapBi("ob", StringBi(), StringBi()).untermify(&t).is_err());
		let tt = parse("ob a:b c:d d:e e:f").unwrap();
		let hm = TaggedHashMapBi("ob", StringBi(), StringBi()).untermify(&tt).unwrap();
		assert!(hm.get("a").unwrap() == "b");
		assert!(hm.get("d").unwrap() == "e");
	}
}