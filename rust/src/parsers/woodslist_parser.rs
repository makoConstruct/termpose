
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
	
	fn begin_quoted(&mut self){
		self.begin_leaf();
		//potentially skip the first character if it's a newline
		match self.iter.peek() {
			Some(&'\n') | Some(&'\r') => {
				self.move_char_ptr_and_update_line_col();
			}
			_=> {}
		}
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
				self.begin_quoted();
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
					self.begin_quoted();
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



pub fn parse_multiline_woodslist<'a>(s:&'a str)-> Result<Wood, PositionedError> { //parses as if it's a file, and each term at root is a separate term. This is not what you want if you expect only a single line, and you want that line to be the root term, but its behaviour is more consistent if you are parsing files
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

pub fn parse_woodslist<'a>(s:&'a str)-> Result<Wood, PositionedError> {
	parse_multiline_woodslist(s).map(|t|{
		let l = assume_branch(t); //parse_multiline_termpose only returns branchs
		if l.v.len() == 1 {
			yank_first(l.v) //just confirmed it's there
		}else{
			//then the caller was wrong, it wasn't a single root term, so I guess, they get the whole Branch? Maybe this should be a PositionedError... I dunno about that
			Branchv(l)
		}
	})
}




pub fn to_woodslist(w:&Wood)-> String {
	let mut ret = String::new();
	inline_stringify_woodslist(w, &mut ret);
	ret
}
pub fn stringify_leaf_woodslist(v:&Leaf, s:&mut String){
	let needs_quotes = v.v.chars().any(|c|{ c == ' ' || c == '\t' || c == '(' || c == ')' });
	if needs_quotes { s.push('"'); }
	push_escaped(s, v.v.as_str());
	if needs_quotes { s.push('"'); }
}
pub fn inline_stringify_woodslist_branch(b:&Branch, s:&mut String){
	s.push('(');
	//space separated
	let mut i = b.v.iter();
	if let Some(ref first) = i.next() {
		inline_stringify_woodslist(first, s);
		while let Some(ref nexto) = i.next() {
			s.push(' ');
			inline_stringify_woodslist(nexto, s);
		}
	}
	s.push(')');
}
pub fn inline_stringify_woodslist(w:&Wood, s:&mut String){
	match *w {
		Branchv(ref b)=> {
			inline_stringify_woodslist_branch(b, s);
		}
		Leafv(ref v)=> {
			stringify_leaf_woodslist(v, s);
		}
	}
}
fn woodslist_inline_length_estimate_for_branch(b:&Branch)-> usize {
	let mut ret = 2; //2 for parens
	if b.v.len() > 0 {
		let spaces_length = b.v.len() - 1;
		ret += b.v.iter().fold(spaces_length, |n, w|{
			n + woodslist_inline_length_estimate(w)
		});
	}
	ret
}
fn woodslist_inline_length_estimate(w:&Wood)-> usize {
	match w {
		&Branchv(ref b)=> {
			woodslist_inline_length_estimate_for_branch(b)
		}
		&Leafv(ref l)=> {
			l.v.len()
		}
	}
}
fn maybe_inline_woodslist_stringification<'a>(w:&'a Wood, column_limit:usize, out:&mut String)-> Option<&'a Branch> { //returns Some Branch that w is iff it did NOT insert it inline (because it didn't have room)
	match w {
		&Branchv(ref b)=> {
			if woodslist_inline_length_estimate_for_branch(b) > column_limit {
				return Some(b);
			}else{
				inline_stringify_woodslist_branch(b, out);
			}
		}
		&Leafv(ref l)=> {
			stringify_leaf_woodslist(l, out);
		}
	}
	None
}
fn do_woodslist_stringification(w:&Wood, indent:&str, indent_depth:usize, column_limit:usize, out:&mut String){
	out.push('\n');
	do_indent(indent, indent_depth, out);
	if let Some(b) = maybe_inline_woodslist_stringification(w, column_limit, out) {
		let mut bi = b.v.iter();
		out.push('(');
		if let Some(fw) = bi.next() {
			if let Some(_fwb) = maybe_inline_woodslist_stringification(fw, column_limit - 1, out) { //column_limit - 1 because there's an opening paren in the line
				//then the first one wont fit in the first one position
				out.push('\n');
				for iw in b.v.iter() {
					do_woodslist_stringification(iw, indent, indent_depth + 1, column_limit, out);
				}
			}else{
				for iw in bi {
					do_woodslist_stringification(iw, indent, indent_depth + 1, column_limit, out);
				}
			}
		}
		out.push('\n');
		do_indent(indent, indent_depth, out);
		out.push(')');
	}
}

/// formats a Wood into readable indented woodslist notation
///
/// # Arguments
///
/// * `column_limit` - column_limit ignores indentation, only limits the length of what's beyond the indentation. The reason is... for a start, that's simpler to implement. If it had a strict limit, deeply indented code would get sort of squashed as it approaches the side, which is visually awkward, and eventually it would have to be allowed to penetrate through the limit, and I didn't want to code that. If you don't expect to indent deeply, this shouldn't make much of a difference to you. Pull requests for a more strictly constraining column limit are welcome.
pub fn indented_woodslist_detail(w:&Wood, indent_is_tab:bool, tab_size:usize, column_limit:usize)-> String {
	let indent_string:String;
	let indent:&str;
	if indent_is_tab {
		indent = "\t";
	}else{
		indent_string = " ".repeat(tab_size);
		indent = indent_string.as_str();
	}
	
	let mut ret = String::new();
	
	//we may need to do a special case for the first level if it's a long branch, in which case, every element should be at depth zero. This differs from the normal case where 
	if let Some(b) = maybe_inline_woodslist_stringification(w, column_limit, &mut ret) {
		for iw in b.v.iter() {
			do_woodslist_stringification(iw, indent, 0, column_limit, &mut ret);
		}
	}
	
	ret
}

/// `indented_woodslist_detail(w, false, 2, 73)`
pub fn indented_woodslist(w:&Wood)-> String {
	indented_woodslist_detail(w, false, 2, 73)
}