

//people who read this code often remark about how horrible it is. The truth is, it's the format that's horrible. By horrible, of course, you mean "complicated". I think the word you're really looking for might be "flexible". There is probably no way to write a parser for a format as sensitive as termpose that isn't "horrible". (I would love to find out about it, if there is)

use super::*;
use std::{
	mem::replace,
	ptr::{ read }
};


#[inline(always)]
fn get_back_mut<T>(v:&mut Vec<T>)-> &mut T {
	if let Some(v) = v.last_mut() { v }
	else{ panic!("this vec should never be empty"); }
}

#[derive(Debug)]
pub struct PositionedError{
	pub line:isize,
	pub column:isize,
	pub msg:String,
}

impl Error for PositionedError {
	fn description(&self) -> &str { self.msg.as_str() }
	fn cause(&self) -> Option<&dyn Error> { None }
}
impl Display for PositionedError {
	fn fmt(&self, f: &mut Formatter) -> Result<(), std::fmt::Error> {
		Debug::fmt(self, f)
	}
}




#[inline(always)]
fn assume_branch_mut(v:&mut Wood)-> &mut Branch {
	match *v {
		Branchv(ref mut ls)=> ls,
		Leafv(_)=> panic!("this Wood is supposed to be a branch"),
	}
}
#[inline(always)]
fn assume_leaf_mut(v:&mut Wood)-> &mut Leaf {
	match *v {
		Branchv(_)=> panic!("this Wood is supposed to be an leaf"),
		Leafv(ref mut ar)=> ar,
	}
}
#[inline(always)]
fn assume_branch(v:Wood)-> Branch {
	match v {
		Branchv(ls)=> ls,
		Leafv(_)=> panic!("this Wood is supposed to be a branch"),
	}
}

fn yank_first<T>(v:Vec<T>)-> T { //you must ensure that the v contains at least one item
	match v.into_iter().next() {
		Some(a)=> a,
		None=> panic!("tried to yank the first thing from a vec that contained no things"),
	}
}

fn accrete_branch(v:&mut Wood)-> &mut Vec<Wood> {
	unsafe{
		replace_self(v, |vv|{
			let (line, column) = vv.line_and_col();
			Branchv(Branch{line, column, v:vec!(vv)})
		}); //safe: branch creation doesn't panic
	}
	&mut assume_branch_mut(v).v
}

#[inline(always)]
unsafe fn replace_self<T, F:FnOnce(T)-> T> (t:&mut T, f:F){
	let fr = f(read(t));
	forget(replace(t, fr));
}

unsafe fn str_from_bounds<'a>(o:*const u8, f:*const u8)-> &'a str {
	std::str::from_utf8_unchecked(std::slice::from_raw_parts(o, f as usize - o as usize))
}

fn is_whitespace(c:char)-> bool { c == ' ' || c == '\t' }

fn do_indent(indent:&str, indent_depth:usize, out:&mut String){
	for _ in 0..indent_depth { out.push_str(indent); }
}

// pub fn parse_nakedbranch<'a>(v:&'a str)-> Result<Wood, PositionedError> {
// 	NakedbranchParserState::<'a>::begin(v).parse()
// }









mod termpose_parser;
pub use self::termpose_parser::*;

mod woodslist_parser;
pub use self::woodslist_parser::*;



#[cfg(test)]
mod tests {
	extern crate test;
	use super::*;
	use std::fs::File;
	use std::io::Read;
	use std::path::PathBuf;
	
	fn read_file_from_root(file_name:&str)-> String {
		let mut d = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
		d.push(file_name);
		let mut ret = String::new();
		let mut file = File::open(d).unwrap();
		file.read_to_string(&mut ret).unwrap();
		ret
	}
	
	// #[test]
	// fn it_work() {
	// 	let ti = branch!(branch!("hue", branch!("when", "you", "now"), "then", "else"));
	// 	let to:Wood<&'static str> = parse_sexp("hue (when you now) then else").unwrap();
	// 	println!("the parsed is {}", to.to_string());
	// 	assert!(ti == to);
	// }
	
