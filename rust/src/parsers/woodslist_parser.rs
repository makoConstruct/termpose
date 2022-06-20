use super::*;

struct SexpParserState<'a> {
    iter: std::iter::Peekable<std::str::Chars<'a>>,
    line: isize,
    column: isize,
}

#[derive(PartialEq)]
enum HowEnded {
    FoundParen,
    FoundEOF,
}

impl<'a> SexpParserState<'a> {
    fn a_fail(&self, message: String) -> Box<WoodError> {
        Box::new(WoodError {
            line: self.line,
            column: self.column,
            msg: message,
            cause: None,
        })
    }

    fn move_char_ptr_and_update_line_col(&mut self) -> Option<char> {
        self.iter.next().and_then(|c| {
            if c == '\r' {
                if Some(&'\n') == self.iter.peek() {
                    //crlf support
                    self.iter.next();
                }
                self.line += 1;
                self.column = 0;
                Some('\n') //if it was a pesky '\r', it wont come through that way
            } else if c == '\n' {
                self.line += 1;
                self.column = 0;
                Some(c)
            } else {
                self.column += 1;
                Some(c)
            }
        })
    }

    fn read_escaped_char(&mut self) -> Result<char, Box<WoodError>> {
        let nco = self.move_char_ptr_and_update_line_col();
        let match_fail_message = "escape slash must be followed by a valid escape character code";
        if let Some(nc) = nco {
            match nc {
                'n' => Ok('\n'),
                'r' => Ok('\r'),
                't' => Ok('\t'),
                'h' => Ok('☃'),
                '"' => Ok('"'),
                '\\' => Ok('\\'),
                _ => Err(self.a_fail(match_fail_message.into())),
            }
        } else {
            Err(self.a_fail(match_fail_message.into()))
        }
    }

    fn seeking(&mut self, into: &mut Branch) -> Result<HowEnded, Box<WoodError>> {
        while let Some(c) = self.move_char_ptr_and_update_line_col() {
            match c {
                '(' => {
                    into.v.push(Branchv(Branch {
                        line: self.line,
                        column: self.column,
                        v: Vec::new(),
                    }));
                    let b = assume_branch_mut(into.v.last_mut().unwrap());
                    let how_inner_ended = self.seeking(b)?;
                    if HowEnded::FoundEOF == how_inner_ended {
                        return Err(Box::new(WoodError {
                            msg: "unmatched opening paren".into(),
                            line: b.line,
                            column: b.column,
                            cause: None,
                        }));
                    }
                }
                ')' => {
                    return Ok(HowEnded::FoundParen);
                }
                ' ' | '\t' | '\n' => {}
                '"' => {
                    into.v.push(Leafv(Leaf {
                        line: self.line,
                        column: self.column,
                        v: String::new(),
                    }));
                    let reading_into = assume_leaf_mut(into.v.last_mut().unwrap());
                    let get_char = |this:&mut SexpParserState| {
                        this.move_char_ptr_and_update_line_col().ok_or_else(|| {
                            this.a_fail(format!("unclosed string starting at "))
                        })
                    };
                    let mut c = get_char(self)?;
                    if c == '\n' {
                        //skip any initial newline, to allow the user to get everything lined up at the baseline, if they want.
                        c = get_char(self)?;
                    }
                    loop {
                        match c {
                            '"' => {
                                break;
                            }
                            '\\' => {
                                reading_into.v.push(self.read_escaped_char()?);
                            }
                            c => {
                                reading_into.v.push(c);
                            }
                        }
                        c = get_char(self)?;
                    }
                }
                mut c => {
                    into.v.push(Leafv(Leaf {
                        line: self.line,
                        column: self.column,
                        v: String::new(),
                    }));
                    let reading_into = assume_leaf_mut(into.v.last_mut().unwrap());
                    loop {
                        match c {
                            '\\' => {
                                reading_into.v.push(self.read_escaped_char()?);
                            }
                            c => {
                                reading_into.v.push(c);
                            }
                        }
                        //return control without advancing it again iff the next character is interrupty, the next char can be dealt with by the outer loop
                        if let Some(nc) = self.iter.peek() {
                            match *nc {
                                ' ' | '\t' | '\n' | '"' | '(' | ')' => {
                                    break;
                                }
                                _ => {}
                            }
                        } else {
                            return Ok(HowEnded::FoundEOF);
                        }
                        c = self.move_char_ptr_and_update_line_col().unwrap();
                    }
                }
            }
        }
        Ok(HowEnded::FoundEOF)
    }
}

