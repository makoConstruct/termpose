@:expose class WoodslistParser{
	
	private var root:Seqs;
	private var input:StringBuf;
	private var parenTermStack:Array<Term>;
	private var stringBeingReadInto:Stri;
	private var line:Int;
	private var column:Int;
	private var index:Int;
	private var currentMode:PF;
	
	public function new(s:String){
		input = new StringIterator(s);
		root = new Seqs(new Array(), 0,0);
		parenTermStack = new Array();
		line = 0;
		column = 0;
		index = 0;
		currentMode = null;
	}
	
	public static function escapeIsNeeded(sy:String):Bool {
		for(i in 0...sy.length){
			var c = sy.charCodeAt(i);
			if(c == " ".code ||
			   c == "(".code ||
			   c == "\t".code ||
			   c == "\n".code ||
			   c == "\r".code ||
			   c == ")".code
			){
				return true;
			}
		}
		return false;
	}
	public static function mapArToVect<A,B>(ar:Array<A>, f: A-> B){
		var v = new Array(ar.length);
		for(i in 0...ar.length){
			v[i] = f(ar[i]);
		}
		return v;
	}
	public static function escapeSymbol(sb:StringBuf, str:String){
		for(i in 0 ... str.length){
			switch(str.charCodeAt(i)){
				case "\\".code:
					sb.addChar("\\".code);
					sb.addChar("\\".code);
				case "\"".code:
					sb.addChar("\\".code);
					sb.addChar("\"".code);
				case "\n".code:
					sb.addChar("\\".code);
					sb.addChar("n".code);
				case "\r".code:
					sb.addChar("\\".code);
					sb.addChar("r".code);
				case c:
					sb.addChar(c);
			}
		}
	}
	public static function array<T>(el:T){var ret = new Array(); ret.push(el); return ret;}
	private function emptyBranch():Term { return new Seqs([], line, column); }
	private function emptyLeaf():Term { return new Stri("", line, column); }
	private function transition(nm:PF){ currentMode = nm; }
	private function giveUp(message:String){ breakAt(line, column, message); }
	private function breakAt(l:Int, c:Int, message:String){ throw new ParsingException("line:"+l+" column:"+c+", no, bad: "+message); }
	
	
	private function openParen():Seqs {
		var s = emptyBranch();
		parenTermStack.back().push(s);
		parenTermStack.push(s);
		transition(seekingTerm);
		return s;
	}
	private function closeParen() {
		if(parenTermStack.len() > 1){
			parenTermStack.pop();
			transition(seekingTerm);
		}else{
			giveUp("unmatched paren");
		}
	}
	private function lastList():Array<Term> {
		return parenTermStack.back().s;
	}
	private function beginLeaf():Seqs {
		var n = emptyLeaf();
		lastList().push(n);
		stringBeingReadInto = n;
		return n;
	}
	
	private function beginQuoted(){
		var l = beginLeaf();
		//potentially skip the first character if it's a newline
		var nc = input.peek()
		if(nc == '\n'.code || nc == '\r'.code){
			moveCharPtrAndUpdateLineCol();
		}
		transition(eatingQuotedString);
	}
	
	private function startReadingThing(c:Null<Int>){
		switch(c){
			case ')'.code:
				closeParen();
			case '('.code:
				openParen();
			case '"'.code:
				beginQuoted();
			case _: {
				beginLeaf();
				transition(eatingLeaf);
				eatingLeaf(Some(c));
			}
		}
	}
	
	private function seekingTerm(co:Null<Int>){
		switch(co){
			case ' '.code:
			case '\t'.code:
			case '\n'.code:
			case '\r'.code:
			case _: {
				startReadingThing(co);
			}
		}
	}

	private function readEscapedChar(){
		var push = function(c:Int){ stringBeingReadInto.push(c) };
		var nco = moveCharPtrAndUpdateLineCol();
		var matchFailMessage = "escape slash must be followed by a valid escape character code";
		if(nco != null){
			switch(nco){
				case 'n'.code: { push('\n'.code); }
				case 'r'.code: { push('\r'.code); }
				case 't'.code: { push('\t'.code); }
				case 'h'.code: { push('☃'.code); }
				case '"'.code: { push('"'.code); }
				case '\\'.code: { push('\\'.code); }
				case _: { giveUp(matchFailMessage); }
			}
		}else{
			giveUp(matchFailMessage);
		}
	}
	
	private function moveCharPtrAndUpdateLineCol():Null<Int>{
		var nco = input.next();
		if(nco){
			if(nco == '\r'.code){
				if('\n' == input.peek()){ //crlf support
					nco = input.next();
				}
				line += 1;
				column = 0;
				return '\n'.code //if it was a pesky '\r', it wont come through that way
			}else if(nco == '\n'.code){
				line += 1;
				column = 0;
			}else{
				self.column += 1;
			}
		}
		return nco;
	}

	private function eatingQuotedString(co:Null<Int>) {
		var push = function(c:Int){ stringBeingReadInto.push(c) };
		switch(co){
			case null: {
				giveUp("unmatched quote, by the end of data");
			}
			case '\\'.code: {
				readEscapedChar();
			}
			case '"'.code: {
				transition(seekingTerm);
			}
			case _: {
				pushChar(co);
			}
		}
	}

	private function eatingLeaf(co:Null<char>) {
		var push = function(c:Int){ stringBeingReadInto.push(c) };
		switch(co){
			case '\n'.code | '\r'.code: {
				transition(seekingTerm);
			}
			case ' '.code | '\t'.code: {
				transition(seekingTerm);
			}
			case '"'.code: {
				beginQuoted();
			}
			case ')'.code : {
				closeParen();
			}
			case '('.code : {
				openParen();
			}
			case '\\'.code: {
				readEscapedChar();
			}
			case _: {
				push(co);
			}
		}
	}
	
	
	
	public function parseMultilineWoodslist(s:String):Term { //parses as if it's a file, and each term at root is a separate term. This is not what you want if you expect only a single line, and you want that line to be the root term, but its behaviour is more consistent if you are parsing files
		var p = new WoodsListParser();
		p.init(s);
		
		while(true){
			let co = moveCharPtrAndUpdateLineCol();
			currentMode(co);
			if(co == null){ break; }
		}
		
		return p.root;
	}
}