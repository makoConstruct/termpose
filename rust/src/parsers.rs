

use super::*;


struct SexpParserState<'a>{
	root:Vec<Term>,
	stack:Vec<*mut Vec<Term>>,
	iter:std::str::Chars<'a>,
	started_eating: &'a str,
	eating_str_mode:bool,
	line:isize,
	column:isize,
}


//This is supposed to invoke unreachable (UB) in release and panic in debug. The point is, it's a *strong* assertion to the optimizer that one of the two branches isn't supposed to be reachable, and that whatever conditionals lead to it do not need to be evaluated. See below for benchmarks demonstrating that this doesn't much occur.
#[inline(always)]
unsafe fn seriously_unreach(s:&str)-> ! {
	if cfg!(debug_assertions) { panic!("{}", s) }
	else{ unreachable() }
}

#[inline(always)]
unsafe fn serious_get_back_mut<T>(v:&mut Vec<T>)-> &mut T {
	if let Some(v) = v.last_mut() { v }
	else{ seriously_unreach("this vec should never be empty"); }
}

#[inline(always)]
unsafe fn seriously_unwrap<T>(v:Option<T>)-> T {
	if let Some(va) = v { va }
	else{ seriously_unreach("this option should have been some"); }
}

#[inline(always)]
unsafe fn seriously_pop<T>(v: &mut Vec<T>)-> T {
	seriously_unwrap(v.pop())
}

#[derive(Debug)]
pub struct PositionedError{
	pub line:isize,
	pub column:isize,
	pub msg:String,
}

impl Error for PositionedError {
	fn description(&self) -> &str { self.msg.as_str() }
	fn cause(&self) -> Option<&Error> { None }
}
impl Display for PositionedError {
	fn fmt(&self, f: &mut Formatter) -> Result<(), std::fmt::Error> {
		Debug::fmt(self, f)
	}
}

// fn find_ptr<'a>(vl:&Vec<Terms<'a>>, p:*const Vec<Terms<'a>>)-> bool {
// 	if vl as *const _ == p {
// 		true
// 	}else{
// 		vl.iter().any(|v:&Terms<'a>|{
// 			if let List(ref lr) = *v {
// 				find_ptr(lr, p)
// 			}else{
// 				false
// 			}
// 		})
// 	}
// }

impl<'a> SexpParserState<'a> {
	fn begin(s:&'a str)-> SexpParserState<'a> {
		SexpParserState{
			root: Vec::new(),
			stack: Vec::new(),
			iter: s.chars(),
			started_eating: "",
			eating_str_mode: false,
			line: 0,
			column: 0,
		}
	}
	fn last_list<'b>(&'b mut self)-> &'b mut Vec<Term> where 'a:'b {
		let ret = if self.stack.len() > 0 {
			unsafe{ *serious_get_back_mut(&mut self.stack) }
		}else{
			&mut self.root
		};
		unsafe{ &mut *ret }
	}
	fn pinch_off_string_if_eating<'b>(&'b mut self, ending:&'a str) where 'a:'b {
		if self.eating_str_mode {
			let len = ending.as_ptr() as usize - self.started_eating.as_ptr() as usize;
			let s = unsafe{ self.started_eating.slice_unchecked(0, len) };
			let line = self.line;
			let column = self.column;
			self.last_list().push(Atomv(Atom{line, column, v:s.to_string()}));
			self.started_eating = "";
			self.eating_str_mode = false;
		}
	}
	unsafe fn close_scope<'b>(&'b mut self) where 'a:'b {
		seriously_pop(&mut self.stack);
	}
	fn new_scope<'b>(&'b mut self) where 'a:'b {
		let line = self.line;
		let column = self.column;
		self.last_list().push(Listv(List{
			line:line,
			column:column,
			v:Vec::new()
		}));
		//potential unnecessary indexing in last_mut, `push` already knows the index. Could be mitigated with a place_back
		unsafe{
			let flp = 
				if let Listv(ref mut final_list_ref) = *seriously_unwrap(self.last_list().last_mut()) {
					(&mut final_list_ref.v) as *mut _
				}else{
					seriously_unreach("this should never happen")
				};
			self.stack.push(flp);
		}
	}
	fn parse_sexp(mut self)-> Result<Term, PositionedError> {
		loop {
			let str_starting_here = self.iter.as_str();
			if let Some(c) = self.iter.next() {
				match c {
					' ' | '\t' => {
						self.pinch_off_string_if_eating(str_starting_here);
						self.column += 1;
					}
					'\n' => {
						self.pinch_off_string_if_eating(str_starting_here);
						self.column = 0;
						self.line += 1;
					}
					'(' => {
						self.pinch_off_string_if_eating(str_starting_here);
						self.new_scope();
						self.column += 1;
					}
					')' => {
						if self.stack.len() == 0 {
							return Err(PositionedError{ line:self.line, column:self.column, msg:"excess paren".into() });
						}else{
							self.pinch_off_string_if_eating(str_starting_here);
							debug_assert!(self.stack.len() > 0);
							unsafe{ self.close_scope(); }
						}
						self.column += 1;
					}
					_ => {
						if !self.eating_str_mode {
							self.eating_str_mode = true;
							self.started_eating = str_starting_here;
						}
						self.line += 1;
					}
				}
			}else{
				if self.stack.len() > 1 {
					return Err(PositionedError{ line:self.line, column:self.column, msg:"paren left open before the end".into() });
				}
				self.pinch_off_string_if_eating(str_starting_here);
				break;
			}
		}
		
		Ok(Listv(List{line:-1, column:-1, v:self.root.into()}))
	}
}

