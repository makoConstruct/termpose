

//people who read this code often remark about how horrible it is. The truth is, it's the format that's horrible. By horrible, of course, you mean "complicated". I think the word you're really looking for might be "flexible". There is probably no way to write a parser for a format as sensitive as termpose that isn't "horrible". (I would love to find out about it, if there is)

use super::*;

struct SexpParserState<'a>{
	root:Vec<Wood>,
	stack:Vec<*mut Vec<Wood>>,
	iter:std::str::Chars<'a>,
	started_eating: &'a str,
	eating_str_mode:bool,
	line:isize,
	column:isize,
}


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
	fn cause(&self) -> Option<&Error> { None }
}
impl Display for PositionedError {
	fn fmt(&self, f: &mut Formatter) -> Result<(), std::fmt::Error> {
		Debug::fmt(self, f)
	}
}

// fn find_ptr<'a>(vl:&Vec<Woods<'a>>, p:*const Vec<Woods<'a>>)-> bool {
// 	if vl as *const _ == p {
// 		true
// 	}else{
// 		vl.iter().any(|v:&Woods<'a>|{
// 			if let Branch(ref lr) = *v {
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
	fn last_branch<'b>(&'b mut self)-> &'b mut Vec<Wood> where 'a:'b {
		let ret = if self.stack.len() > 0 {
			*get_back_mut(&mut self.stack)
		}else{
			&mut self.root
		};
		unsafe{ &mut *ret }
	}
	fn pinch_off_string_if_eating<'b>(&'b mut self, ending:&'a str) where 'a:'b {
		if self.eating_str_mode {
			let len = ending.as_ptr() as usize - self.started_eating.as_ptr() as usize;
			let s = unsafe{ self.started_eating.get_unchecked(0..len) };
			let line = self.line;
			let column = self.column;
			self.last_branch().push(Leafv(Leaf{line, column, v:s.to_string()}));
			self.started_eating = "";
			self.eating_str_mode = false;
		}
	}
	unsafe fn close_scope<'b>(&'b mut self) where 'a:'b {
		self.stack.pop();
	}
	fn new_scope<'b>(&'b mut self) where 'a:'b {
		let line = self.line;
		let column = self.column;
		self.last_branch().push(Branchv(Branch{
			line:line,
			column:column,
			v:Vec::new()
		}));
		//potential unnecessary indexing in last_mut, `push` already knows the index. Could be mitigated with a place_back
		let flp: *mut Vec<Wood> =
			if let Branchv(ref mut l) = * get_back_mut(self.last_branch()) {
				&mut l.v
			}else{
				panic!("but I thought we just inserted a branch");
			};
		self.stack.push(flp);
	}
	fn parse_sexp(mut self)-> Result<Wood, PositionedError> {
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
		
		Ok(Branchv(Branch{line:-1, column:-1, v:self.root.into()}))
	}
}

pub fn parse_sexp<'a>(v:&'a str)-> Result<Wood, PositionedError> {
	SexpParserState::<'a>::begin(v).parse_sexp()
}




// pub fn parse_nakedbranch<'a>(v:&'a str)-> Result<Wood, PositionedError> {
// 	NakedbranchParserState::<'a>::begin(v).parse()
// }

pub fn parse_nakedbranch<'a>(_v:&'a str)-> Result<Wood, PositionedError> {
	Err(PositionedError{line:-1, column:-1, msg:"not implemented :p".into()})
}









//this has been written in the style of a C++ programmer who doesn't want to waste a single cycle. However: Every use of unsafe has been justified with a comment

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


unsafe fn replace_self<T, F>(v:&mut T, f:F) where F : FnOnce(T)-> T { //for replacing a thing with a transformation of itself. It's actually fairly safe, you just have to guarantee that this transformation, f, doesn't panic, or else this function will drop an uninitialized, or v's drop will be called twice, or something. I don't even know, so don't don't panic or else!
	let res = f(replace(v, uninitialized()));
	forget(replace(v, res));
}

