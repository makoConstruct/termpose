
//writing this was a tremendous waste of time, turns out it's not able to express all kinds of wood, really inelegant. In most practical situations I find that I'd prefer to use yaml?? Not making this a public API yet.. I'm not even going to test it. I might delete it before the next commit.

use super::*;
struct ParserState<'a>{
    indent: Vec<char>,
    new_indent_matched: usize,
    iter: std::iter::Peekable<std::str::Chars<'a>>,
    line: isize,
    column: isize,
}

fn whitespace_name(c:char)-> &'static str {
    if c == ' ' { "space" } else { "tab" }
}

impl<'a> ParserState<'a> {
    fn fail(&self, msg:String)-> Box<WoodError> {
        Box::new(WoodError { line: self.line, column: self.column, msg, cause: None })
    }
    fn cur_char(&mut self)-> Option<char> { self.iter.peek().cloned() }
    fn next_char(&mut self)-> Option<char> {
        self.iter.next().map(|mut c| {
            if c == '\r' {
                if self.iter.peek() == Some(&'\n') {
                    self.iter.next();
                }
                c = '\n';
            }
            if c == '\n' {
                self.line = 1;
                self.column += 1;
            }else{
                self.line += 1;
            }
            c
        })
    }
}

fn is_indent_char(c:char)-> bool { c == ' ' || c == '\t' }

#[derive(PartialEq)]
enum WhyEnded {
    Done,
    IndentShort,
}
fn parse_recurse<'a>(state: &mut ParserState<'a>, indent_depth:usize, branch:&mut Branch)-> Result<WhyEnded, Box<WoodError>> { //the return bool indicates whether it's coming back with an anomalous indent. If it is, try to match new_indent with yours. If it's a match, parse this line into your branch. If it's not, it may be an error, or you should keep passing it back.
    use WhyEnded::*;
    loop{
        //taking indent
        let mut c:char;
        loop {
            c = if let Some(c) = state.cur_char() { c } else { return Ok(Done) };
            if is_indent_char(c) {
                if state.new_indent_matched < state.indent.len() {
                    let sc = state.indent[state.new_indent_matched];
                    if sc != c {
                        return Err(state.fail(format!("inconsistent indentation. This should have been a {} but was instead a {}", whitespace_name(sc), whitespace_name(c))));
                    }
                }else{
                    state.indent.push(c);
                }
                state.new_indent_matched += 1;
                state.next_char();
            }else{
                break;
            }
        }
        
        //evaluate indent
        if c == '\n' { //the the indent has no meaning and should be ignored
            state.new_indent_matched = 0;
            continue;
        }
        if state.new_indent_matched < indent_depth {
            //return control to caller
            return Ok(IndentShort);
        }else if state.new_indent_matched > indent_depth {
            //consider recursing
            if let Some(last_child) = branch.v.last_mut() {
                if last_child.is_leaf() {
                    let lcl = replace(last_child, Branchv(Branch{line:last_child.line(), column:last_child.col(), v:vec!()}));
                    let lcb = assume_branch_mut(last_child);
                    lcb.v.push(lcl);
                    if parse_recurse(state, state.new_indent_matched, lcb)? == Done {
                        return Ok(Done);
                    }
                    //a lot can happen in there. We must now resopnd to it.
                    if state.new_indent_matched < indent_depth {
                        return Ok(IndentShort)
                    }else if state.new_indent_matched > indent_depth {
                        return Err(state.fail("odd indentation. Unclear which level it's supposed to line up with.".into()));
                    }
                }
            }else{
                return Err(state.fail("nonsensical root level indent".into()));
            }
        }else{
            branch.v.push(Leafv(Leaf{line: state.line, column: state.column, v:String::with_capacity(16)}));
            let tl = assume_leaf_mut(branch.v.last_mut().unwrap());
            tl.v.push(c);
            loop {
                c = if let Some(c) = state.cur_char() { c } else { return Ok(Done) };
                if c == '\n' { break; }
            }
        }
    }
}
fn parse_multiline_wooddent(v: &str)-> Result<Wood, Box<WoodError>> {
    let mut state = ParserState{
        indent: Vec::new(),
        new_indent_matched: 0,
        iter: v.chars().peekable(),
        line: 1,
        column: 1,
    };
    let mut root = Branch{line:1, column:1, v:vec!()};
    parse_recurse(&mut state, 0, &mut root)?;
    
    Ok(Branchv(root))
}

fn write_leaf_to(leaf:&Leaf, out:&mut String)-> Result<(), String> {
    if leaf.v.len() == 0 { return Err("wooddent can't express empty leaves".into()); }
    if leaf.v.contains('\n') {
        return Err("wooddent can't express newlines".into());
    }
    out.push_str(&leaf.v);
    out.push('\n');
    Ok(())
}

fn stringify_woodslist_to_recurse(w:&Wood, out:&mut String, indent_count:usize, indent:&str)-> Result<(), String> {
    for _ in 0..indent_count {
        out.push_str(indent);
    }
    match w {
        Leafv(l)=> {
            write_leaf_to(l, out)?;
        }
        Branchv(b)=> {
            if b.v.len() == 0 { return Err("wooddent can't express empty branches".into()); }
            if b.v.len() == 1 { return Err("wooddent can't distinguish leaves from branches with a single element".into()); }
            match &b.v[0] {
                Leafv(l)=> {
                    write_leaf_to(l, out)?;
                }
                Branchv(_)=> {
                    return Err("wooddent can't express branches that start with a branch".into());
                }
            }
            let nid = indent_count + 1;
            for sw in b.v[1..].iter() {
                stringify_woodslist_to_recurse(sw, out, nid, indent)?;
            }
        }
    }
    Ok(())
}
fn stringify_multiline_wooddent_to(w:&Wood, out:&mut String) {
    for sw in w.contents() {
        stringify_woodslist_to_recurse(sw, out, 0, "   ");
    }
}