pub fn parse_sexp<'a>(v:&'a str)-> Result<Term, PositionedError> {
	SexpParserState::<'a>::begin(v).parse_sexp()
}








//this has been written in the style of a C++ programmer who doesn't want to waste a single cycle. However: Every use of unsafe has been justified with a comment

unsafe fn seriously_get_first_mut<T>(v:&mut Vec<T>)-> &mut T {
	if v.len() > 0 {
		v.get_unchecked_mut(0)
	}else{
		seriously_unreach("vec was supposed to have something in it")
	}
}
#[inline(always)]
unsafe fn seriously_list_mut(v:&mut Term)-> &mut List {
	match *v {
		Listv(ref mut ls)=> ls,
		Atomv(_)=> seriously_unreach("this Term is supposed to be a list"),
	}
}
#[inline(always)]
unsafe fn seriously_atom_mut(v:&mut Term)-> &mut Atom {
	match *v {
		Listv(_)=> seriously_unreach("this Term is supposed to be an atom"),
		Atomv(ref mut ar)=> ar,
	}
}
// #[inline(always)]
// unsafe fn seriously_list_ref(v:&Term)-> &List {
// 	match *v {
// 		Listv(ref ls)=> ls,
// 		Atomv(_)=> seriously_unreach("this Term is supposed to be a list"),
// 	}
// }
// #[inline(always)]
// unsafe fn seriously_atom_ref(v:&Term)-> &Atom {
// 	match *v {
// 		Listv(_)=> seriously_unreach("this Term is supposed to be an atom"),
// 		Atomv(ref ar)=> ar,
// 	}
// }
#[inline(always)]
unsafe fn seriously_list(v:Term)-> List {
	match v {
		Listv(ls)=> ls,
		Atomv(_)=> seriously_unreach("this Term is supposed to be a list"),
	}
}
// #[inline(always)]
// unsafe fn seriously_atom(v:Term)-> Atom {
// 	match v {
// 		Listv(_)=> seriously_unreach("this Term is supposed to be an atom"),
// 		Atomv(ar)=> ar,
// 	}
// }

unsafe fn seriously_yank_first<T>(v:Vec<T>)-> T { //you must ensure that the v contains at least one item
	match v.into_iter().next() {
		Some(a)=> a,
		None=> seriously_unreach("tried to yank the first thing from a vec that contained no things"),
	}
}


unsafe fn replace_self<T, F>(v:&mut T, f:F) where F : FnOnce(T)-> T { //for replacing a thing with a transformation of itself. It's actually fairly safe, you just have to guarantee that this transformation, f, doesn't panic, or else this function will drop an uninitialized, or v's drop will be called twice, or something. I don't even know, so don't don't panic or else!
	let res = f(replace(v, uninitialized()));
	forget(replace(v, res));
}

unsafe fn str_from_bounds<'a>(o:*const u8, f:*const u8)-> &'a str {
	std::str::from_utf8_unchecked(std::slice::from_raw_parts(o, f as usize - o as usize))
}