	#[test]
	fn test_tests_term_file() {
		let teststr = read_file_from_root("tests.term");
		let testt = parse_multiline_termpose(teststr.as_str()).unwrap();
		let testb = testt.find("tests").unwrap();
		
		for tc in testb.tail() {
			let test_name = tc.initial_str();
			let mut ttests = tc.tail();
			if let Some(first_case) = ttests.next() {
				for remaining_case in ttests {
					if first_case != remaining_case {
						panic!(format!("in \"{}\" case, {} did not equal {}", test_name, first_case.to_string(), remaining_case.to_string()));
					}
				}
			}
		}
		
		let failts = testt.find("failing").unwrap();
		for tc in failts.tail() {
			let test_name = tc.initial_str();
			for t in tc.tail() {
				if let Ok(v) = parse_multiline_termpose(t.initial_str()) {
					panic!("in \"{}\" case, this string should not have parsed successfully: {}\n. It parsed to {}", test_name, t.initial_str(), v.to_string());
				}
			}
		}
	}
	
	
	#[test]
	fn test_tests_sexp_file() {
		let teststr = read_file_from_root("sexp tests.sexp");
		let testt = parse_multiline_woodslist(teststr.as_str()).unwrap();
		println!("{}", testt.to_string());
		let testb = testt.find("tests").unwrap();
		
		for tc in testb.tail() {
			let test_name = tc.initial_str();
			let mut ttests = tc.tail();
			if let Some(first_case) = ttests.next() {
				for remaining_case in ttests {
					if first_case != remaining_case {
						panic!(format!("in \"{}\" case, {} did not equal {}", test_name, first_case.to_string(), remaining_case.to_string()));
					}
				}
			}
		}
		
		if let Some(failts) = testt.find("failing") {
			for tc in failts.tail() {
				let test_name = tc.initial_str();
				for t in tc.tail() {
					if let Ok(v) = parse_multiline_termpose(t.initial_str()) {
						panic!("in \"{}\" case, this string should not have parsed successfully: {}\n. It parsed to {}", test_name, t.initial_str(), v.to_string());
					}
				}
			}
		}
	}
	
	
	#[test]
	fn idempotence() {
		let term = branch!("how", branch!("about", "that"));
		assert!(term == parse_termpose(parse_termpose(term.to_string().as_str()).unwrap().to_string().as_str()).unwrap())
	}
	
	use self::test::Bencher;
	#[bench]
	fn bench_long_long_term(b: &mut Bencher) {
		let data = read_file_from_root("longterm.term");
		b.iter(||{
			parse_termpose(data.as_ref())
		});
	}
	
	#[test]
	fn longterm_termpose_idempotence(){
		let w = parse_termpose(&read_file_from_root("longterm.term")).unwrap();
		let wiw = parse_termpose(&pretty_termpose(&w)).unwrap();
		assert_eq!(&w, &wiw);
	}
	
	#[test]
	fn longterm_woodslist_idempotence(){
		let w = parse_woodslist(&read_file_from_root("longterm.term")).unwrap();
		let wiw = parse_woodslist(&indented_woodslist(&w)).unwrap();
		assert_eq!(&w, &wiw);
	}
	
	fn windowsify(v:&str)-> String {
		let mut out = String::new();
		let mut vc = v.chars().peekable();
		while let Some(c) = vc.next() {
			if c == '\n' {
				out.push('\r');
				out.push('\n');
			}else if c == '\r' {
				if vc.peek() == Some(&'\n') {
					//skip it
					vc.next();
				}else{
					out.push('\r');
					out.push('\n');
				}
			}else{
				out.push(c);
			}
		}
		out
	}
	
	#[test]
	fn windows_style_line_endings_do_parse() {
		let wshortterm = windowsify(read_file_from_root("shortterm.term").as_str());
		
		let expected_data = branch!(
			branch!("let", "a", "2"),
			branch!(
				branch!("if", "a"),
				branch!("then", branch!("+", "a", "2")),
				branch!("else", "0")
			)
		);
		
		let parsed = parse_multiline_termpose(&wshortterm).unwrap();
		if expected_data != parsed {
			panic!("{} did not equal expected_data", parsed.to_string());
		}
	}
	
	#[test]
	fn windows_style_line_endings_as_expected_in_multiline_string() {
		let wshortterm = windowsify("
ms \"
	a
	
	b
");
		
		let expected_data = "a\n\nb";
		
		let parsed = parse_multiline_termpose(&wshortterm).unwrap();
		let pms = parsed.find("ms").unwrap().tail().next().unwrap().initial_str();
		assert_eq!(expected_data, pms);
	}
	
		#[test]
		fn windows_style_line_endings_as_expected_in_multiline_woodslist_string() {
			let wshortterm = windowsify("
	(ms \"
a

b\")
	");
			
			let expected_data = "a\n\nb";
			
			let parsed = parse_multiline_woodslist(&wshortterm).unwrap();
			let pms = parsed.find("ms").unwrap().tail().next().unwrap().initial_str();
			assert_eq!(expected_data, pms);
		}
	
	#[test]
	fn woodslist_parser_skips_first_newline() {
		let w = parse_woodslist("aaa \"\naa sdi \n  idj\" a").unwrap();
		assert_eq!(&branch!("aaa", "aa sdi \n  idj", "a"), &w, "uh");
	}
	
	#[test]
	fn woodslist_parser_doesnt_skip_first_non_newline() {
		let w = parse_woodslist("aaa \"aa sdi \n  idj\" a").unwrap();
		assert_eq!(&branch!("aaa", "aa sdi \n  idj", "a"), &w, "uh");
	}
}