unsafe fn str_from_bounds<'a>(o:*const u8, f:*const u8)-> &'a str {
	std::str::from_utf8_unchecked(std::slice::from_raw_parts(o, f as usize - o as usize))
}

fn is_whitespace(c:char)-> bool { c == ' ' || c == '\t' }

struct ParserState<'a>{
	root: Wood,
	indent_stack: Vec<&'a str>,
	indent_branch_stack: Vec<*mut Vec<Wood>>, //the branchs corresponding to each indent level, into which new lines on that level are inserted
	line_paren_stack: Vec<*mut Wood>,
	cur_char_ptr: *const u8,
	//optimization: Consider making these three an untagged union, since only one is used at a time?:
	stretch_reading_start: *const u8, //used when taking an indent
	leaf_being_read_into: *mut String,
	colon_receptacle: *mut Vec<Wood>,
	last_completed_term_on_line: *mut Wood, //for attaching the next pairing
	multilines_indent: &'a str,
	// previous_line_hanging_term: *mut Wood, //this is the term things will be inserted into if there's an indent.
	iter: std::str::Chars<'a>,
	line: isize,
	column: isize,
	mode: fn(&mut ParserState<'a>, Option<char>)-> Result<(), PositionedError>,
	chosen_style: WoodStyle,
}

#[derive(Clone)]
pub struct WoodStyle {
	pub open:char,
	pub close:char,
	pub pairing:char,
}

static DEFAULT_STYLE:WoodStyle = WoodStyle{ open:'(', close:')', pairing:':' };

impl<'a> ParserState<'a> {
	
	fn style(&self)-> &WoodStyle { &self.chosen_style }
	
	fn a_fail(&self, message:String)-> Result<(), PositionedError> { Err(PositionedError{
		line: self.line,
		column: self.column,
		msg: message,
	}) }
	
	fn mkbranch(&self)-> Wood { Branchv(Branch{ line:self.line, column:self.column, v:Vec::new() }) }
	
	fn start_line(&mut self, c:char)-> Result<(), PositionedError> {
		let bin:*mut Vec<Wood> = *get_back_mut(&mut self.indent_branch_stack); //there is always at least root in the indent_branch_stack
		unsafe{
			(*bin).push(self.mkbranch());
		}
		self.line_paren_stack.clear();
		self.line_paren_stack.push(get_back_mut(unsafe{ &mut *bin }));
		self.last_completed_term_on_line = null_mut();
		self.start_reading_thing(c)
	}
	
	fn consider_collapsing_outer_branch_of_previous_line(&mut self){
		let line_term: *mut Wood = self.line_paren_stack[0];
		let line_branch_length:usize = assume_branch_mut(unsafe{ &mut*line_term }).v.len(); //line_term is always a branch
		if line_branch_length == 1 {
			unsafe{
				replace_self(&mut*line_term, |l|{
					let Branch{ v, .. } = assume_branch(l);
					yank_first(v)
				}) //safe: this_line has been proven to be a branch in assume_branch_mut, so assume_branch cannot panic
			}
		}
	}
	