struct ParserState<'a>{
	root: Term,
	indent_stack: Vec<&'a str>,
	indent_list_stack: Vec<*mut Vec<Term>>, //the lists corresponding to each indent level, into which new lines on that level are inserted
	line_paren_stack: Vec<*mut Term>,
	cur_char_ptr: *const u8,
	//optimization: Consider making these three an untagged union, since only one is used at a time?:
	stretch_reading_start: *const u8, //used when taking an indent
	atom_being_read_into: *mut String,
	colon_receptacle: *mut Vec<Term>,
	multilines_indent: &'a str,
	// previous_line_hanging_term: *mut Term, //this is the term things will be inserted into if there's an indent.
	iter: std::str::Chars<'a>,
	line: isize,
	column: isize,
	mode: fn(&mut ParserState<'a>, Option<char>)-> Result<(), PositionedError>,
	chosen_style: TermposeStyle,
}

#[derive(Clone)]
pub struct TermposeStyle {
	pub open:char,
	pub close:char,
	pub pairing:char,
}

static DEFAULT_STYLE:TermposeStyle = TermposeStyle{ open:'(', close:')', pairing:':' };

impl<'a> ParserState<'a> {
	
	fn style(&self)-> &TermposeStyle { &self.chosen_style }
	
	fn a_fail(&self, message:String)-> Result<(), PositionedError> { Err(PositionedError{
		line: self.line,
		column: self.column,
		msg: message,
	}) }
	
	fn mklist(&self)-> Term { Listv(List{ line:self.line, column:self.column, v:Vec::new() }) }
	
	fn start_line(&mut self, c:char)-> Result<(), PositionedError> {
		unsafe{
			let bin:*mut Vec<Term> = *serious_get_back_mut(&mut self.indent_list_stack); //safe: there is always at least root in the indent_list_stack
			(*bin).push(self.mklist());
			self.line_paren_stack.clear();
			self.line_paren_stack.push(serious_get_back_mut(&mut *bin)); //safe: just made that
			self.start_reading_thing(c)
		}
	}
	
	fn consider_collapsing_outer_list_of_previous_line(&mut self){
		unsafe{
			let line_term: *mut Term = *seriously_get_first_mut(&mut self.line_paren_stack);
			let line_list_length:usize = seriously_list_mut(&mut*line_term).v.len(); //safe: line_term is always a list
			if line_list_length == 1 {
				replace_self(&mut*line_term, |l|{
					let List{ v, .. } = seriously_list(l);
					seriously_yank_first(v)
				}) //safe: this_line always starts out a list, until we make it not one here. We have proven that it contains something. seriously_list can panic but only in debug mode, and it will not because we know it really is a list
			}
		}
	}
	
	fn take_hanging_list_for_insert(&mut self)-> *mut Vec<Term> {
		if self.colon_receptacle != null_mut() {
			replace(&mut self.colon_receptacle, null_mut())
		}else{
			&mut unsafe{seriously_list_mut(&mut**serious_get_back_mut(&mut self.line_paren_stack))}.v //safe; always something in parenstack, and it's always a list
		}
	}
	fn iterate_char_iter(&mut self)-> Option<char> {
		self.cur_char_ptr = self.iter.as_str().as_ptr();
		self.iter.next()
	}
	fn next_char_ptr(&self)-> *const u8 { self.iter.as_str().as_ptr() }
	fn next_col(&mut self){ self.column += 1; }
	fn next_line(&mut self){ self.line += 1; self.column = 0; }
	
	fn open_paren(&mut self) {
		let lti = self.mklist();
		let list_for_insert = self.take_hanging_list_for_insert();
		unsafe{(*list_for_insert).push(lti)};
		self.line_paren_stack.push(unsafe{serious_get_back_mut(&mut*list_for_insert)} as *mut _); //safe: just made that
	}
	fn close_paren(&mut self)-> Result<(), PositionedError> {
		self.colon_receptacle = null_mut();
		if self.line_paren_stack.len() > 1 {
			unsafe{seriously_pop(&mut self.line_paren_stack)}; //safe: we just checked and confirmed there's something there
			Ok(())
		}else{
			self.a_fail("unmatched paren".into())
		}
	}
	fn open_colon(&mut self) {
		//seek the back term and accrete over it. If there isn't one, create an empty list
		unsafe{
			let rl = self.take_hanging_list_for_insert();
			self.colon_receptacle = if (*rl).len() > 0 {
				accrete_list(serious_get_back_mut(&mut*rl)) //safe: just verified something was there
			}else{
				(*rl).push(self.mklist());
				&mut seriously_list_mut(serious_get_back_mut(&mut*rl)).v //safe: just put it there
			}
		}
	}	
	fn begin_atom(&mut self, list_for_insert:*mut Vec<Term>) {
		let to_push = Atomv(Atom{line:self.line, column:self.column, v:String::new()});
		unsafe{(*list_for_insert).push(to_push)};
		self.atom_being_read_into = &mut unsafe{seriously_atom_mut(serious_get_back_mut(&mut *list_for_insert))}.v;
	}
	fn begin_atom_with_char(&mut self, list_for_insert:*mut Vec<Term>, c:char)-> Result<(), PositionedError> {
		let to_push = Atomv(Atom{line:self.line, column:self.column, v:String::new()});
		unsafe{(*list_for_insert).push(to_push)};
		self.atom_being_read_into = &mut unsafe{seriously_atom_mut(serious_get_back_mut(&mut *list_for_insert))}.v;
		if c == '\\' {
			try!(self.read_escaped_char());
		}else{
			unsafe{(*self.atom_being_read_into).push(c)};
		};
		Ok(())
	}
	
