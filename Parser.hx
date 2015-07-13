import haxe.ds.Vector;

class Stringificable{ //has a method for stringifying through a StringBuf, provides a toString through this
	function stringify(sb:StringBuf){}
	function toString():String {
		var buf = new StringBuf();
		stringify(buf);
		return buf.toString();
	}
}

// enum Try<T>{
// 	Success(result:T);
// 	Failure(msg:String);
// }
// private function map<T,D>(ty:Try<T>, f:T->D):Try<D> {
// 	switch (ty) {
// 		case Success(v):   return Success(f(ty));
// 		case Failure(msg): return Failure(msg);
// 	}
// }

@:expose interface BufferedIterator<T>{
	function peek():Null<T>;
	function next():T;
	function hasNext():Bool;
}

class BufferedIteratorFromIterator<T> implements BufferedIterator<T>{
	var hasValue:Bool;
	var value:Null<T>;
	var i:Iterator<T>;
	public function new(i:Iterator<T>){
		this.i = i;
		cycle();
	}
	private function cycle(){
		hasValue = i.hasNext();
		if(hasValue){
			value = i.next();
		}else{
			value = null;
		}
	}
	public function peek():Null<T>{ return value; }
	public function next():T{
		var ret = value;
		cycle();
		return ret;
	}
	public function hasNext():Bool{ return hasValue; }
}


class StringIterator implements BufferedIterator<Int> {
	var i:Int;
	var s:String;
	public function new(v:String){
		s = v;
		i = 0;
	}
	public function hasNext():Bool {return i < s.length;}
	public function next():Int {
		var ret = s.charCodeAt(i);
		i = i + 1;
		return ret;
	}
	public function peek():Null<Int> {if(i < s.length){return s.charCodeAt(i);}else{return null;}}
}



@:expose class Term extends Stringificable{
	var line:Int;
	var column:Int;
	function jsonString():String{
		var sb = new StringBuf();
		buildJsonString(sb);
		return sb.toString();
	}
	function buildJsonString(sb:StringBuf){}
}

@:expose class Stri extends Term{
	public var sy:String;
	public function new(sy:String, line:Int, column:Int){
		this.sy = sy;
		this.line = line;
		this.column = column;
	}
	override function stringify(sb:StringBuf){
		if(Parser.escapeIsNeeded(sy)){
			sb.addChar('"'.code);
			Parser.escapeSymbol(sb, sy);
			sb.addChar('"'.code);
		}else{
			sb.addSub(sy,0);
		}
	}
	override function buildJsonString(sb:StringBuf){
		sb.addChar('"'.code);
		Parser.escapeSymbol(sb, sy);
		sb.addChar('"'.code);
	}
}

@:expose class Seqs extends Term{
	public var s:Vector<Term>;
	public function new(s:Vector<Term>, line:Int, column:Int){
		this.s = s;
		this.line = line;
		this.column = column;
	}
	// function name():String{
	// 	if(s.length >= 1){
	// 		switch(s(0)){
	// 			case Stri(sy): return sy;
	// 			case _: return "";
	// 		}
	// 	}else{ return ""; }
	// }
	override function stringify(sb:StringBuf){
		sb.addChar('('.code);
		if(s.length >= 1){
			s.get(0).stringify(sb);
			for(i in 1...s.length){
				sb.addChar(' '.code);
				s.get(i).stringify(sb);
			}
		}
		sb.addChar(')'.code);
	}
	override function buildJsonString(sb:StringBuf){
		sb.addChar('['.code);
		if(s.length > 0){
			s.get(0).buildJsonString(sb);
		}
		for(i in 1...s.length){
			sb.addChar(','.code);
			s.get(i).buildJsonString(sb);
		}
		sb.addChar(']'.code);
	}
}


@:expose class ParsingException{
	var msg:String;
	public function new(s:String){msg = s;}
}

private interface InterTerm{
	var line:Int;
	var column:Int;
	function toTerm():Term;
	var pointing:PointsInterTerm;
}
private class PointsInterTerm{ //sometimes an interterm might want to reattach partway through the process. It should be attached through a PointsInterTerm which it knows so it can do that.
	public var t:InterTerm;
	public function new(t:InterTerm){this.t = t;}
}