	fn take_hanging_branch_for_insert(&mut self)-> *mut Vec<Wood> {
		if self.colon_receptacle != null_mut() {
			replace(&mut self.colon_receptacle, null_mut())
		}else{
			&mut unsafe{assume_branch_mut(&mut**get_back_mut(&mut self.line_paren_stack))}.v //safe; always something in parenstack, and it's always a branch
		}
	}
	fn take_hanging_branch_for_new_line(&mut self)-> *mut Vec<Wood> {
		if self.colon_receptacle != null_mut() {
			replace(&mut self.colon_receptacle, null_mut())
		}else{
			if self.line_paren_stack.len() > 1 { //then there's an open paren
				&mut unsafe{assume_branch_mut(&mut**get_back_mut(&mut self.line_paren_stack))}.v //safe; always something in parenstack, and it's always a branch
			}else{ //it's the root line paren stack. A modification may need to be made.
				let rpl = unsafe{&mut*self.line_paren_stack[0]};
				let root_pl_len = assume_branch_mut(rpl).v.len();
				if root_pl_len > 1 { //then it needs to be its own branch
					accrete_branch(rpl);
				}
				&mut assume_branch_mut(rpl).v
			}
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
		let lti = self.mkbranch();
		let branch_for_insert = self.take_hanging_branch_for_insert();
		unsafe{(*branch_for_insert).push(lti)};
		self.line_paren_stack.push(unsafe{&mut *get_back_mut(&mut*branch_for_insert)});
		self.last_completed_term_on_line = null_mut();
	}
	fn close_paren(&mut self)-> Result<(), PositionedError> {
		self.colon_receptacle = null_mut();
		if self.line_paren_stack.len() > 1 {
			self.last_completed_term_on_line = unsafe{ &mut **get_back_mut(&mut self.line_paren_stack)};
			self.line_paren_stack.pop(); //safe: we just checked and confirmed there's something there
			Ok(())
		}else{
			self.a_fail("unmatched paren".into())
		}
	}
	fn take_last_completed_term_on_line(&mut self)-> *mut Wood {
		replace(&mut self.last_completed_term_on_line, null_mut())
	}
	fn open_colon(&mut self)-> Result<(), PositionedError> {
		//seek the back term and accrete over it. If there isn't one, create an empty branch
		self.colon_receptacle = {
			let lt = self.take_last_completed_term_on_line();
			if lt != null_mut() {
				unsafe{ accrete_branch(&mut *lt) }
			}else{
				return Err(PositionedError{line: self.line, column: self.column, msg:"no previous term, cannot open a colon here".into()});
			}
		};
		Ok(())
	}
	fn begin_leaf(&mut self, branch_for_insert:*mut Vec<Wood>) {
		let to_push = Leafv(Leaf{line:self.line, column:self.column, v:String::new()});
		unsafe{(*branch_for_insert).push(to_push)};
		self.last_completed_term_on_line = unsafe{get_back_mut(&mut *branch_for_insert)};
		self.leaf_being_read_into = &mut unsafe{assume_leaf_mut(get_back_mut(&mut *branch_for_insert))}.v;
	}
	fn begin_leaf_with_char(&mut self, branch_for_insert:*mut Vec<Wood>, c:char)-> Result<(), PositionedError> {
		let to_push = Leafv(Leaf{line:self.line, column:self.column, v:String::new()});
		unsafe{(*branch_for_insert).push(to_push)};
		self.last_completed_term_on_line = unsafe{get_back_mut(&mut *branch_for_insert)};
		self.leaf_being_read_into = &mut unsafe{assume_leaf_mut(get_back_mut(&mut *branch_for_insert))}.v;
		if c == '\\' {
			try!(self.read_escaped_char());
		}else{
			unsafe{(*self.leaf_being_read_into).push(c)};
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
				let l = self.take_hanging_branch_for_insert();
				self.begin_leaf(l);
				self.mode = Self::eating_quoted_string;
			},
			c if c == self.style().pairing=> {
				try!(self.open_colon());
				self.mode = Self::seeking_term;
			},
			_=> {
				let l = self.take_hanging_branch_for_insert();
				try!(self.begin_leaf_with_char(l, c));
				self.mode = Self::eating_leaf;
			},
		}
		self.next_col();
		Ok(())
	}
	
	fn pop_indent_stack_down(&mut self, this_indent:&'a str)-> Result<(), PositionedError> {
		loop{
			let containing_indent = *get_back_mut(&mut self.indent_stack); //we can be assured that there is always something at back, because a str can't be smaller than the root indent "" and not be a prefix of it
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
			self.indent_branch_stack.pop();
		}
	}
	
	fn end_unindented_line(&mut self){
		self.consider_collapsing_outer_branch_of_previous_line();
		self.colon_receptacle = null_mut();
	}
	