	fn start_reading_thing(&mut self, c:char)-> Result<(), PositionedError> {
		match c {
			c if c == self.style().close=> {
				try!(self.close_paren());
				self.mode = Self::seeking_immediately_after_thing;
			},
			c if c == self.style().open=> {
				self.open_paren();
				self.mode = Self::seeking_term;
			},
			'"'=> {
				let l = self.take_hanging_list_for_insert();
				self.begin_atom(l);
				self.mode = Self::eating_quoted_string;
			},
			c if c == self.style().pairing=> {
				self.open_colon();
				self.mode = Self::seeking_term;
			},
			_=> {
				let l = self.take_hanging_list_for_insert();
				try!(self.begin_atom_with_char(l, c));
				self.mode = Self::eating_atom;
			},
		}
		self.next_col();
		Ok(())
	}
	
	fn pop_indent_stack_down(&mut self, this_indent:&'a str)-> Result<(), PositionedError> {
		loop{
			let containing_indent = *unsafe{serious_get_back_mut(&mut self.indent_stack)}; //safe: we can be assured that there is always something at back, because a str can't be smaller than the root indent "" and not be a prefix of it
			if this_indent.len() == containing_indent.len() {
				if this_indent == containing_indent {
					//found it
					return Ok(());
				}else{
					return self.a_fail("inconsistent indentation".into());
				}
			}else if this_indent.len() > containing_indent.len() {
				//oh no, it's too short to be with the last level and too long to be with the next level, it must not be in the allowed set
				return self.a_fail("inconsistent indentation".into());
			}
			self.indent_stack.pop();
			self.indent_list_stack.pop();
		}
	}
	
	fn end_unindented_line(&mut self){
		self.consider_collapsing_outer_list_of_previous_line();
		self.colon_receptacle = null_mut();
	}
	