private class Sq implements InterTerm{
	public var s:Array<PointsInterTerm>;
	public var line:Int;
	public var column:Int;
	public var pointing:PointsInterTerm;
	public function new(s:Array<PointsInterTerm>, line:Int, column:Int, pointing:PointsInterTerm){
		this.s = s;
		this.line = line;
		this.column = column;
		this.pointing = pointing;
		pointing.t = this;
	}
	public function toTerm():Term{return toSeqs();}
	public function toSeqs(){ return new Seqs(Parser.mapArToVect(s, function(e){return e.t.toTerm();}), line, column); }
}
private class St implements InterTerm{
	public var sy:String;
	public var line:Int;
	public var column:Int;
	public var pointing:PointsInterTerm;
	public function new(sy:String, line:Int, column:Int, pointing:PointsInterTerm){
		this.sy = sy;
		this.line = line;
		this.column = column;
		this.pointing = pointing;
		this.pointing.t = this;
	}
	public function toTerm():Term{ return new Stri(sy, line, column); }
}

typedef PF = Bool -> Int -> Void;

@:expose class Parser{
	public static function prefixes(shorter:String, longer:String):Bool{
		if(shorter.length > longer.length){
			return false;
		}else{
			for(i in 0...shorter.length){
				if(shorter.charCodeAt(i) != longer.charCodeAt(i)){
					return false;
				}
			}
			return true;
		}
	}
	public static function escapeIsNeeded(sy:String):Bool {
		for(i in 0...sy.length){
			var c = sy.charCodeAt(i);
			if(c == " ".code ||
			   c == "(".code ||
			   c == ":".code ||
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
		var v = new Vector(ar.length);
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
	
	private static function dropSeqLayerIfSole(pt:PointsInterTerm){ //assumes pt is Sq
		var arb = cast(pt.t, Sq).s;
		if(arb.length == 1){
			var soleEl = arb[0].t;
			soleEl.pointing = pt;
			pt.t = soleEl;
		}
	}
	private function growSeqsLayer(pi:PointsInterTerm):Sq {
		var oldEl = pi.t;
		var newPIT = new PointsInterTerm(oldEl);
		oldEl.pointing = newPIT;
		var newSq = new Sq(array(newPIT), line, column, pi);
		return newSq;
	}
	private function interSq(ar:Array<PointsInterTerm>):PointsInterTerm {
		if(ar == null) ar = new Array();
		var pt = new PointsInterTerm(null);
		new Sq(ar, line, column, pt);
		return pt;
	}
	private function emptyInterSq():PointsInterTerm{return interSq(new Array());}
	private function interSt(sy:String):PointsInterTerm {
		var pt = new PointsInterTerm(null);
		new St(sy, line, column, pt);
		return pt;
	}
	
	private var stringBuffer:StringBuf;
	private var previousIndentation:String;
	private var salientIndentation:String;
	private var hasFoundLine:Bool;
	private var rootArBuf:Array<PointsInterTerm>;
	private var parenTermStack:Array<PointsInterTerm>;
	private var line:Int;
	private var column:Int;
	private var index:Int;
	private var lastAttachedTerm:PointsInterTerm;
	private var currentMode:PF;
	private var modes:Array<PF>;
	private var indentStack:Array<{depth:Int, contents:Array<PointsInterTerm>}>;  //indentation length, tailestTermSequence. It is enough to keep track of length. We can derive ∀(a,b∈Line.Indentation) a.length = b.length → a = b from the prefix checking that is done before adding to the stack
	private var multilineStringIndentBuffer:StringBuf;
	private var multilineStringsIndent:String;
	private var containsImmediateNext:Sq;
	
	private function init(){
		stringBuffer = new StringBuf();
		previousIndentation = "";
		salientIndentation = "";
		hasFoundLine = false;
		parenTermStack = new Array();
		line = 0;
		column = 0;
		index = 0;
		currentMode = null;
		lastAttachedTerm = null;
		modes = new Array();
		rootArBuf = new Array();
		indentStack = array({depth:0, contents:rootArBuf});
		multilineStringIndentBuffer = new StringBuf();
		multilineStringsIndent = null;
	}
	
	
	//general notes:
	//the parser consists of a loop that pumps chars from the input string into whatever the current Mode is. Modes jump from one to the next according to cues, sometimes forwarding the cue onto the new mode before it gets any input from the input loop. There is a mode stack, but it is rarely used. Modes are essentially just a Char => Void (named 'PF', short for Processing Funnel(JJ it's legacy from when they were Partial Functions)). CR LFs ("\r\n"s) are filtered to a single LF (This does mean that a windows formatted file will have unix line endings when parsing multiline strings, that is not a significant issue, ignoring the '\r's will probably save more pain than it causes and the escape sequence \r is available if they're explicitly desired).
	//terms are not attached until they are fully formed. You see, the type and address of a term can change when brackets are appended after it. Say you're reading a(f)(g), you finish reading "a", now you have a symbol. Can you attach it yet? No, because it becomes a Seqs when you find the (, and once you find the ), can you attach the Seqs? No, because it needs to be wrapped in a new seqs when You come to the next (, only once you're sure there're no more parens can you attach the thing.
	//I think.
	//There are other ways I could have handled this, but figuring out which one is best seems like it'd take more effort than just finishing this as it is.
	
	// key aspects of global state to regard:
	// stringBuffer:StringBuf    where indentation and symbols are collected and cut off
	// indentStack:Array[(Int, Array[InterTerm])]    encodes the levels of indentation we've traversed and the parent container for each level
	// parenStack:Array[Array[InterTerm]]    encodes the levels of parens we've traversed and the container for each level. The first entry is the root line, which has no parens.
	private function transition(nm:PF){ currentMode = nm; }
	private function pushMode(nm:PF){
		modes.push(currentMode);
		transition(nm);
	}
	private function popMode(){
		currentMode = modes.pop();
	}
	private function giveUp(message:String){ breakAt(line, column, message); }
	private function breakAt(l:Int, c:Int, message:String){ throw new ParsingException("line:"+l+" column:"+c+", no, bad: "+message); }
	private function receiveForemostRecepticle():Array<PointsInterTerm>{ //note side effects
		if(containsImmediateNext != null){
			var ret = containsImmediateNext.s;
			containsImmediateNext = null;
			return ret;
		}else{
			if(parenTermStack.length == 0){
				var rootParenLevel = emptyInterSq();
				parenTermStack.push(rootParenLevel);
				indentStack[indentStack.length-1].contents.push(rootParenLevel);
			}
			return (cast(parenTermStack[parenTermStack.length-1].t, Sq)).s;
		}
	}
	private function attach(t:PointsInterTerm){
		receiveForemostRecepticle().push(t);
		lastAttachedTerm = t;
	}
	private function finishTakingSymbolAndAttach():PointsInterTerm{
		var newSt = interSt(stringBuffer.toString());
		attach(newSt);
		stringBuffer = new StringBuf();
		return newSt;
	}
	private function finishTakingIndentationAndAdjustLineAttachment(){
		//Iff there is no indented content or that indented content falls within an inner paren(iff the parenTermStack is longer than one), and the line only has one item at root, the root element in the parenstack should drop a seq layer so that it is just that element. In all other case, leave it as a sequence.
		
		previousIndentation = salientIndentation;
		salientIndentation = stringBuffer.toString();
		stringBuffer = new StringBuf();
		if(hasFoundLine){
			if(salientIndentation.length > previousIndentation.length){
				if(! prefixes(previousIndentation, salientIndentation)){
					breakAt(line - salientIndentation.length, column, "inconsistent indentation at");
				}
				
				if(parenTermStack.length > 1 || containsImmediateNext != null) dropSeqLayerIfSole(parenTermStack[0]); //if antecedents and IsSole, the root element contains the indented stuff
				
				indentStack.push({depth:salientIndentation.length, contents:receiveForemostRecepticle()});
			}else{
				
				dropSeqLayerIfSole(parenTermStack[0]);
				
				containsImmediateNext = null;
				if(salientIndentation.length < previousIndentation.length){
					if(! prefixes(salientIndentation, previousIndentation)){
						breakAt(line, column, "inconsistent indentation");
					}
					//pop to enclosing scope
					while(indentStack[indentStack.length-1].depth > salientIndentation.length){
						if(indentStack[indentStack.length-1].depth < salientIndentation.length){
							breakAt(line, column, "inconsistent indentation, sibling elements have different indentation");
						}
						indentStack.pop();
					}
				}
			}
			parenTermStack = new Array();
		}else{
			hasFoundLine = true;
		}
	}
	private function closeParen(){
		if(parenTermStack.length <= 1) //not supposed to be <= 0, the bottom level is the root line, and must be retained
			giveUp("unbalanced paren");
		containsImmediateNext = null;
		parenTermStack.pop();
	}
	private function receiveFinishedSymbol():PointsInterTerm{
		var ret = interSt(stringBuffer.toString());
		stringBuffer = new StringBuf();
		return ret;
	}
	private function eatingIndentation(fileEnd:Bool, c:Int){
		if(fileEnd){
			stringBuffer = new StringBuf();
			finishTakingIndentationAndAdjustLineAttachment();
		}else switch(c){
			case '\n'.code:
				stringBuffer = new StringBuf();
			case ':'.code | '('.code:
				finishTakingIndentationAndAdjustLineAttachment();
				transition(seekingTerm);
				seekingTerm(false,c);
			case ')'.code:
				giveUp("nothing to close");
			case '"'.code:
				finishTakingIndentationAndAdjustLineAttachment();
				transition(buildingQuotedSymbol);
			case ' '.code | '\t'.code:
				stringBuffer.addChar(c);
			case c:
				finishTakingIndentationAndAdjustLineAttachment();
				transition(buildingSymbol);
				buildingSymbol(false,c);
		}
	}
	private function seekingTerm(fileEnd:Bool, c:Int){
		if(fileEnd){
			finishTakingIndentationAndAdjustLineAttachment();
		}else switch(c){
			case '('.code :
				var newSq = emptyInterSq();
				attach(newSq);
				parenTermStack.push(newSq);
			case ')'.code :
				closeParen();
				transition(immediatelyAfterTerm);
			case ':'.code :
				var newSq = emptyInterSq();
				attach(newSq);
				containsImmediateNext = cast(newSq.t, Sq);
			case '\n'.code :
				transition(eatingIndentation);
			case ' '.code | '\t'.code :
			case '"'.code :
				transition(buildingQuotedSymbol);
			case c:
				transition(buildingSymbol);
				buildingSymbol(false, c);
		}
	}
	private function immediatelyAfterTerm(fileEnd:Bool, c:Int){
		if(fileEnd){
			finishTakingIndentationAndAdjustLineAttachment();
		}else switch(c){
			case '('.code :
				var newLevel = lastAttachedTerm;
				growSeqsLayer(newLevel);
				parenTermStack.push(newLevel);
				transition(seekingTerm);
			case ')'.code :
				closeParen();
			case ':'.code :
				containsImmediateNext = growSeqsLayer(lastAttachedTerm);
				transition(seekingTerm);
			case '\n'.code :
				transition(eatingIndentation);
			case ' '.code | '\t'.code :
				transition(seekingTerm);
			case '"'.code :
				containsImmediateNext = growSeqsLayer(lastAttachedTerm);
				transition(buildingQuotedSymbol);
			case c :
				giveUp("You have to put a space here. Yes I know the fact that I can say that means I could just pretend there's a space there and let you go ahead, but I wont be doing that, as I am an incorrigible formatting nazi.");
		}
	}
	private function buildingSymbol(fileEnd:Bool, c:Int){
		if(fileEnd){
			finishTakingSymbolAndAttach();
			finishTakingIndentationAndAdjustLineAttachment();
		}else switch(c){
			case ' '.code | '\t'.code :
				finishTakingSymbolAndAttach();
				transition(seekingTerm);
			case ':'.code | '\n'.code | '('.code | ')'.code :
				finishTakingSymbolAndAttach();
				transition(immediatelyAfterTerm);
				immediatelyAfterTerm(false,c);
			case '"'.code :
				finishTakingSymbolAndAttach();
				transition(immediatelyAfterTerm);
				immediatelyAfterTerm(false,c);
			case c :
				stringBuffer.addChar(c);
		}
	}
	private function buildingQuotedSymbol(fileEnd:Bool, c:Int){
		if(fileEnd){
			finishTakingSymbolAndAttach();
			finishTakingIndentationAndAdjustLineAttachment();
		}else switch(c){
			case '"'.code :
				finishTakingSymbolAndAttach();
				transition(immediatelyAfterTerm);
			case '\\'.code :
				pushMode(takingEscape);
			case '\n'.code :
				if(stringBuffer.length == 0){
					transition(multiLineFirstLine);
				}else{
					finishTakingSymbolAndAttach();
					transition(eatingIndentation);
				}
			case c :
				stringBuffer.addChar(c);
		}
	}
	private function takingEscape(fileEnd:Bool, c:Int){
		if(fileEnd)
			giveUp("invalid escape sequence, no one can escape the end of the file");
		else{
			switch(c){
				case 'h'.code : stringBuffer.addChar('☃'.code);
				case 'n'.code : stringBuffer.addChar('\n'.code);
				case 'r'.code : stringBuffer.addChar('\r'.code);
				case 't'.code : stringBuffer.addChar('\t'.code);
				case g : stringBuffer.addChar(g);
			}
			popMode();
		}
	}
	private function multiLineFirstLine(fileEnd:Bool, c:Int){
		if(fileEnd){
			finishTakingSymbolAndAttach();
			finishTakingIndentationAndAdjustLineAttachment();
		}else switch(c){
			case ' '.code | '\t'.code :
				multilineStringIndentBuffer.addChar(c);
			case c :
				multilineStringsIndent = multilineStringIndentBuffer.toString();
				if(multilineStringsIndent.length > salientIndentation.length){
					if(prefixes(salientIndentation, multilineStringsIndent)){
						transition(multiLineTakingText);
						multiLineTakingText(false,c);
					}else{
						giveUp("inconsistent indentation");
					}
				}else{
					finishTakingSymbolAndAttach();
					//transfer control to eatingIndentation
					stringBuffer = new StringBuf();
					stringBuffer.addSub(multilineStringsIndent,0);
					multilineStringsIndent = null;
					transition(eatingIndentation);
					eatingIndentation(false,c);
				}
				multilineStringIndentBuffer = new StringBuf();
		}
	}
	private function multiLineTakingIndent(fileEnd:Bool, c:Int){
		if(fileEnd){
			finishTakingSymbolAndAttach();
			finishTakingIndentationAndAdjustLineAttachment();
		}else if(c == ' '.code || c == '\t'.code){
			multilineStringIndentBuffer.addChar(c);
			if(multilineStringIndentBuffer.length == multilineStringsIndent.length){ //then we're through with the indent
				if(prefixes(multilineStringsIndent, multilineStringsIndent)){
					//now we know that it continues, we can insert the endline from the previous line
					stringBuffer.addChar('\n'.code);
					transition(multiLineTakingText);
				}else{
					giveUp("inconsistent indentation");
				}
				multilineStringIndentBuffer = new StringBuf();
			}
		}else if(c == '\n'.code){
			multilineStringIndentBuffer = new StringBuf(); //ignores whitespace lines
		}else{
			var indentAsItWas = multilineStringIndentBuffer.toString();
			multilineStringIndentBuffer = new StringBuf();
			//assert(indentAsItWas.length < multilineStringsIndent.length)
			if(prefixes(indentAsItWas, multilineStringsIndent)){
				//breaking out, transfer control to eatingIndentation
				finishTakingSymbolAndAttach();
				stringBuffer = new StringBuf();
				stringBuffer.addSub(indentAsItWas,0);
				transition(eatingIndentation);
				eatingIndentation(false,c);
			}else{
				giveUp("inconsistent indentation");
			}
		}
	}
	private function multiLineTakingText(fileEnd:Bool, c:Int){
		if(fileEnd){
			finishTakingSymbolAndAttach();
			finishTakingIndentationAndAdjustLineAttachment();
		}else switch(c){
			case '\n'.code :
				// stringBuffer.addChar('\n'.code   will not add the newline until we're sure the multiline string is continuing
				transition(multiLineTakingIndent);
			case c :
				stringBuffer.addChar(c);
		}
	}
	
	
	public function parseToSeqs(s:BufferedIterator<Int>):Seqs {
		init();
		//pump characters into the mode of the parser until the read head has been graduated to the end
		transition(eatingIndentation);
		
		while(s.hasNext()){
			var c = s.next();
			if(c == '\r'.code){ //handle windows' deviant line endings
				c = '\n'.code;
				if(s.hasNext() && s.peek() == '\n'.code){ //if the \r has a \n following it, don't register that
					s.next();
				}
			}
			currentMode(false,c);
			index += 1;
			if(c == '\n'.code){
				line += 1;
				column = 0;
			}else{
				column += 1;
			}
		}
		currentMode(true,'☠'.code);
		var res = new Seqs(mapArToVect(rootArBuf, function(pit){return pit.t.toTerm();}),0,0);
		
		return res;
	}
	public function parseStringToSeqs(s:String):Seqs { return parseToSeqs(new StringIterator(s)); }
}