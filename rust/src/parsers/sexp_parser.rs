
use super::*;

struct SexpParserState<'a>{
	root: Wood,
	paren_stack: Vec<*mut Vec<Wood>>,
	leaf_being_read_into: *mut String,
	iter: std::iter::Peekable<std::str::Chars<'a>>,
	line: isize,
	column: isize,
	mode: fn(&mut SexpParserState<'a>, Option<char>)-> Result<(), PositionedError>,
}

impl<'a> SexpParserState<'a> {
	
	fn a_fail(&self, message:String)-> Result<(), PositionedError> { Err(PositionedError{
		line: self.line,
		column: self.column,
		msg: message,
	}) }
	
	fn mkbranch(&self)-> Wood { Branchv(Branch{ line:self.line, column:self.column, v:Vec::new() }) }
	
	fn branch_for_insert(&mut self)-> *mut Vec<Wood> {
		*get_back_mut(&mut self.paren_stack)
	}
	// fn iterate_char_iter(&mut self)-> Option<char> {
	// 	self.cur_char_ptr = self.iter.as_str().as_ptr();
	// 	self.iter.next()
	// }
	
	fn open_paren(&mut self) {
		let lti = self.mkbranch();
		let branch_for_insert = self.branch_for_insert();
		unsafe{(*branch_for_insert).push(lti)};
		self.paren_stack.push(&mut assume_branch_mut(get_back_mut(unsafe{&mut *branch_for_insert})).v);
		self.mode = Self::seeking_term;
	}
	fn close_paren(&mut self)-> Result<(), PositionedError> {
		if self.paren_stack.len() > 1 {
			self.paren_stack.pop();
			self.mode = Self::seeking_term;
			Ok(())
		}else{
			self.a_fail("unmatched paren".into())
		}
	}
	fn begin_leaf(&mut self){
		let to_push = Leafv(Leaf{line:self.line, column:self.column, v:String::new()});
		let branch_for_insert = self.branch_for_insert();
		unsafe{(*branch_for_insert).push(to_push)};
		self.leaf_being_read_into = &mut unsafe{assume_leaf_mut(get_back_mut(&mut *branch_for_insert))}.v;
	}
	
	fn begin_quote(&mut self){
		self.begin_leaf();
		self.mode = Self::eating_quoted_string;
	}
	
	fn start_reading_thing(&mut self, c:char)-> Result<(), PositionedError> {
		match c {
			')' => {
				try!(self.close_paren());
			},
			'(' => {
				self.open_paren();
			},
			'"'=> {
				self.begin_quote();
			},
			_=> {
				self.begin_leaf();
				self.mode = Self::eating_leaf;
				try!(self.eating_leaf(Some(c)));
			},
		}
		Ok(())
	}
	
	fn seeking_term(&mut self, co:Option<char>)-> Result<(), PositionedError> {
		if let Some(c) = co {
			match c {
				' ' | '\t' => {},
				'\n' | '\r' => {} // \r shouldn't really occur here, but whatever
				_=> {
					try!(self.start_reading_thing(c));
				}
			}
		}
		Ok(())
	}

	fn read_escaped_char(&mut self)-> Result<(), PositionedError> {
		let push = |slf:&mut Self, c:char| unsafe{((*slf.leaf_being_read_into)).push(c)}; //safe: leaf_being_read_into must have been validated before this mode could have been entered
		let nco = self.move_char_ptr_and_update_line_col();
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
		Ok(())
	}
	
	fn move_char_ptr_and_update_line_col(&mut self)-> Option<char> {
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

	fn eating_quoted_string(&mut self, co:Option<char>)-> Result<(), PositionedError> {
		let push_char = |slf:&mut Self, c:char| unsafe{((*slf.leaf_being_read_into)).push(c)}; //safe: leaf_being_read_into must have been validated before this mode could have been entered
		if let Some(c) = co {
			match c {
				'\\'=> {
					try!(self.read_escaped_char());
				},
				'"'=> {
					self.mode = Self::seeking_term;
				},
				_=> {
					push_char(self, c);
				}
			}
		}
		Ok(())
	}

	fn eating_leaf(&mut self, co:Option<char>)-> Result<(), PositionedError> {
		let push_char = |slf:&mut Self, c:char| unsafe{((*slf.leaf_being_read_into)).push(c)}; //safe: leaf_being_read_into must have been validated before this mode could have been entered
		if let Some(c) = co {
			match c {
				'\n' | '\r' => {
					self.mode = Self::seeking_term;
				}
				' ' | '\t' => {
					self.mode = Self::seeking_term;
				}
				'"'=> {
					self.begin_quote();
				}
				')' => {
					try!(self.close_paren());
				}
				'(' => {
					self.open_paren();
				}
				'\\'=> {
					try!(self.read_escaped_char());
				}
				_=> {
					push_char(self, c);
				}
			}
		}
		Ok(())
	}
} //SexpParserState



pub fn parse_multiline_sexp<'a>(s:&'a str)-> Result<Wood, PositionedError> { //parses as if it's a file, and each term at root is a separate term. This is not what you want if you expect only a single line, and you want that line to be the root term, but its behaviour is more consistent if you are parsing files
	let mut state = SexpParserState::<'a>{
		root: branch!(), //a yet empty line
		paren_stack: Vec::new(),
		leaf_being_read_into: null_mut(),
		iter: s.chars().peekable(),
		line: 0,
		column: 0,
		mode: SexpParserState::<'a>::seeking_term,
	};
	state.paren_stack = vec!(&mut assume_branch_mut(&mut state.root).v);
	
	
	loop {
		// let co = state.iterate_char_iter();
		let co = state.move_char_ptr_and_update_line_col();
		try!((state.mode)(&mut state, co));
		if co == None { break; }
	}
	
	Ok(state.root)
}

pub fn parse_sexp<'a>(s:&'a str)-> Result<Wood, PositionedError> {
	parse_multiline_sexp(s).map(|t|{
		let l = assume_branch(t); //parse_multiline_termpose only returns branchs
		if l.v.len() == 1 {
			yank_first(l.v) //just confirmed it's there
		}else{
			//then the caller was wrong, it wasn't a single root term, so I guess, they get the whole Branch? Maybe this should be a PositionedError... I dunno about that
			Branchv(l)
		}
	})
}