	fn notice_this_new_indentation<OnSmaller, OnEqual, OnGreater>(&mut self, sc:OnSmaller, ec:OnEqual, gc:OnGreater)-> Result<(), PositionedError> where
		OnSmaller : FnOnce(&mut Self)-> Result<(), PositionedError>,
		OnEqual   : FnOnce(&mut Self)-> Result<(), PositionedError>,
		OnGreater : FnOnce(&mut Self, &'a str)-> Result<(), PositionedError>,
	{
		//there's definitely a thing here, ending indentation
		let this_indent = unsafe{str_from_bounds(self.stretch_reading_start, self.cur_char_ptr)};
		let containing_indent = *unsafe{serious_get_back_mut(&mut self.indent_stack)}; //safe: indent function always has something in it
		if containing_indent.len() == this_indent.len() {
			if containing_indent != this_indent {
				return self.a_fail("inconsistent indentation".into());
			}
			//no indent:
			self.end_unindented_line();
			ec(self)
		}else if this_indent.len() < containing_indent.len() {
			if !containing_indent.starts_with(this_indent) {
				return self.a_fail("inconsistent indentation".into());
			}
			//no indent:
			self.end_unindented_line();
			//pop indent stack until we're on the right level
			try!(self.pop_indent_stack_down(this_indent));
			sc(self)
		}else{ //greater
			if !this_indent.starts_with(containing_indent) {
				return self.a_fail("inconsistent indentation".into());
			}
			gc(self, this_indent)
		}
	}
	
	
	//modes
	
	fn seeking_beginning(&mut self, co:Option<char>)-> Result<(), PositionedError> {
		if let Some(c) = co {
			match c {
				' ' | '\t' => {
					self.next_col();
					Ok(())
				},
				'\n' => {
					self.next_line();
					Ok(())
				},
				_ => {
					self.start_line(c)
				},
			}
		}else{
			Ok(())
		}
	}
	
	fn eating_indentation(&mut self, co:Option<char>)-> Result<(), PositionedError> {
		if let Some(c) = co {
			match c {
				' ' | '\t' => {
					self.next_col();
				},
				'\n' => {
					self.stretch_reading_start = self.next_char_ptr();
					// self.mode = &Self::eating_indentation;
					self.next_line();
				},
				_=> {
					//there's definitely a thing here, ending indentation
					try!(self.notice_this_new_indentation(
						|_  :&mut Self|{ Ok(()) },
						|_  :&mut Self|{ Ok(()) },
						|slf:&mut Self, this_indent:&'a str|{
							let plhl = slf.take_hanging_list_for_insert();
							slf.indent_stack.push(this_indent);
							slf.indent_list_stack.push(plhl as *mut _);
							Ok(())
						},
					));
					try!(self.start_line(c));
				}
			}
		}else{
			//absense of indent confirmed, so
			self.end_unindented_line();
		}
		Ok(())
	}
	fn seeking_term(&mut self, co:Option<char>)-> Result<(), PositionedError> {
		if let Some(c) = co {
			match c {
				' ' | '\t' => {
					self.next_col();
				},
				'\n' => {
					self.stretch_reading_start = self.next_char_ptr();
					self.mode = Self::eating_indentation;
					self.next_col();
				},
				c if c == self.style().pairing=> {
					self.open_colon();
					self.next_col();
				},
				_=> {
					try!(self.start_reading_thing(c));
				}
			}
		}else{
			self.end_unindented_line();
		}
		Ok(())
	}

	fn read_escaped_char(&mut self)-> Result<(), PositionedError> {
		let push = |slf:&mut Self, c:char| unsafe{((*slf.atom_being_read_into)).push(c)}; //safe: atom_being_read_into must have been validated before this mode could have been entered
		let nco = self.iterate_char_iter();
		let match_fail_message = "escape slash must be followed by a valid escape character code";
		if let Some(nc) = nco {
			match nc {
				'n'=> { push(self, '\n'); }
				'r'=> { push(self, '\r'); }
				't'=> { push(self, '\t'); }
				'h'=> { push(self, '☃'); }
				'"'=> { push(self, '"'); }
				'\\'=> { push(self, '\\'); }
				_=> { return self.a_fail(match_fail_message.into()); }
			}
		}else{
			return self.a_fail(match_fail_message.into());
		}
		self.column += 2;
		Ok(())
	}

	fn eating_quoted_string(&mut self, co:Option<char>)-> Result<(), PositionedError> {
		let push_char = |slf:&mut Self, c:char| unsafe{((*slf.atom_being_read_into)).push(c)}; //safe: atom_being_read_into must have been validated before this mode could have been entered
		if let Some(c) = co {
			match c {
				'\n'=> {
					self.stretch_reading_start = self.next_char_ptr();
					if unsafe{(*self.atom_being_read_into).len()} == 0 {
						//begin multiline string
						self.mode = Self::eating_initial_multline_string_indentation;
					}else {
						self.mode = Self::eating_indentation;
					}
					self.next_line();
				},
				'\\'=> {
					try!(self.read_escaped_char());
				},
				'"'=> {
					self.next_col();
					self.mode = Self::seeking_immediately_after_thing;
				},
				_=> {
					push_char(self, c);
					self.next_col();
				}
			}
		}else{
			self.end_unindented_line();
		}
		Ok(())
	}

	fn eating_atom(&mut self, co:Option<char>)-> Result<(), PositionedError> {
		let push_char = |slf:&mut Self, c:char| unsafe{((*slf.atom_being_read_into)).push(c)}; //safe: atom_being_read_into must have been validated before this mode could have been entered
		if let Some(c) = co {
			match c {
				' ' | '\t' => {
					self.mode = Self::seeking_term;
					self.next_col();
				},
				'\n'=> {
					self.stretch_reading_start = self.next_char_ptr();
					self.mode = Self::eating_indentation;
					self.next_line();
				},
				'\\'=> {
					try!(self.read_escaped_char());
				},
				c if c == self.style().pairing=> {
					self.open_colon();
					self.mode = Self::seeking_term;
					self.next_col();
				},
				c if c == self.style().close=> {
					try!(self.close_paren());
					self.mode = Self::seeking_immediately_after_thing;
					self.next_col();
				},
				c if c == self.style().open=> {
					self.notice_paren_immediately_after_thing();
				},
				_=> {
					push_char(self, c);
					self.next_col();
				}
			}
		}else{
			self.end_unindented_line();
		}
		Ok(())
	}
	