pub fn parse_multiline_woodslist<'a>(s: &'a str) -> Result<Wood, Box<WoodError>> {
    //parses as if it's a file, and each term at root is a separate term. This is not what you want if you expect only a single line, and you want that line to be the root term, but its behaviour is more consistent if you are parsing files
    let mut state = SexpParserState {
        iter: s.chars().peekable(),
        line: 1,
        column: 1,
    };

    let mut root_branch = Branch {
        column: 1,
        line: 1,
        v: vec![],
    };

    match state.seeking(&mut root_branch)? {
        HowEnded::FoundEOF => Ok(Branchv(root_branch)),
        HowEnded::FoundParen => Err(Box::new(WoodError {
            msg: "unmatched closing paren".into(),
            line: state.line,
            column: state.column,
            cause: None,
        })),
    }
}

pub fn parse_woodslist<'a>(s: &'a str) -> Result<Wood, Box<WoodError>> {
    parse_multiline_woodslist(s).map(|t| {
        let l = assume_branch(t); //parse_multiline_termpose only returns branchs
        if l.v.len() == 1 {
            yank_first(l.v) //just confirmed it's there
        } else {
            //then the caller was wrong, it wasn't a single root term, so I guess, they get the whole Branch? Maybe this should be a PositionedError... I dunno about that
            Branchv(l)
        }
    })
}

pub fn to_woodslist(w: &Wood) -> String {
    let mut ret = String::new();
    inline_stringify_woodslist(w, &mut ret);
    ret
}
pub fn stringify_leaf_woodslist(v: &Leaf, s: &mut String) {
    let needs_quotes =
        v.v.chars()
            .any(|c| c == ' ' || c == '\t' || c == '(' || c == ')');
    if needs_quotes {
        s.push('"');
    }
    push_escaped(s, v.v.as_str());
    if needs_quotes {
        s.push('"');
    }
}
pub fn inline_stringify_woodslist_branch(b: &Branch, s: &mut String) {
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
pub fn inline_stringify_woodslist(w: &Wood, s: &mut String) {
    match *w {
        Branchv(ref b) => {
            inline_stringify_woodslist_branch(b, s);
        }
        Leafv(ref v) => {
            stringify_leaf_woodslist(v, s);
        }
    }
}
fn woodslist_inline_length_estimate_for_branch(b: &Branch) -> usize {
    let mut ret = 2; //2 for parens
    if b.v.len() > 0 {
        let spaces_length = b.v.len() - 1;
        ret += b.v.iter().fold(spaces_length, |n, w| {
            n + woodslist_inline_length_estimate(w)
        });
    }
    ret
}
fn woodslist_inline_length_estimate(w: &Wood) -> usize {
    match w {
        &Branchv(ref b) => woodslist_inline_length_estimate_for_branch(b),
        &Leafv(ref l) => l.v.len(),
    }
}
fn maybe_inline_woodslist_stringification<'a>(
    w: &'a Wood,
    column_limit: usize,
    out: &mut String,
) -> Option<&'a Branch> {
    //returns Some Branch that w is iff it did NOT insert it inline (because it didn't have room)
    match w {
        &Branchv(ref b) => {
            if woodslist_inline_length_estimate_for_branch(b) > column_limit {
                return Some(b);
            } else {
                inline_stringify_woodslist_branch(b, out);
            }
        }
        &Leafv(ref l) => {
            stringify_leaf_woodslist(l, out);
        }
    }
    None
}
fn do_woodslist_stringification(
    w: &Wood,
    indent: &str,
    indent_depth: usize,
    column_limit: usize,
    out: &mut String,
) {
    out.push('\n');
    do_indent(indent, indent_depth, out);
    if let Some(b) = maybe_inline_woodslist_stringification(w, column_limit, out) {
        let mut bi = b.v.iter();
        out.push('(');
        if let Some(fw) = bi.next() {
            if let Some(_fwb) = maybe_inline_woodslist_stringification(fw, column_limit - 1, out) {
                //column_limit - 1 because there's an opening paren in the line
                //then the first one wont fit in the first one position
                out.push('\n');
                for iw in b.v.iter() {
                    do_woodslist_stringification(iw, indent, indent_depth + 1, column_limit, out);
                }
            } else {
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
pub fn indented_woodslist_detail(
    w: &Wood,
    indent_is_tab: bool,
    tab_size: usize,
    column_limit: usize,
) -> String {
    let indent_string: String;
    let indent: &str;
    if indent_is_tab {
        indent = "\t";
    } else {
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
pub fn indented_woodslist(w: &Wood) -> String {
    indented_woodslist_detail(w, false, 2, 73)
}
