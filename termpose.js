(function (console, $hx_exports) { "use strict";
function $extend(from, fields) {
	function Inherit() {} Inherit.prototype = from; var proto = new Inherit();
	for (var name in fields) proto[name] = fields[name];
	if( fields.toString !== Object.prototype.toString ) proto.toString = fields.toString;
	return proto;
}
var HxOverrides = function() { };
HxOverrides.__name__ = true;
HxOverrides.cca = function(s,index) {
	var x = s.charCodeAt(index);
	if(x != x) return undefined;
	return x;
};
HxOverrides.substr = function(s,pos,len) {
	if(pos != null && pos != 0 && len != null && len < 0) return "";
	if(len == null) len = s.length;
	if(pos < 0) {
		pos = s.length + pos;
		if(pos < 0) pos = 0;
	} else if(len < 0) len = s.length + len - pos;
	return s.substr(pos,len);
};
Math.__name__ = true;
var Std = function() { };
Std.__name__ = true;
Std.string = function(s) {
	return js_Boot.__string_rec(s,"");
};
var StringBuf = function() {
	this.b = "";
};
StringBuf.__name__ = true;
StringBuf.prototype = {
	addSub: function(s,pos,len) {
		if(len == null) this.b += HxOverrides.substr(s,pos,null); else this.b += HxOverrides.substr(s,pos,len);
	}
	,__class__: StringBuf
};
var Stringificable = function() { };
Stringificable.__name__ = true;
Stringificable.prototype = {
	stringify: function(sb) {
	}
	,toString: function() {
		var buf = new StringBuf();
		this.stringify(buf);
		return buf.b;
	}
	,__class__: Stringificable
};
var BufferedIterator = $hx_exports.BufferedIterator = function() { };
BufferedIterator.__name__ = true;
BufferedIterator.prototype = {
	__class__: BufferedIterator
};
var BufferedIteratorFromIterator = function(i) {
	this.i = i;
	this.cycle();
};
BufferedIteratorFromIterator.__name__ = true;
BufferedIteratorFromIterator.__interfaces__ = [BufferedIterator];
BufferedIteratorFromIterator.prototype = {
	cycle: function() {
		this.hasValue = this.i.hasNext();
		if(this.hasValue) this.value = this.i.next(); else this.value = null;
	}
	,peek: function() {
		return this.value;
	}
	,next: function() {
		var ret = this.value;
		this.cycle();
		return ret;
	}
	,hasNext: function() {
		return this.hasValue;
	}
	,__class__: BufferedIteratorFromIterator
};
var StringIterator = function(v) {
	this.s = v;
	this.i = 0;
};
StringIterator.__name__ = true;
StringIterator.__interfaces__ = [BufferedIterator];
StringIterator.prototype = {
	hasNext: function() {
		return this.i < this.s.length;
	}
	,next: function() {
		var ret = HxOverrides.cca(this.s,this.i);
		this.i = this.i + 1;
		return ret;
	}
	,peek: function() {
		if(this.i < this.s.length) return HxOverrides.cca(this.s,this.i); else return null;
	}
	,__class__: StringIterator
};
var Term = $hx_exports.Term = function() { };
Term.__name__ = true;
Term.__super__ = Stringificable;
Term.prototype = $extend(Stringificable.prototype,{
	jsonString: function() {
		var sb = new StringBuf();
		this.buildJsonString(sb);
		return sb.b;
	}
	,buildJsonString: function(sb) {
	}
	,prettyPrint: function() {
		var sb = new StringBuf();
		this.buildPrettyPrint(sb,2,true,80);
		return sb.b;
	}
	,buildPrettyPrint: function(sb,numberOfSpacesElseTab,windowsLineEndings,maxLineWidth) {
		var indent;
		if(numberOfSpacesElseTab != null) indent = Parser.repeatString(" ",numberOfSpacesElseTab); else indent = "\t";
		var lineEndings;
		if(windowsLineEndings) lineEndings = "\r\n"; else lineEndings = "\n";
		this.buildPrettyPrinting(sb,0,indent,lineEndings,maxLineWidth);
	}
	,buildPrettyPrinting: function(sb,depth,indent,lineEndings,lineWidth) {
		if(this.v != null) {
			var _g = 0;
			while(_g < depth) {
				var i = _g++;
				sb.b += HxOverrides.substr(indent,0,null);
			}
			this.stringify(sb);
			sb.b += HxOverrides.substr(lineEndings,0,null);
		} else {
			var ts = this.s;
			var _g1 = 0;
			while(_g1 < depth) {
				var i1 = _g1++;
				sb.b += HxOverrides.substr(indent,0,null);
			}
			if(ts.length == 0) {
				sb.b += ":";
				sb.b += HxOverrides.substr(lineEndings,0,null);
			} else if(this.estimateLength() > lineWidth) {
				if(ts[0].estimateLength() > lineWidth) {
					sb.b += ":";
					sb.b += HxOverrides.substr(lineEndings,0,null);
					var _g2 = 0;
					while(_g2 < ts.length) {
						var t = ts[_g2];
						++_g2;
						t.buildPrettyPrinting(sb,depth + 1,indent,lineEndings,lineWidth);
					}
				} else {
					ts[0].baseLinePrettyStringify(sb);
					sb.b += HxOverrides.substr(lineEndings,0,null);
					var _g11 = 0;
					var _g3 = ts.length;
					while(_g11 < _g3) {
						var i2 = _g11++;
						var e = ts[i2];
						e.buildPrettyPrinting(sb,depth + 1,indent,lineEndings,lineWidth);
					}
				}
			} else {
				this.baseLinePrettyStringify(sb);
				sb.b += HxOverrides.substr(lineEndings,0,null);
			}
		}
	}
	,sexpStringify: function(sb) {
		this.stringify(sb);
	}
	,prettyStringify: function(sb) {
	}
	,baseLineStringify: function(sb) {
	}
	,baseLinePrettyStringify: function(sb) {
	}
	,estimateLength: function() {
		if(this.s != null) {
			var sum = 0;
			var _g = 0;
			var _g1 = this.s;
			while(_g < _g1.length) {
				var t = _g1[_g];
				++_g;
				sum += t.estimateLength() + 1;
			}
			return 2 + sum - 1;
		} else return this.v.length + 2;
	}
	,__class__: Term
});
var Stri = $hx_exports.Stri = function(vp,line,column) {
	this.s = null;
	this.v = vp;
	this.line = line;
	this.column = column;
};
Stri.__name__ = true;
Stri.__super__ = Term;
Stri.prototype = $extend(Term.prototype,{
	stringify: function(sb) {
		if(Parser.escapeIsNeeded(this.v)) {
			sb.b += "\"";
			Parser.escapeSymbol(sb,this.v);
			sb.b += "\"";
		} else sb.addSub(this.v,0,null);
	}
	,prettyStringify: function(sb) {
		this.stringify(sb);
	}
	,baseLineStringify: function(sb) {
		this.stringify(sb);
	}
	,buildJsonString: function(sb) {
		sb.b += "\"";
		if(Parser.escapeIsNeeded(this.v)) Parser.escapeSymbol(sb,this.v); else sb.addSub(this.v,0,null);
		sb.b += "\"";
	}
	,__class__: Stri
});
var Seqs = $hx_exports.Seqs = function(s,line,column) {
	this.s = s;
	this.v = null;
	this.line = line;
	this.column = column;
};
Seqs.__name__ = true;
Seqs.__super__ = Term;
Seqs.prototype = $extend(Term.prototype,{
	stringify: function(sbuf) {
		sbuf.b += "(";
		this.baseLineStringify(sbuf);
		sbuf.b += ")";
	}
	,baseLineStringify: function(sbuf) {
		if(this.s.length >= 1) {
			this.s[0].stringify(sbuf);
			var _g1 = 1;
			var _g = this.s.length;
			while(_g1 < _g) {
				var i = _g1++;
				sbuf.b += " ";
				this.s[i].stringify(sbuf);
			}
		}
	}
	,baseLinePrettyStringify: function(sbuf) {
		if(this.s.length >= 1) {
			this.s[0].prettyStringify(sbuf);
			var _g1 = 1;
			var _g = this.s.length;
			while(_g1 < _g) {
				var i = _g1++;
				sbuf.b += " ";
				this.s[i].prettyStringify(sbuf);
			}
		}
	}
	,prettyStringify: function(sb) {
		if(this.s.length == 0) sb.b += ":"; else if(this.s.length == 2) {
			this.s[0].prettyStringify(sb);
			sb.b += ":";
			this.s[1].prettyStringify(sb);
		} else {
			var starts;
			if(this.s[0].v != null) {
				this.s[0].stringify(sb);
				sb.b += "(";
				if(this.s.length >= 1) this.s[1].prettyStringify(sb);
				starts = 2;
			} else {
				sb.b += "(";
				this.s[0].prettyStringify(sb);
				starts = 1;
			}
			var _g1 = starts;
			var _g = this.s.length;
			while(_g1 < _g) {
				var i = _g1++;
				sb.b += " ";
				this.s[i].prettyStringify(sb);
			}
			sb.b += ")";
		}
	}
	,buildJsonString: function(sbuf) {
		sbuf.b += "[";
		if(this.s.length > 0) this.s[0].buildJsonString(sbuf);
		var _g1 = 1;
		var _g = this.s.length;
		while(_g1 < _g) {
			var i = _g1++;
			sbuf.b += ",";
			this.s[i].buildJsonString(sbuf);
		}
		sbuf.b += "]";
	}
	,__class__: Seqs
});
var ParsingException = $hx_exports.ParsingException = function(s) {
	this.msg = s;
};
ParsingException.__name__ = true;
ParsingException.prototype = {
	__class__: ParsingException
};
var _$Termpose_InterTerm = function() { };
_$Termpose_InterTerm.__name__ = true;
_$Termpose_InterTerm.prototype = {
	__class__: _$Termpose_InterTerm
};
var _$Termpose_PointsInterTerm = function(t) {
	this.t = t;
};
_$Termpose_PointsInterTerm.__name__ = true;
_$Termpose_PointsInterTerm.prototype = {
	__class__: _$Termpose_PointsInterTerm
};
var _$Termpose_Sq = function(s,line,column,pointing) {
	this.s = s;
	this.line = line;
	this.column = column;
	pointing.t = this;
};
_$Termpose_Sq.__name__ = true;
_$Termpose_Sq.__interfaces__ = [_$Termpose_InterTerm];
_$Termpose_Sq.prototype = {
	toTerm: function() {
		return this.toSeqs();
	}
	,toSeqs: function() {
		return new Seqs(Parser.mapArToVect(this.s,function(e) {
			return e.t.toTerm();
		}),this.line,this.column);
	}
	,__class__: _$Termpose_Sq
};
var _$Termpose_St = function(sy,line,column,pointing) {
	this.sy = sy;
	this.line = line;
	this.column = column;
	pointing.t = this;
};
_$Termpose_St.__name__ = true;
_$Termpose_St.__interfaces__ = [_$Termpose_InterTerm];
_$Termpose_St.prototype = {
	toTerm: function() {
		return new Stri(this.sy,this.line,this.column);
	}
	,__class__: _$Termpose_St
};
var Parser = $hx_exports.Parser = function() {
};
Parser.__name__ = true;
Parser.prefixes = function(shorter,longer) {
	if(shorter.length > longer.length) return false; else {
		var _g1 = 0;
		var _g = shorter.length;
		while(_g1 < _g) {
			var i = _g1++;
			if(HxOverrides.cca(shorter,i) != HxOverrides.cca(longer,i)) return false;
		}
		return true;
	}
};
Parser.repeatString = function(str,n) {
	var sb_b = "";
	var _g = 0;
	while(_g < n) {
		var i = _g++;
		sb_b += HxOverrides.substr(str,0,null);
	}
	return sb_b;
};
Parser.escapeIsNeeded = function(sy) {
	var _g1 = 0;
	var _g = sy.length;
	while(_g1 < _g) {
		var i = _g1++;
		var c = HxOverrides.cca(sy,i);
		if(c == 32 || c == 40 || c == 58 || c == 9 || c == 10 || c == 13 || c == 41) return true;
	}
	return false;
};
Parser.mapArToVect = function(ar,f) {
	var v;
	var this1;
	this1 = new Array(ar.length);
	v = this1;
	var _g1 = 0;
	var _g = ar.length;
	while(_g1 < _g) {
		var i = _g1++;
		var val = f(ar[i]);
		v[i] = val;
	}
	return v;
};
Parser.escapeSymbol = function(sb,str) {
	var _g1 = 0;
	var _g = str.length;
	while(_g1 < _g) {
		var i = _g1++;
		var _g2 = HxOverrides.cca(str,i);
		var c = _g2;
		if(_g2 != null) switch(_g2) {
		case 92:
			sb.b += "\\";
			sb.b += "\\";
			break;
		case 34:
			sb.b += "\\";
			sb.b += "\"";
			break;
		case 10:
			sb.b += "\\";
			sb.b += "n";
			break;
		case 13:
			sb.b += "\\";
			sb.b += "r";
			break;
		default:
			sb.b += String.fromCharCode(c);
		} else sb.b += String.fromCharCode(c);
	}
};
Parser.array = function(el) {
	var ret = [];
	ret.push(el);
	return ret;
};
Parser.dropSeqLayerIfSole = function(pt) {
	var arb;
	arb = (js_Boot.__cast(pt.t , _$Termpose_Sq)).s;
	if(arb.length == 1) {
		var soleEl = arb[0].t;
		pt.t = soleEl;
	}
};
Parser.prototype = {
	growSeqsLayer: function(pi) {
		var oldEl = pi.t;
		var newPIT = new _$Termpose_PointsInterTerm(oldEl);
		var newSq = new _$Termpose_Sq(Parser.array(newPIT),this.line,this.column,pi);
		return newSq;
	}
	,interSq: function(ar) {
		if(ar == null) ar = [];
		var pt = new _$Termpose_PointsInterTerm(null);
		new _$Termpose_Sq(ar,this.line,this.column,pt);
		return pt;
	}
	,emptyInterSq: function() {
		return this.interSq([]);
	}
	,interSt: function(sy) {
		var pt = new _$Termpose_PointsInterTerm(null);
		new _$Termpose_St(sy,this.line,this.column,pt);
		return pt;
	}
	,init: function() {
		this.stringBuffer = new StringBuf();
		this.previousIndentation = "";
		this.salientIndentation = "";
		this.hasFoundLine = false;
		this.parenTermStack = [];
		this.line = 0;
		this.column = 0;
		this.index = 0;
		this.currentMode = null;
		this.lastAttachedTerm = null;
		this.modes = [];
		this.rootArBuf = [];
		this.indentStack = Parser.array({ depth : 0, contents : this.rootArBuf});
		this.multilineStringIndentBuffer = new StringBuf();
		this.multilineStringsIndent = null;
	}
	,transition: function(nm) {
		this.currentMode = nm;
	}
	,pushMode: function(nm) {
		this.modes.push(this.currentMode);
		this.transition(nm);
	}
	,popMode: function() {
		this.currentMode = this.modes.pop();
	}
	,giveUp: function(message) {
		this.breakAt(this.line,this.column,message);
	}
	,breakAt: function(l,c,message) {
		throw new js__$Boot_HaxeError(new ParsingException("line:" + l + " column:" + c + ", no, bad: " + message));
	}
	,receiveForemostRecepticle: function() {
		if(this.containsImmediateNext != null) {
			var ret = this.containsImmediateNext.s;
			this.containsImmediateNext = null;
			return ret;
		} else {
			if(this.parenTermStack.length == 0) {
				var rootParenLevel = this.emptyInterSq();
				this.parenTermStack.push(rootParenLevel);
				this.indentStack[this.indentStack.length - 1].contents.push(rootParenLevel);
			}
			return (js_Boot.__cast(this.parenTermStack[this.parenTermStack.length - 1].t , _$Termpose_Sq)).s;
		}
	}
	,attach: function(t) {
		this.receiveForemostRecepticle().push(t);
		this.lastAttachedTerm = t;
	}
	,finishTakingSymbolAndAttach: function() {
		var newSt = this.interSt(this.stringBuffer.b);
		this.attach(newSt);
		this.stringBuffer = new StringBuf();
		return newSt;
	}
	,finishTakingIndentationAndAdjustLineAttachment: function() {
		this.previousIndentation = this.salientIndentation;
		this.salientIndentation = this.stringBuffer.b;
		this.stringBuffer = new StringBuf();
		if(this.hasFoundLine) {
			if(this.salientIndentation.length > this.previousIndentation.length) {
				if(!Parser.prefixes(this.previousIndentation,this.salientIndentation)) this.breakAt(this.line - this.salientIndentation.length,this.column,"inconsistent indentation at");
				if(this.parenTermStack.length > 1 || this.containsImmediateNext != null) Parser.dropSeqLayerIfSole(this.parenTermStack[0]);
				this.indentStack.push({ depth : this.salientIndentation.length, contents : this.receiveForemostRecepticle()});
			} else {
				Parser.dropSeqLayerIfSole(this.parenTermStack[0]);
				this.containsImmediateNext = null;
				if(this.salientIndentation.length < this.previousIndentation.length) {
					if(!Parser.prefixes(this.salientIndentation,this.previousIndentation)) this.breakAt(this.line,this.column,"inconsistent indentation");
					while(this.indentStack[this.indentStack.length - 1].depth > this.salientIndentation.length) {
						if(this.indentStack[this.indentStack.length - 1].depth < this.salientIndentation.length) this.breakAt(this.line,this.column,"inconsistent indentation, sibling elements have different indentation");
						this.indentStack.pop();
					}
				}
			}
			this.parenTermStack = [];
		} else this.hasFoundLine = true;
	}
	,closeParen: function() {
		if(this.parenTermStack.length <= 1) this.giveUp("unbalanced paren");
		this.containsImmediateNext = null;
		this.parenTermStack.pop();
	}
	,receiveFinishedSymbol: function() {
		var ret = this.interSt(this.stringBuffer.b);
		this.stringBuffer = new StringBuf();
		return ret;
	}
	,eatingIndentation: function(fileEnd,c) {
		if(fileEnd) {
			this.stringBuffer = new StringBuf();
			this.finishTakingIndentationAndAdjustLineAttachment();
		} else {
			var c1 = c;
			switch(c) {
			case 10:
				this.stringBuffer = new StringBuf();
				break;
			case 58:case 40:
				this.finishTakingIndentationAndAdjustLineAttachment();
				this.transition($bind(this,this.seekingTerm));
				this.seekingTerm(false,c);
				break;
			case 41:
				this.giveUp("nothing to close");
				break;
			case 34:
				this.finishTakingIndentationAndAdjustLineAttachment();
				this.transition($bind(this,this.buildingQuotedSymbol));
				break;
			case 32:case 9:
				this.stringBuffer.b += String.fromCharCode(c);
				break;
			default:
				this.finishTakingIndentationAndAdjustLineAttachment();
				this.transition($bind(this,this.buildingSymbol));
				this.buildingSymbol(false,c1);
			}
		}
	}
	,seekingTerm: function(fileEnd,c) {
		if(fileEnd) this.finishTakingIndentationAndAdjustLineAttachment(); else {
			var c1 = c;
			switch(c) {
			case 40:
				var newSq = this.emptyInterSq();
				this.attach(newSq);
				this.parenTermStack.push(newSq);
				break;
			case 41:
				this.closeParen();
				this.transition($bind(this,this.immediatelyAfterTerm));
				break;
			case 58:
				var newSq1 = this.emptyInterSq();
				this.attach(newSq1);
				this.containsImmediateNext = js_Boot.__cast(newSq1.t , _$Termpose_Sq);
				break;
			case 10:
				this.transition($bind(this,this.eatingIndentation));
				break;
			case 32:case 9:
				break;
			case 34:
				this.transition($bind(this,this.buildingQuotedSymbol));
				break;
			default:
				this.transition($bind(this,this.buildingSymbol));
				this.buildingSymbol(false,c1);
			}
		}
	}
	,immediatelyAfterTerm: function(fileEnd,c) {
		if(fileEnd) this.finishTakingIndentationAndAdjustLineAttachment(); else {
			var c1 = c;
			switch(c) {
			case 40:
				var newLevel = this.lastAttachedTerm;
				this.growSeqsLayer(newLevel);
				this.parenTermStack.push(newLevel);
				this.transition($bind(this,this.seekingTerm));
				break;
			case 41:
				this.closeParen();
				break;
			case 58:
				this.containsImmediateNext = this.growSeqsLayer(this.lastAttachedTerm);
				this.transition($bind(this,this.seekingTerm));
				break;
			case 10:
				this.transition($bind(this,this.eatingIndentation));
				break;
			case 32:case 9:
				this.transition($bind(this,this.seekingTerm));
				break;
			case 34:
				this.containsImmediateNext = this.growSeqsLayer(this.lastAttachedTerm);
				this.transition($bind(this,this.buildingQuotedSymbol));
				break;
			default:
				this.giveUp("You have to put a space here. Yes I know the fact that I can say that means I could just pretend there's a space there and let you go ahead, but I wont be doing that, as I am an incorrigible formatting nazi.");
			}
		}
	}
	,buildingSymbol: function(fileEnd,c) {
		if(fileEnd) {
			this.finishTakingSymbolAndAttach();
			this.finishTakingIndentationAndAdjustLineAttachment();
		} else {
			var c1 = c;
			switch(c) {
			case 32:case 9:
				this.finishTakingSymbolAndAttach();
				this.transition($bind(this,this.seekingTerm));
				break;
			case 58:case 10:case 40:case 41:
				this.finishTakingSymbolAndAttach();
				this.transition($bind(this,this.immediatelyAfterTerm));
				this.immediatelyAfterTerm(false,c);
				break;
			case 34:
				this.finishTakingSymbolAndAttach();
				this.transition($bind(this,this.immediatelyAfterTerm));
				this.immediatelyAfterTerm(false,c);
				break;
			default:
				this.stringBuffer.b += String.fromCharCode(c1);
			}
		}
	}
	,buildingQuotedSymbol: function(fileEnd,c) {
		if(fileEnd) {
			this.finishTakingSymbolAndAttach();
			this.finishTakingIndentationAndAdjustLineAttachment();
		} else {
			var c1 = c;
			switch(c) {
			case 34:
				this.finishTakingSymbolAndAttach();
				this.transition($bind(this,this.immediatelyAfterTerm));
				break;
			case 92:
				this.pushMode($bind(this,this.takingEscape));
				break;
			case 10:
				if(this.stringBuffer.b.length == 0) this.transition($bind(this,this.multiLineFirstLine)); else {
					this.finishTakingSymbolAndAttach();
					this.transition($bind(this,this.eatingIndentation));
				}
				break;
			default:
				this.stringBuffer.b += String.fromCharCode(c1);
			}
		}
	}
	,takingEscape: function(fileEnd,c) {
		if(fileEnd) this.giveUp("invalid escape sequence, no one can escape the end of the file"); else {
			var g = c;
			switch(c) {
			case 104:
				this.stringBuffer.b += String.fromCharCode(9731);
				break;
			case 110:
				this.stringBuffer.b += "\n";
				break;
			case 114:
				this.stringBuffer.b += "\r";
				break;
			case 116:
				this.stringBuffer.b += "\t";
				break;
			default:
				this.stringBuffer.b += String.fromCharCode(g);
			}
			this.popMode();
		}
	}
	,multiLineFirstLine: function(fileEnd,c) {
		if(fileEnd) {
			this.finishTakingSymbolAndAttach();
			this.finishTakingIndentationAndAdjustLineAttachment();
		} else {
			var c1 = c;
			switch(c) {
			case 32:case 9:
				this.multilineStringIndentBuffer.b += String.fromCharCode(c);
				break;
			default:
				this.multilineStringsIndent = this.multilineStringIndentBuffer.b;
				if(this.multilineStringsIndent.length > this.salientIndentation.length) {
					if(Parser.prefixes(this.salientIndentation,this.multilineStringsIndent)) {
						this.transition($bind(this,this.multiLineTakingText));
						this.multiLineTakingText(false,c1);
					} else this.giveUp("inconsistent indentation");
				} else {
					this.finishTakingSymbolAndAttach();
					this.stringBuffer = new StringBuf();
					this.stringBuffer.addSub(this.multilineStringsIndent,0,null);
					this.multilineStringsIndent = null;
					this.transition($bind(this,this.eatingIndentation));
					this.eatingIndentation(false,c1);
				}
				this.multilineStringIndentBuffer = new StringBuf();
			}
		}
	}
	,multiLineTakingIndent: function(fileEnd,c) {
		if(fileEnd) {
			this.finishTakingSymbolAndAttach();
			this.finishTakingIndentationAndAdjustLineAttachment();
		} else if(c == 32 || c == 9) {
			this.multilineStringIndentBuffer.b += String.fromCharCode(c);
			if(this.multilineStringIndentBuffer.b.length == this.multilineStringsIndent.length) {
				if(Parser.prefixes(this.multilineStringsIndent,this.multilineStringsIndent)) {
					this.stringBuffer.b += "\n";
					this.transition($bind(this,this.multiLineTakingText));
				} else this.giveUp("inconsistent indentation");
				this.multilineStringIndentBuffer = new StringBuf();
			}
		} else if(c == 10) this.multilineStringIndentBuffer = new StringBuf(); else {
			var indentAsItWas = this.multilineStringIndentBuffer.b;
			this.multilineStringIndentBuffer = new StringBuf();
			if(Parser.prefixes(indentAsItWas,this.multilineStringsIndent)) {
				this.finishTakingSymbolAndAttach();
				this.stringBuffer = new StringBuf();
				this.stringBuffer.b += HxOverrides.substr(indentAsItWas,0,null);
				this.transition($bind(this,this.eatingIndentation));
				this.eatingIndentation(false,c);
			} else this.giveUp("inconsistent indentation");
		}
	}
	,multiLineTakingText: function(fileEnd,c) {
		if(fileEnd) {
			this.finishTakingSymbolAndAttach();
			this.finishTakingIndentationAndAdjustLineAttachment();
		} else {
			var c1 = c;
			switch(c) {
			case 10:
				this.transition($bind(this,this.multiLineTakingIndent));
				break;
			default:
				this.stringBuffer.b += String.fromCharCode(c1);
			}
		}
	}
	,parseToSeqs: function(s) {
		this.init();
		this.transition($bind(this,this.eatingIndentation));
		while(s.hasNext()) {
			var c = s.next();
			if(c == 13) {
				c = 10;
				if(s.hasNext() && s.peek() == 10) s.next();
			}
			this.currentMode(false,c);
			this.index += 1;
			if(c == 10) {
				this.line += 1;
				this.column = 0;
			} else this.column += 1;
		}
		this.currentMode(true,9760);
		var res = new Seqs(Parser.mapArToVect(this.rootArBuf,function(pit) {
			return pit.t.toTerm();
		}),0,0);
		return res;
	}
	,parseStringToSeqs: function(s) {
		return this.parseToSeqs(new StringIterator(s));
	}
	,__class__: Parser
};
var Termpose = $hx_exports.Termpose = function() { };
Termpose.__name__ = true;
Termpose.parseMultiline = function(s) {
	return new Parser().parseToSeqs(new StringIterator(s));
};
Termpose.parse = function(s) {
	var res = Termpose.parseMultiline(s);
	var ress = res.s;
	if(ress.length == 1) return ress[0]; else return res;
};
var js__$Boot_HaxeError = function(val) {
	Error.call(this);
	this.val = val;
	this.message = String(val);
	if(Error.captureStackTrace) Error.captureStackTrace(this,js__$Boot_HaxeError);
};
js__$Boot_HaxeError.__name__ = true;
js__$Boot_HaxeError.__super__ = Error;
js__$Boot_HaxeError.prototype = $extend(Error.prototype,{
	__class__: js__$Boot_HaxeError
});
var js_Boot = function() { };
js_Boot.__name__ = true;
js_Boot.getClass = function(o) {
	if((o instanceof Array) && o.__enum__ == null) return Array; else {
		var cl = o.__class__;
		if(cl != null) return cl;
		var name = js_Boot.__nativeClassName(o);
		if(name != null) return js_Boot.__resolveNativeClass(name);
		return null;
	}
};
js_Boot.__string_rec = function(o,s) {
	if(o == null) return "null";
	if(s.length >= 5) return "<...>";
	var t = typeof(o);
	if(t == "function" && (o.__name__ || o.__ename__)) t = "object";
	switch(t) {
	case "object":
		if(o instanceof Array) {
			if(o.__enum__) {
				if(o.length == 2) return o[0];
				var str2 = o[0] + "(";
				s += "\t";
				var _g1 = 2;
				var _g = o.length;
				while(_g1 < _g) {
					var i1 = _g1++;
					if(i1 != 2) str2 += "," + js_Boot.__string_rec(o[i1],s); else str2 += js_Boot.__string_rec(o[i1],s);
				}
				return str2 + ")";
			}
			var l = o.length;
			var i;
			var str1 = "[";
			s += "\t";
			var _g2 = 0;
			while(_g2 < l) {
				var i2 = _g2++;
				str1 += (i2 > 0?",":"") + js_Boot.__string_rec(o[i2],s);
			}
			str1 += "]";
			return str1;
		}
		var tostr;
		try {
			tostr = o.toString;
		} catch( e ) {
			if (e instanceof js__$Boot_HaxeError) e = e.val;
			return "???";
		}
		if(tostr != null && tostr != Object.toString && typeof(tostr) == "function") {
			var s2 = o.toString();
			if(s2 != "[object Object]") return s2;
		}
		var k = null;
		var str = "{\n";
		s += "\t";
		var hasp = o.hasOwnProperty != null;
		for( var k in o ) {
		if(hasp && !o.hasOwnProperty(k)) {
			continue;
		}
		if(k == "prototype" || k == "__class__" || k == "__super__" || k == "__interfaces__" || k == "__properties__") {
			continue;
		}
		if(str.length != 2) str += ", \n";
		str += s + k + " : " + js_Boot.__string_rec(o[k],s);
		}
		s = s.substring(1);
		str += "\n" + s + "}";
		return str;
	case "function":
		return "<function>";
	case "string":
		return o;
	default:
		return String(o);
	}
};
js_Boot.__interfLoop = function(cc,cl) {
	if(cc == null) return false;
	if(cc == cl) return true;
	var intf = cc.__interfaces__;
	if(intf != null) {
		var _g1 = 0;
		var _g = intf.length;
		while(_g1 < _g) {
			var i = _g1++;
			var i1 = intf[i];
			if(i1 == cl || js_Boot.__interfLoop(i1,cl)) return true;
		}
	}
	return js_Boot.__interfLoop(cc.__super__,cl);
};
js_Boot.__instanceof = function(o,cl) {
	if(cl == null) return false;
	switch(cl) {
	case Int:
		return (o|0) === o;
	case Float:
		return typeof(o) == "number";
	case Bool:
		return typeof(o) == "boolean";
	case String:
		return typeof(o) == "string";
	case Array:
		return (o instanceof Array) && o.__enum__ == null;
	case Dynamic:
		return true;
	default:
		if(o != null) {
			if(typeof(cl) == "function") {
				if(o instanceof cl) return true;
				if(js_Boot.__interfLoop(js_Boot.getClass(o),cl)) return true;
			} else if(typeof(cl) == "object" && js_Boot.__isNativeObj(cl)) {
				if(o instanceof cl) return true;
			}
		} else return false;
		if(cl == Class && o.__name__ != null) return true;
		if(cl == Enum && o.__ename__ != null) return true;
		return o.__enum__ == cl;
	}
};
js_Boot.__cast = function(o,t) {
	if(js_Boot.__instanceof(o,t)) return o; else throw new js__$Boot_HaxeError("Cannot cast " + Std.string(o) + " to " + Std.string(t));
};
js_Boot.__nativeClassName = function(o) {
	var name = js_Boot.__toStr.call(o).slice(8,-1);
	if(name == "Object" || name == "Function" || name == "Math" || name == "JSON") return null;
	return name;
};
js_Boot.__isNativeObj = function(o) {
	return js_Boot.__nativeClassName(o) != null;
};
js_Boot.__resolveNativeClass = function(name) {
	return (Function("return typeof " + name + " != \"undefined\" ? " + name + " : null"))();
};
var $_, $fid = 0;
function $bind(o,m) { if( m == null ) return null; if( m.__id__ == null ) m.__id__ = $fid++; var f; if( o.hx__closures__ == null ) o.hx__closures__ = {}; else f = o.hx__closures__[m.__id__]; if( f == null ) { f = function(){ return f.method.apply(f.scope, arguments); }; f.scope = o; f.method = m; o.hx__closures__[m.__id__] = f; } return f; }
String.prototype.__class__ = String;
String.__name__ = true;
Array.__name__ = true;
var Int = { __name__ : ["Int"]};
var Dynamic = { __name__ : ["Dynamic"]};
var Float = Number;
Float.__name__ = ["Float"];
var Bool = Boolean;
Bool.__ename__ = ["Bool"];
var Class = { __name__ : ["Class"]};
var Enum = { };
js_Boot.__toStr = {}.toString;
})(typeof console != "undefined" ? console : {log:function(){}}, typeof window != "undefined" ? window : exports);