	fn back_term(&mut self)-> &mut Term {
		unsafe{serious_get_back_mut(&mut seriously_list_mut(&mut **serious_get_back_mut(&mut self.line_paren_stack)).v)}
	}
	
	fn notice_paren_immediately_after_thing(&mut self){
		let bl = self.back_term() as *mut _;
		accrete_list(unsafe{&mut *bl});
		self.line_paren_stack.push(bl);
		self.mode = Self::seeking_term;
		self.next_col();
	}

	fn seeking_immediately_after_thing(&mut self, co:Option<char>)-> Result<(), PositionedError> { //generally called after ')' or a closing '"', because '"'s and self.style().opens have slightly different meaning in that context
		if let Some(c) = co {
			match c {
				'"'=> {
					unsafe{
						let lt: *mut Vec<Term> = *serious_get_back_mut(&mut self.line_paren_stack) as *mut _;
						if (*lt).len() == 0 {
							seriously_unreach("begin_seeking_immediately_after_thing should not be called after entering an empty paren");
						}
						let bt = serious_get_back_mut(&mut*lt);
						let nl = accrete_list(&mut*bt);
						nl.push(Atomv(Atom{line:self.line, column:self.column, v:String::new()}));
						self.atom_being_read_into = &mut seriously_atom_mut(serious_get_back_mut(nl)).v; //safe: just made that
					}
					self.mode = Self::eating_quoted_string;
					self.next_col();
				},
				c if c == self.style().close=> {
					try!(self.close_paren());
					self.next_col();
				},
				c if c == self.style().open=> {
					self.notice_paren_immediately_after_thing();
				},
				c if c == self.style().pairing=> {
					self.open_colon();
					self.mode = Self::seeking_term;
					self.next_col();
				},
				' ' | '\t' => {
					self.mode = Self::seeking_term;
					self.next_col();
				},
				'\n'=> {
					self.stretch_reading_start = self.next_char_ptr();
					self.mode = Self::eating_indentation;
					self.next_line();
				},
				_=> {
					let il = self.take_hanging_list_for_insert();
					try!(self.begin_atom_with_char(il, c));
					self.mode = Self::eating_atom;
					self.next_col();
				},
			}
		}else{
			self.end_unindented_line();
		}
		Ok(())
	}

	fn eating_initial_multline_string_indentation(&mut self, co:Option<char>)-> Result<(), PositionedError> {
		if let Some(c) = co {
			match c {
				' ' | '\t' => {
					self.next_col();
				},
				'\n'=> {
					self.stretch_reading_start = self.next_char_ptr();
					self.next_line();
				},
				_=> {
					//ending indentation
					try!(self.notice_this_new_indentation(
						|slf:&mut Self|{
							//not multiline after all
							slf.start_line(c)
						},
						|slf:&mut Self|{
							//not multiline after all
							slf.start_line(c)
						},
						|slf:&mut Self, this_indent:&'a str|{
							slf.multilines_indent = this_indent;
							unsafe{(*slf.atom_being_read_into).push(c)};
							slf.mode = Self::eating_multiline_content;
							slf.next_col();
							Ok(())
						},
					));
				},
			}
		}else{
			self.end_unindented_line();
		}
		Ok(())
	}

	fn eating_multiline_content(&mut self, co:Option<char>)-> Result<(), PositionedError> {
		if let Some(c) = co {
			match c {
				'\n'=> {
					// unsafe{(*self.atom_being_read_into).push(c)}; //actually, we'll only take the character once it's been confirmed that the indent goes all the way up
					self.stretch_reading_start = self.next_char_ptr();
					self.mode = Self::eating_multiline_later_indent;
					self.next_line();
				},
				_=> {
					unsafe{(*self.atom_being_read_into).push(c)};
					self.next_col();
				}
			}
		}else{
			self.end_unindented_line();
		}
		Ok(())
	}

	fn eating_multiline_later_indent(&mut self, co:Option<char>)-> Result<(), PositionedError> {
		if let Some(c) = co {
			match c {
				' ' | '\t' => {
					let curstr = unsafe{str_from_bounds(self.stretch_reading_start, self.next_char_ptr())}; //safe: these u8 pointers are fine
					if curstr.len() == self.multilines_indent.len() {
						if curstr != self.multilines_indent {
							return self.a_fail("inconsistent indentation".into());
						}
						unsafe{(*self.atom_being_read_into).push('\n')}; //only now do we finalize the newline given
						self.mode = Self::eating_multiline_content;
					}else{
						if !self.multilines_indent.starts_with(curstr) {
							return self.a_fail("inconsistent indentation".into());
						}
					}
					self.next_col();
				},
				'\n'=> {
					//does not eat the newline
					self.stretch_reading_start = self.next_char_ptr();
					self.next_line();
				},
				_=> {
					//ending indentation
					//since the indentation hasn't ended already, this must be a shorter line than the multiline scope, so we'll pop
					let this_indent = unsafe{str_from_bounds(self.stretch_reading_start, self.cur_char_ptr)};
					try!(self.pop_indent_stack_down(this_indent));
					self.consider_collapsing_outer_list_of_previous_line();
					try!(self.start_line(c));
				},
			}
		}else{
			self.end_unindented_line();
		}
		Ok(())
	}

} //ParserState