	fn notice_this_new_indentation<OnSmaller, OnEqual, OnGreater>(&mut self, sc:OnSmaller, ec:OnEqual, gc:OnGreater)-> Result<(), PositionedError> where
		OnSmaller : FnOnce(&mut Self)-> Result<(), PositionedError>,
		OnEqual   : FnOnce(&mut Self)-> Result<(), PositionedError>,
		OnGreater : FnOnce(&mut Self, &'a str)-> Result<(), PositionedError>,
	{
		//there's definitely a thing here, ending indentation
		let this_indent = unsafe{str_from_bounds(self.stretch_reading_start, self.cur_char_ptr)};
		let containing_indent = *get_back_mut(&mut self.indent_stack); //safe: indent function always has something in it
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
							let plhl = slf.take_hanging_branch_for_new_line();
							slf.indent_stack.push(this_indent);
							slf.indent_branch_stack.push(plhl as *mut _);
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
				c if c == self.style().pairing => {
					try!(self.open_colon());
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
		let push = |slf:&mut Self, c:char| unsafe{((*slf.leaf_being_read_into)).push(c)}; //safe: leaf_being_read_into must have been validated before this mode could have been entered
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
		let push_char = |slf:&mut Self, c:char| unsafe{((*slf.leaf_being_read_into)).push(c)}; //safe: leaf_being_read_into must have been validated before this mode could have been entered
		if let Some(c) = co {
			match c {
				'\n'=> {
					self.stretch_reading_start = self.next_char_ptr();
					let ar = unsafe{&mut *self.leaf_being_read_into};
					if ar.chars().all(is_whitespace) {
						//begin multiline string
						ar.clear();
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

	fn eating_leaf(&mut self, co:Option<char>)-> Result<(), PositionedError> {
		let push_char = |slf:&mut Self, c:char| unsafe{((*slf.leaf_being_read_into)).push(c)}; //safe: leaf_being_read_into must have been validated before this mode could have been entered
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
				'"'=> {
					self.notice_quote_immediately_after_thing();
				},
				c if c == self.style().pairing=> {
					try!(self.open_colon());
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
	
	fn notice_paren_immediately_after_thing(&mut self){
		let bl: *mut Wood = self.take_last_completed_term_on_line();
		if bl == null_mut() {
			panic!("notice_paren_immediately_after_thing was called with no previous thing");
		}
		accrete_branch(unsafe{&mut *bl});
		self.line_paren_stack.push(bl);
		self.mode = Self::seeking_term;
		self.next_col();
	}
	
	fn notice_quote_immediately_after_thing(&mut self){
		let lt: *mut Vec<Wood> = unsafe{ &mut assume_branch_mut(&mut **get_back_mut(&mut self.line_paren_stack)).v };
		if unsafe{(*lt).len()} == 0 {
			panic!("notice_quote_immediately_after_thing should not be called after entering an empty paren");
		}
		let bt = get_back_mut(unsafe{ &mut*lt });
		let nl = accrete_branch(bt);
		nl.push(Leafv(Leaf{line:self.line, column:self.column, v:String::new()}));
		self.leaf_being_read_into = &mut assume_leaf_mut(get_back_mut(nl)).v; //safe: just made that
		self.mode = Self::eating_quoted_string;
		self.next_col();
	}

	fn seeking_immediately_after_thing(&mut self, co:Option<char>)-> Result<(), PositionedError> { //generally called after ')' or a closing '"', because '"'s and self.style().opens have slightly different meaning in that context
		if let Some(c) = co {
			match c {
				'"'=> {
					self.notice_quote_immediately_after_thing();
				},
				c if c == self.style().close=> {
					try!(self.close_paren());
					self.next_col();
				},
				c if c == self.style().open=> {
					self.notice_paren_immediately_after_thing();
				},
				c if c == self.style().pairing=> {
					try!(self.open_colon());
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
					let il = self.take_hanging_branch_for_insert();
					try!(self.begin_leaf_with_char(il, c));
					self.mode = Self::eating_leaf;
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
							unsafe{(*slf.leaf_being_read_into).push(c)};
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
					// unsafe{(*self.leaf_being_read_into).push(c)}; //actually, we'll only take the character once it's been confirmed that the indent goes all the way up
					self.stretch_reading_start = self.next_char_ptr();
					self.mode = Self::eating_multiline_later_indent;
					self.next_line();
				},
				_=> {
					unsafe{(*self.leaf_being_read_into).push(c)};
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
						unsafe{(*self.leaf_being_read_into).push('\n')}; //only now do we finalize the newline given
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
					self.consider_collapsing_outer_branch_of_previous_line();
					try!(self.start_line(c));
				},
			}
		}else{
			self.end_unindented_line();
		}
		Ok(())
	}

} //ParserState

fn accrete_branch(v:&mut Wood)-> &mut Vec<Wood> {
	unsafe{
		replace_self(v, |vv|{
			let (line, col) = vv.line_and_col();
			Branchv(Branch{line:line, column:col, v:vec!(vv)})
		}); //safe: branch creation doesn't panic
	}
	&mut assume_branch_mut(v).v
}


pub fn parse_multiline_style<'a>(s:&'a str, style:WoodStyle)-> Result<Wood, PositionedError> { //parses as if it's a file, and each term at root is a separate term. This is not what you want if you expect only a single line, and you want that line to be the root term, but its behaviour is more consistent if you are parsing files
	let mut state = ParserState::<'a>{
		root: branch!(), //a yet empty line
		indent_stack: vec!(""),
		stretch_reading_start: s.as_ptr(),
		cur_char_ptr: s.as_ptr(),
		colon_receptacle: null_mut(),
		last_completed_term_on_line: null_mut(),
		leaf_being_read_into: null_mut(),
		iter: s.chars(),
		line: 0,
		column: 0,
		multilines_indent: "",
		line_paren_stack: vec!(),
		indent_branch_stack: vec!(),
		mode: ParserState::<'a>::seeking_beginning,
		chosen_style: style,
	};
	state.indent_branch_stack = vec!(&mut assume_branch_mut(&mut state.root).v);
	
	
	loop {
		let co = state.iterate_char_iter();
		try!((state.mode)(&mut state, co));
		if co == None { break; }
	}
	
	Ok(state.root)
}

pub fn parse_multiline<'a>(s:&'a str)-> Result<Wood, PositionedError> {
	parse_multiline_style(s, DEFAULT_STYLE.clone())
}

pub fn parse<'a>(s:&'a str)-> Result<Wood, PositionedError> {
	parse_multiline(s).map(|t|{
		let l = assume_branch(t); //parse_multiline only returns branchs
		if l.v.len() == 1 {
			yank_first(l.v) //just confirmed it's there
		}else{
			//then the caller was wrong, it wasn't a single root term, so I guess, they get the whole Branch? Maybe this should be a PositionedError... I dunno about that
			Branchv(l)
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
	// 	let ti = branch!(branch!("hue", branch!("when", "you", "now"), "then", "else"));
	// 	let to:Wood<&'static str> = parse_sexp("hue (when you now) then else").unwrap();
	// 	println!("the parsed is {}", to.to_string());
	// 	assert!(ti == to);
	// }
	
	#[test]
	fn test_tests_term_file() {
		let teststr = read_file_from_root("tests.term");
		let testt = parse_multiline(teststr.as_str()).unwrap();
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
				if let Ok(v) = parse_multiline(t.initial_str()) {
					panic!("in \"{}\" case, this string should not have parsed successfully: {}\n. It parsed to {}", test_name, t.initial_str(), v.to_string());
				}
			}
		}
		
		//failing tests
	}
	
	
	#[test]
	fn idempotence() {
		let term = branch!("how", branch!("about", "that"));
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
	
}
