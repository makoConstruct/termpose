import haxe.ds.Vector;

class Stringificable{ //has a method for stringifying through a StringBuf, provides a toString through this
	public function stringify(sb:StringBuf){}
	public function toString():String {
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
	public var line (default, null):Int;
	public var column (default, null):Int;
	public function jsonString():String{
		var sb = new StringBuf();
		buildJsonString(sb);
		return sb.toString();
	}
	public function s():Null<Vector<Term>> return null;
	public function v():Null<String> return null;
	public function buildJsonString(sb:StringBuf){}
	public function prettyPrint():String{
		var sb = new StringBuf();
		buildPrettyPrint(sb, 2, true, 80);
		return sb.toString();
	}
	public function buildPrettyPrint(sb:StringBuf, numberOfSpacesElseTab:Null<Int>, windowsLineEndings:Bool, maxLineWidth:Int){
		var indent:String = if(numberOfSpacesElseTab != null){ Parser.repeatString(" ",numberOfSpacesElseTab); }else{ "\t"; };
		var lineEndings:String = if(windowsLineEndings){ "\r\n"; }else{ "\n"; };
		buildPrettyPrinting(sb, 0, indent, lineEndings, maxLineWidth);
	}
	private function buildPrettyPrinting(sb:StringBuf, depth:Int, indent:String, lineEndings:String, lineWidth:Int){
		if(this.v() != null){
			for(i in 0...depth){ sb.addSub(indent,0); }
			stringify(sb);
			sb.addSub(lineEndings,0);
		}else{
			var ts:Vector<Term> = this.s();
			for(i in 0...depth){ sb.addSub(indent,0); }
			if(ts.length == 0){
				sb.addChar(':'.code);
				sb.addSub(lineEndings,0);
			}else{
				if(estimateLength() > lineWidth){
					//print in indent style
					if(ts[0].estimateLength() > lineWidth){
						//indent sequence style
						sb.addChar(':'.code);
						sb.addSub(lineEndings,0);
						for(t in ts){
							t.buildPrettyPrinting(sb, depth+1, indent, lineEndings, lineWidth);
						}
					}else{
						ts[0].baseLinePrettyStringify(sb);
						sb.addSub(lineEndings,0);
						for(i in 0...ts.length){
							var e = ts[i];
							e.buildPrettyPrinting(sb, depth+1, indent, lineEndings, lineWidth);
						}
					}
				}else{
					//print inline style
					baseLinePrettyStringify(sb);
					sb.addSub(lineEndings,0);
				}
			}
		}
	}
	public function sexpStringify(sb:StringBuf){stringify(sb);}
	public function prettyStringify(sb:StringBuf){}
	private function baseLineStringify(sb:StringBuf){}
	private function baseLinePrettyStringify(sb:StringBuf){}
	private function estimateLength():Int{
		if(this.s() != null){
			var sum = 0;
			for(t in this.s()){
				sum += t.estimateLength() + 1;
			}
			return 2 + sum - 1;
		}else{
			return this.v().length + 2;
		}
	}
}

@:expose class Stri extends Term{
	public var vp:String;
	override public function s():Null<Vector<Term>> return null;
	override public function v():Null<String> return vp;
	public function new(vp:String, line:Int, column:Int){
		this.vp = vp;
		this.line = line;
		this.column = column;
	}
	override public function stringify(sb:StringBuf){
		if(Parser.escapeIsNeeded(vp)){
			sb.addChar('"'.code);
			Parser.escapeSymbol(sb, vp);
			sb.addChar('"'.code);
		}else{
			sb.addSub(vp,0);
		}
	}
	override public function prettyStringify(sb:StringBuf){stringify(sb);}
	override private function baseLineStringify(sb:StringBuf){stringify(sb);}
	override public function buildJsonString(sb:StringBuf){
		sb.addChar('"'.code);
		if(Parser.escapeIsNeeded(vp)){
			Parser.escapeSymbol(sb, vp);
		}else{
			sb.addSub(vp,0);
		}
		sb.addChar('"'.code);
	}
}

@:expose class Seqs extends Term{
	public var sp:Vector<Term>;
	override public function s():Null<Vector<Term>> return sp;
	override public function v():Null<String> return null;
	public function new(s:Vector<Term>, line:Int, column:Int){
		this.sp = s;
		this.line = line;
		this.column = column;
	}
	// function name():String{
	// 	if(s.length >= 1){
	// 		switch(s(0)){
	// 			case Stri(v): return v;
	// 			case _: return "";
	// 		}
	// 	}else{ return ""; }
	// }
	override public function stringify(sbuf:StringBuf){
		sbuf.addChar('('.code);
		baseLineStringify(sbuf);
		sbuf.addChar(')'.code);
	}
	override private function baseLineStringify(sbuf:StringBuf){
		if(sp.length >= 1){
			sp.get(0).stringify(sbuf);
			for(i in 1...sp.length){
				sbuf.addChar(' '.code);
				sp.get(i).stringify(sbuf);
			}
		}
	}
	override private function baseLinePrettyStringify(sbuf:StringBuf){
		if(sp.length >= 1){
			sp.get(0).prettyStringify(sbuf);
			for(i in 1...sp.length){
				sbuf.addChar(' '.code);
				sp.get(i).prettyStringify(sbuf);
			}
		}
	}
	override public function prettyStringify(sb:StringBuf){
		if(sp.length == 0){
			sb.addChar(':'.code);
		}else if(sp.length == 2){
			sp[0].prettyStringify(sb);
			sb.addChar(':'.code);
			sp[1].prettyStringify(sb);
		}else{
			var starts:Int;
			if(sp[0].v() != null){
				sp[0].stringify(sb);
				sb.addChar('('.code);
				if(sp.length >= 1){
					sp[1].prettyStringify(sb);
				}
				starts = 2;
			}else{
				sb.addChar('('.code);
				sp[0].prettyStringify(sb);
				starts = 1;
			}
			for(i in starts...sp.length){
				sb.addChar(' '.code);
				sp[i].prettyStringify(sb);
			}
			sb.addChar(')'.code);
		}
	}
	override public function buildJsonString(sbuf:StringBuf){
		sbuf.addChar('['.code);
		if(sp.length > 0){
			sp.get(0).buildJsonString(sbuf);
		}
		for(i in 1...sp.length){
			sbuf.addChar(','.code);
			sp.get(i).buildJsonString(sbuf);
		}
		sbuf.addChar(']'.code);
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
	public function new(){}
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
	public static function repeatString(str:String, n:Int):String{
		var sb = new StringBuf();
		for(i in 0...n){
			sb.addSub(str,0);
		}
		return sb.toString();
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
	
	// for more discussion, see Termpose.scala
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



@:expose class Termpose{
	public static function parseMultiline(s:String):Seqs return new Parser().parseToSeqs(new StringIterator(s));
	public static function parse(s:String):Term {
		var res = parseToSeqs(s);
		var ress = res.s();
		if(ress.length == 1) return ress[0];
		else return res;
	}
}