fn accrete_list(v:&mut Term)-> &mut Vec<Term> {
	unsafe{
		replace_self(v, |vv|{
			let (line, col) = vv.line_and_col();
			Listv(List{line:line, column:col, v:vec!(vv)})
		}); //safe: list creation doesn't panic
		&mut seriously_list_mut(v).v
	}
}


pub fn parse_multiline_style<'a>(s:&'a str, style:TermposeStyle)-> Result<Term, PositionedError> { //parses as if it's a file, and each term at root is a separate term. This is not what you want if you expect only a single line, and you want that line to be the root term, but its behaviour is more consistent if you are parsing files
	let mut state = ParserState::<'a>{
		root: list!(), //a yet empty line
		indent_stack: vec!(""),
		stretch_reading_start: s.as_ptr(),
		cur_char_ptr: s.as_ptr(),
		colon_receptacle: null_mut(),
		atom_being_read_into: null_mut(),
		iter: s.chars(),
		line: 0,
		column: 0,
		multilines_indent: "",
		line_paren_stack: vec!(),
		indent_list_stack: vec!(),
		mode: ParserState::<'a>::seeking_beginning,
		chosen_style: style,
	};
	state.indent_list_stack = vec!(unsafe{&mut seriously_list_mut(&mut state.root).v}); //safe: just amde thath
	
	
	loop {
		let co = state.iterate_char_iter();
		try!((state.mode)(&mut state, co));
		if co == None { break; }
	}
	
	Ok(state.root)
}

pub fn parse_multiline<'a>(s:&'a str)-> Result<Term, PositionedError> {
	parse_multiline_style(s, DEFAULT_STYLE.clone())
}

