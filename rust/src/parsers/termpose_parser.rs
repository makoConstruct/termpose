
use super::*;

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
	
	fn move_char_ptr_and_update_line_col(&mut self)-> Option<char> {
		self.cur_char_ptr = self.iter.as_str().as_ptr();
		self.iter.next().and_then(|c|{
			if c == '\n' || c == '\r' {
				//continue over any ensuing '\r's
				let mut ic = self.iter.clone();
				//the weird staggered structure reflects the fact that the first \r after a newline is ignored (because of windows)
				if Some('\r') == ic.next() {
					self.iter.next();
					while Some('\r') == ic.next() {
						self.iter.next();
						self.line += 1;
					}
				}
				self.line += 1;
				self.column = 0;
				Some('\n') //note, if it was a pesky '\r', it wont come through that way
			}else {
				self.column += 1;
				Some(c)
			}
		})
	}
	
	fn next_char_ptr(&self)-> *const u8 { self.iter.as_str().as_ptr() }
	// fn next_col(&mut self){ self.column += 1; }
	// fn next_line(&mut self){ self.line += 1; self.column = 0; }
	
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
					Ok(())
				},
				'\n' => {
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
				},
				'\n' => {
					self.stretch_reading_start = self.next_char_ptr();
					// self.mode = &Self::eating_indentation;
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
				},
				'\n' => {
					self.stretch_reading_start = self.next_char_ptr();
					self.mode = Self::eating_indentation;
				},
				c if c == self.style().pairing => {
					try!(self.open_colon());
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
		let match_fail_message = "escape slash must be followed by a valid escape character code";
		if let Some(nc) = self.move_char_ptr_and_update_line_col() {
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
				},
				'\\'=> {
					try!(self.read_escaped_char());
				},
				'"'=> {
					self.mode = Self::seeking_immediately_after_thing;
				},
				_=> {
					push_char(self, c);
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
				},
				'\n'=> {
					self.stretch_reading_start = self.next_char_ptr();
					self.mode = Self::eating_indentation;
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
				},
				c if c == self.style().close=> {
					try!(self.close_paren());
					self.mode = Self::seeking_immediately_after_thing;
				},
				c if c == self.style().open=> {
					self.notice_paren_immediately_after_thing();
				},
				_=> {
					push_char(self, c);
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
	}

	fn seeking_immediately_after_thing(&mut self, co:Option<char>)-> Result<(), PositionedError> { //generally called after ')' or a closing '"', because '"'s and self.style().opens have slightly different meaning in that context
		if let Some(c) = co {
			match c {
				'"'=> {
					self.notice_quote_immediately_after_thing();
				},
				c if c == self.style().close=> {
					try!(self.close_paren());
				},
				c if c == self.style().open=> {
					self.notice_paren_immediately_after_thing();
				},
				c if c == self.style().pairing=> {
					try!(self.open_colon());
					self.mode = Self::seeking_term;
				},
				' ' | '\t' => {
					self.mode = Self::seeking_term;
				},
				'\n'=> {
					self.stretch_reading_start = self.next_char_ptr();
					self.mode = Self::eating_indentation;
				},
				_=> {
					let il = self.take_hanging_branch_for_insert();
					try!(self.begin_leaf_with_char(il, c));
					self.mode = Self::eating_leaf;
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
				' ' | '\t' => {},
				'\n'=> {
					self.stretch_reading_start = self.next_char_ptr();
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
				},
				_=> {
					unsafe{(*self.leaf_being_read_into).push(c)};
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
				},
				'\n'=> {
					//does not eat the newline
					self.stretch_reading_start = self.next_char_ptr();
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


pub fn parse_multiline_termpose_style<'a>(s:&'a str, style:WoodStyle)-> Result<Wood, PositionedError> { //parses as if it's a file, and each term at root is a separate term. This is not what you want if you expect only a single line, and you want that line to be the root term, but its behaviour is more consistent if you are parsing files
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
		let co = state.move_char_ptr_and_update_line_col();
		try!((state.mode)(&mut state, co));
		if co == None { break; }
	}
	
	Ok(state.root)
}

pub fn parse_multiline_termpose<'a>(s:&'a str)-> Result<Wood, PositionedError> {
	parse_multiline_termpose_style(s, DEFAULT_STYLE.clone())
}

pub fn parse_termpose<'a>(s:&'a str)-> Result<Wood, PositionedError> {
	parse_multiline_termpose(s).map(|t|{
		let l = assume_branch(t); //parse_multiline_termpose only returns branchs
		if l.v.len() == 1 {
			yank_first(l.v) //just confirmed it's there
		}else{
			//then the caller was wrong, it wasn't a single root term, so I guess, they get the whole Branch? Maybe this should be a PositionedError... I dunno about that
			Branchv(l)
		}
	})
}