pub fn parse<'a>(s:&'a str)-> Result<Term, PositionedError> {
	parse_multiline(s).map(|t|{
		let l = unsafe{seriously_list(t)}; //safe: parse_multiline only returns lists
		if l.v.len() == 1 {
			unsafe{seriously_yank_first(l.v)} //safe: just confirmed it's there
		}else{
			//then you were wrong, it wasn't a single root term :< what do you expect
			Listv(l)
		}
	})
}












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
	// 	let ti = list!(list!("hue", list!("when", "you", "now"), "then", "else"));
	// 	let to:Term<&'static str> = parse_sexp("hue (when you now) then else").unwrap();
	// 	println!("the parsed is {}", to.to_string());
	// 	assert!(ti == to);
	// }
	
	#[test]
	fn test_tests_term_file() {
		let teststr = read_file_from_root("tests.term");
		let testt = parse_multiline(teststr.as_str()).unwrap();
		let testb = testt.find("tests").unwrap();
		for tc in testb.tail() {
			let test_name = tc.initial_string();
			let mut ttests = tc.tail();
			if let Some(first_case) = ttests.next() {
				for remaining_case in ttests {
					if first_case != remaining_case {
						panic!(format!("in \"{}\" case, {} did not equal {}", test_name, first_case.to_string(), remaining_case.to_string()));
					}
				}
			}
		}
	}
	
	
	#[test]
	fn idempotence() {
		let term = list!("how", list!("about", "that"));
		assert!(term == parse(parse(term.to_string().as_str()).unwrap().to_string().as_str()).unwrap())
	}
	
	use self::test::Bencher;
	#[bench]
	fn bench_long_long_term(b: &mut Bencher) {
		let data = read_file_from_root("longterm.term");
		b.iter(||{
			parse(data.as_ref())
		});
	}
	
	//results: dynamic style makes no difference to performance. The files have different structures under different styles, so a masking effect could be going on (dynamic may have an advantage), but meh
	// #[bench]
	// fn bench_long_long_term_with_dynamic_style(b: &mut Bencher) {
	// 	let data = read_file_from_root("longterm.term");
	// 	b.iter(||{
	// 		parse_multiline_style(
	// 			data.as_str(),
	// 			test::black_box(DEFAULT_STYLE.clone())
	// 		)
	// 	})
	// }
	
	
	
	//the following tests are for testing the performance effects of the seriously_assume methods. Results: They are small. seriously_unwrap is significant, but methods about assuming vecs contain something don't seem to do much at all. Around 3% speedup.
	
	#[inline(always)]
	fn array_some_predicate(i:usize)-> bool { (((i & (i >> 1)) ^ (i/3)) & 1) != 0 } //an odd condition so that the optimizer wont just infer all of this
	
	// #[inline(always)]
	// fn option_array_test<F>(b: &mut Bencher, f:F) where F : Fn(usize, &Option<usize>)-> usize {
	// 	let mut subject = Vec::new();
	// 	for i in 0..30000 {
	// 		if array_some_predicate(i) {
	// 			subject.push(Some(i));
	// 		}else{
	// 			subject.push(None);
	// 		}
	// 	}
		
	// 	b.iter(||{
	// 		let mut total = 0;
	// 		for (i, o) in subject.iter().enumerate() {
	// 			total += f(i, o)
	// 		}
	// 		total
	// 	})
	// }
	
	// #[bench]
	// fn test_array_seriously_unwrap(b: &mut Bencher){
	// 	option_array_test(b, |i:usize, o:&Option<usize>|{
	// 		if array_some_predicate(i) {
	// 			unsafe{ seriously_unwrap(*o) }
	// 		}else{
	// 			0
	// 		}
	// 	})
	// }
	
	// #[bench]
	// fn test_array_control(b: &mut Bencher){
	// 	option_array_test(b, |i:usize, o:&Option<usize>|{
	// 		if array_some_predicate(i) {
	// 			match *o {
	// 				Some(i)=> i,
	// 				None=> panic!("bahh!"),
	// 			}
	// 		}else{
	// 			0
	// 		}
	// 	})
	// }
	
	
	
	
	// #[inline(always)]
	// fn vec_array_test<F>(b: &mut Bencher, f:F) where F : Fn(usize, &Vec<usize>)-> usize {
	// 	let mut subject = Vec::new();
	// 	for i in 0..30000 {
	// 		if array_some_predicate(i) {
	// 			subject.push(vec!(i));
	// 			// println!("on");
	// 		}else{
	// 			subject.push(vec!());
	// 			// println!("off");
	// 		}
	// 	}
		
	// 	b.iter(||{
	// 		let mut total = 0;
	// 		for (i, o) in subject.iter().enumerate() {
	// 			total += f(i, o)
	// 		}
	// 		total
	// 	})
	// }
	
	// #[inline(always)]
	// unsafe fn serious_get_back<T>(v:&Vec<T>)-> &T {
	// 	if v.len() != 0 { v.get_unchecked(v.len() - 1) }
	// 	else{ seriously_unreach("this vec should never be empty"); }
	// }
	
	// #[bench]
	// fn test_vec_array_seriously(b: &mut Bencher){
	// 	vec_array_test(b, |i:usize, o:&Vec<usize>|{
	// 		if array_some_predicate(i) {
	// 			unsafe{ *o.get_unchecked(0) }
	// 		}else{
	// 			0
	// 		}
	// 	})
	// }
	
	// #[bench]
	// fn test_vec_array_control(b: &mut Bencher){
	// 	vec_array_test(b, |i:usize, o:&Vec<usize>|{
	// 		if array_some_predicate(i) {
	// 			match o.last() {
	// 				Some(i)=> *i,
	// 				None=> panic!("bahh!"),
	// 			}
	// 		}else{
	// 			0
	// 		}
	// 	})
	// }
	
}
