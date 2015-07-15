import util.{Try, Success, Failure}
import collection.mutable.{ArrayBuffer, HashMap, ArrayStack}
import collection.BufferedIterator
import java.io.{File}
import io.Source
import collection.breakOut


object Termpose{
	trait Stringificable{ //has a method for stringifying through a stringBuilder, provides a toString through this
		def stringify(sb:StringBuilder):Unit
		override def toString ={
			val buf = new StringBuilder
			stringify(buf)
			buf.result
		}
	}

	private def escapeIsNeeded(sy:String) = sy.exists {
		case ' ' => true
		case '(' => true
		case ':' => true
		case '\t' => true
		case '\n' => true
		case '\r' => true
		case ')' => true
		case _:Char => false
	}

	private def escapeSymbol(sb:StringBuilder, str:String){
		for(c <- str){
			c match{
				case '\\' =>
					sb += '\\'
					sb += '\\'
				case '"' =>
					sb += '\\'
					sb += '"'
				case '\n' =>
					sb += '\\'
					sb += 'n'
				case '\r' =>
					sb += '\\'
					sb += 'r'
				case c:Char =>
					sb += c
			}
		}
	}

	sealed trait Term extends Stringificable{
		val line:Int
		val column:Int
		def asMap:Map[String, Seq[Term]]
		def v:String
		def tail:Seq[Term]
		def jsonString:String ={
			val sb = new StringBuilder
			buildJsonString(sb)
			sb.result
		}
		def buildJsonString(sb:StringBuilder):Unit
		def prettyPrinted:String ={
			val sb = new StringBuilder
			buildPrettyPrint(sb, Some(2), true, 80)
			sb.result
		}
		private[Termpose] def estimateLength:Int
		private[Termpose] def prettyPrinting(sb:StringBuilder, depth:Int, indent:String, lineEndings:String, lineWidth:Int):Unit
		private[Termpose] def baseLineStringify(sb:StringBuilder):Unit
		def buildPrettyPrint(sb:StringBuilder, numberOfSpacesElseTab:Option[Int], windowsLineEndings:Boolean, maxLineWidth:Int){
			val indent:String = numberOfSpacesElseTab match{
				case Some(n)=> " "*n
				case None=> "\t"
			}
			val lineEndings:String = if(windowsLineEndings) "\r\n" else "\n"
			prettyPrinting(sb, 0, indent, lineEndings, maxLineWidth)
		}
	}
	object Stri{
		def apply(s:String) = new Stri(s,0,0)
		def unapply(s:Stri) = Some(s.v)
	}
	class Stri(val sy:String, val line:Int, val column:Int) extends Term{
		def asMap:Map[String, Seq[Term]] = Map.empty
		def v:String = sy
		def tail:Seq[Term] = Seq.empty
		def stringify(sb:StringBuilder){
			if(escapeIsNeeded(sy)){
				sb += '"'
				escapeSymbol(sb, v)
				sb += '"'
			}else{
				sb ++= v
			}
		}
		def buildJsonString(sb:StringBuilder){
			sb += '"'
			escapeSymbol(sb, sy)
			sb += '"'
		}
		private[Termpose] def estimateLength:Int = sy.length
		private[Termpose] def prettyPrinting(sb:StringBuilder, depth:Int, indent:String, lineEndings:String, lineWidth:Int){
			for(i <- 0 until depth){ sb ++= indent }
			stringify(sb)
			sb ++= lineEndings
		}
		private[Termpose] def baseLineStringify(sb:StringBuilder){ stringify(sb) }
	}
	object Seqs{
		def apply(terms:Term*) = new Seqs(terms,0,0)
		def from(s:Seq[Term]) = new Seqs(s,0,0)
		def unapply(s:Seqs):Option[Seq[Term]] = Some(s.s)
	}
	class Seqs(val s:Seq[Term], val line:Int, val column:Int) extends Term{
		def v:String = if(s.length >= 1){
			s(0) match{
				case Stri(sy)=> sy
				case _=> ""
			}
		}else{ "" }
		private lazy val _asMap:Map[String, Seq[Term]] = s.map{
			case sexp:Seqs=>
				if(sexp.s.length >= 1){
					sexp.s(0) match{
						case Stri(sy)=> Some((sy, sexp.tail))
						case _=> None
					}
				}else None
			case _=> None
		}.flatten.toMap
		def asMap:Map[String, Seq[Term]] = _asMap
		def tail:Seq[Term] = s.tail
		private[Termpose] def baseLineStringify(sb:StringBuilder){
			if(s.length >= 1){
				val iter = s.iterator
				iter.next.stringify(sb)
				for(e <- iter){
					sb += ' '
					e.stringify(sb)
				}
			}
		}
		private[Termpose]  def prettyPrinting(sb:StringBuilder, depth:Int, indent:String, lineEndings:String, lineWidth:Int){
			for(i <- 0 until depth){ sb ++= indent }
			if(s.length == 0){
				sb += ':'
				sb ++= lineEndings
			}else{
				if(estimateLength > lineWidth){
					//print in indent style
					if(s.head.estimateLength > lineWidth){
						//indent sequence style
						sb += ':'
						sb ++= lineEndings
						for(t <- s){
							t.prettyPrinting(sb, depth+1, indent, lineEndings, lineWidth)
						}
					}else{
						s.head.baseLineStringify(sb)
						sb ++= lineEndings
						for(e <- s.tail){
							e.prettyPrinting(sb, depth+1, indent, lineEndings, lineWidth)
						}
					}
				}else{
					//print inline style
					baseLineStringify(sb)
					sb ++= lineEndings
				}
			}
		}
		def stringify(sb:StringBuilder){
			sb += '('
			baseLineStringify(sb)
			sb += ')'
		}
		def buildJsonString(sb:StringBuilder){
			sb += '['
			val iter = s.iterator
			if(iter.hasNext){
				iter.next.buildJsonString(sb)
			}
			while(iter.hasNext){
				sb += ','
				iter.next.buildJsonString(sb)
			}
			sb += ']'
		}
		private[Termpose]  def estimateLength:Int = 2 + s.map{ 1 + _.estimateLength }.sum - 1
	}

	class ParsingException(msg:String, line:Int, column:Int) extends RuntimeException(msg)

	class Parser{
		private sealed trait InterTerm{
			val line:Int
			val column:Int
			def toTerm:Term
			var pointing:PointsInterTerm
		}
		private def dropSeqLayerIfSole(pt:PointsInterTerm){ //assumes pt is Sq
			val arb = pt.t.asInstanceOf[Sq].s
			if(arb.length == 1){
				val soleEl = arb(0).t
				soleEl.pointing = pt
				pt.t = soleEl
			}
		}
		private def growSeqsLayer(pi:PointsInterTerm):Sq ={
			val oldEl = pi.t
			val newPIT = new PointsInterTerm(oldEl)
			oldEl.pointing = newPIT
			val newSq = new Sq(ArrayBuffer(newPIT), line, column, pi)
			newSq
		}
		private class PointsInterTerm(var t:InterTerm) //sometimes an interterm might want to reattach partway through the process. It should be attached through a PointsInterTerm which it knows so it can do that.
		private def interSq:PointsInterTerm ={
			val pt = new PointsInterTerm(null)
			new Sq(new ArrayBuffer(), line, column, pt)
			pt
		}
		private def interSq(ar:ArrayBuffer[PointsInterTerm]):PointsInterTerm ={
			val pt = new PointsInterTerm(null)
			new Sq(ar, line, column, pt)
			pt
		}
		private def interSt(sy:Symbol):PointsInterTerm ={
			val pt = new PointsInterTerm(null)
			new St(sy, line, column, pt)
			pt
		}
		private class Sq(val s:ArrayBuffer[PointsInterTerm], val line:Int, val column:Int, override var pointing:PointsInterTerm) extends InterTerm{
			pointing.t = this
			def toTerm:Term = toSeqs
			def toSeqs = new Seqs(s.map{ _.t.toTerm }, line, column)
		}
		private class St(val sy:Symbol, val line:Int, val column:Int, override var pointing:PointsInterTerm) extends InterTerm{
			pointing.t = this
			def toTerm:Term = new Stri(sy.name, line, column)
		}
		
		private val stringBuffer = new StringBuilder(512)
		private var previousIndentation:String = ""
		private var salientIndentation:String = ""
		private var hasFoundLine = false
		private var rootArBuf:ArrayBuffer[PointsInterTerm] = new ArrayBuffer()
		private val parenTermStack:ArrayBuffer[PointsInterTerm] = ArrayBuffer()
		private var line = 0
		private var column = 0
		private var index = 0
		private var lastAttachedTerm:PointsInterTerm = null
		private type PF = (Boolean, Char) => Unit
		private var currentMode:PF = null
		private val modes = new ArrayStack[PF]
		private val indentStack = ArrayStack[(Int, ArrayBuffer[PointsInterTerm])]((0, rootArBuf))  //indentation length, tailestTermSequence. It is enough to keep track of length. We can derive ∀(a,b∈Line.Indentation) a.length = b.length → a = b from the prefix checking that is done before adding to the stack
		private val multilineStringIndentBuffer = new StringBuilder
		private var multilineStringsIndent:String = null
		private var containsImmediateNext:Sq = null
		
		private def reinit{
			//=___=
			stringBuffer.clear
			previousIndentation = ""
			salientIndentation = ""
			hasFoundLine = false
			parenTermStack.clear
			line = 0 
			column = 0
			index = 0
			currentMode = null
			lastAttachedTerm = null
			modes.clear
			rootArBuf = new ArrayBuffer()
			indentStack.clear
			indentStack.push((0, rootArBuf))
			multilineStringIndentBuffer.clear
			multilineStringsIndent = null
		}
		
		
		//general notes:
		//the parser consists of a loop that pumps chars from the input string into whatever the current Mode is. Modes jump from one to the next according to cues, sometimes forwarding the cue onto the new mode before it gets any input from the input loop. There is a mode stack, but it is rarely used. Modes are essentially just a Char => Unit (named 'PF', short for Processing Funnel(JJ it's legacy from when they were Partial Functions)). CR LFs ("\r\n"s) are filtered to a single LF (This does mean that a windows formatted file will have unix line endings when parsing multiline strings, that is not a significant issue, ignoring the '\r's will probably save more pain than it causes and the escape sequence \r is available if they're explicitly desired).
		//terms are not attached until they are fully formed. You see, the type and address of a term can change when brackets are appended after it. Say you're reading a(f)(g), you finish reading "a", now you have a symbol. Can you attach it yet? No, because it becomes a Seqs when you find the (, and once you find the ), can you attach the Seqs? No, because it needs to be wrapped in a new seqs when You come to the next (, only once you're sure there're no more parens can you attach the thing.
		//I think.
		//There are other ways I could have handled this, but figuring out which one is best seems like it'd take more effort than just finishing this as it is.
		
		// key aspects of global state to regard:
		// stringBuffer:StringBuilder    where indentation and symbols are collected and cut off
		// indentStack:ArrayBuffer[(Int, ArrayBuffer[InterTerm])]    encodes the levels of indentation we've traversed and the parent container for each level
		// parenStack:ArrayBuffer[ArrayBuffer[InterTerm]]    encodes the levels of parens we've traversed and the container for each level. The first entry is the root line, which has no parens.
		private def transition(nm:PF){ currentMode = nm }
		private def pushMode(nm:PF){
			modes push currentMode
			transition(nm)
		}
		private def popMode{
			currentMode = modes.pop()
		}
		private def break(message:String):Nothing = breakAt(line, column, message)
		private def breakAt(l:Int, c:Int, message:String):Nothing = throw new ParsingException(message,l,c)
		private def receiveForemostRecepticle:ArrayBuffer[PointsInterTerm] ={ //note side effects
			if(containsImmediateNext != null){
				val ret = containsImmediateNext.s
				containsImmediateNext = null
				ret
			}else{
				if(parenTermStack.isEmpty){
					val rootParenLevel = interSq
					parenTermStack += rootParenLevel
					indentStack.top._2 += rootParenLevel
				}
				parenTermStack.last.t.asInstanceOf[Sq].s
			}
		}
		private def attach(t:PointsInterTerm){
			receiveForemostRecepticle += t
			lastAttachedTerm = t
		}
		private def finishTakingSymbolAndAttach:PointsInterTerm ={
			val newSt = interSt(Symbol(stringBuffer.result))
			attach(newSt)
			stringBuffer.clear
			newSt
		}
		private def finishTakingIndentationAndAdjustLineAttachment{
			//Iff there is no indented content or that indented content falls within an inner paren(iff the parenTermStack is longer than one), and the line only has one item at root, the root element in the parenstack should drop a seq layer so that it is just that element. In all other case, leave it as a sequence.
			
			previousIndentation = salientIndentation
			salientIndentation = stringBuffer.result
			stringBuffer.clear
			if(hasFoundLine){
				if(salientIndentation.length > previousIndentation.length){
					if(! salientIndentation.startsWith(previousIndentation)){
						breakAt(line - salientIndentation.length, column, "inconsistent indentation at")
					}
					
					if(parenTermStack.length > 1 || containsImmediateNext != null) dropSeqLayerIfSole(parenTermStack.head) //if antecedents and IsSole, the root element contains the indented stuff
					
					indentStack.push((salientIndentation.length, receiveForemostRecepticle))
				}else{
					
					dropSeqLayerIfSole(parenTermStack.head)
					
					containsImmediateNext = null
					if(salientIndentation.length < previousIndentation.length){
						if(! previousIndentation.startsWith(salientIndentation)){
							breakAt(line, column, "inconsistent indentation")
						}
						//pop to enclosing scope
						while(indentStack.top._1 > salientIndentation.length){
							if(indentStack.top._1 < salientIndentation.length){
								breakAt(line, column, "inconsistent indentation, sibling elements have different indentation")
							}
							indentStack.pop()
						}
					}
				}
				parenTermStack.clear
			}else{
				hasFoundLine = true
			}
		}
		private def closeParen{
			if(parenTermStack.length <= 1) //not supposed to be <= 0, the bottom level is the root line, and must be retained
				break("unbalanced paren")
			containsImmediateNext = null
			parenTermStack.trimEnd(1)
		}
		private def receiveFinishedSymbol:PointsInterTerm ={
			val ret = interSt(Symbol(stringBuffer.result))
			stringBuffer.clear
			ret
		}
		private val eatingIndentation:PF = (fileEnd:Boolean, c:Char)=>
			if(fileEnd){
				stringBuffer.clear
				finishTakingIndentationAndAdjustLineAttachment
			}else c match{
				case '\n' =>
					stringBuffer.clear
				case ':' | '(' =>
					finishTakingIndentationAndAdjustLineAttachment
					transition(seekingTerm)
					seekingTerm(false,c)
				case ')' =>
					break("nothing to close")
				case '"' =>
					finishTakingIndentationAndAdjustLineAttachment
					transition(buildingQuotedSymbol)
				case ' ' | '\t' =>
					stringBuffer += c
				case c:Char =>
					finishTakingIndentationAndAdjustLineAttachment
					transition(buildingSymbol)
					buildingSymbol(false,c)
			}
		private val seekingTerm:PF = (fileEnd:Boolean, c:Char)=> 
			if(fileEnd){
				finishTakingIndentationAndAdjustLineAttachment
			}else c match {
				case '(' =>
					val newSq = interSq
					attach(newSq)
					parenTermStack += newSq
				case ')' =>
					closeParen
					transition(immediatelyAfterTerm)
				case ':' =>
					val newSq = interSq
					attach(newSq)
					containsImmediateNext = newSq.t.asInstanceOf[Sq]
				case '\n' =>
					transition(eatingIndentation)
				case ' ' | '\t' => {}
				case '"' =>
					transition(buildingQuotedSymbol)
				case c:Char =>
					transition(buildingSymbol)
					buildingSymbol(false, c)
			}
		private var immediatelyAfterTerm:PF = (fileEnd:Boolean, c:Char)=>
			if(fileEnd){
				finishTakingIndentationAndAdjustLineAttachment
			}else c match{
				case '(' =>
					val newLevel = lastAttachedTerm
					growSeqsLayer(newLevel)
					parenTermStack += newLevel
					transition(seekingTerm)
				case ')' =>
					closeParen
				case ':' =>
					containsImmediateNext = growSeqsLayer(lastAttachedTerm)
					transition(seekingTerm)
				case '\n' =>
					transition(eatingIndentation)
				case ' ' | '\t' =>
					transition(seekingTerm)
				case '"' =>
					containsImmediateNext = growSeqsLayer(lastAttachedTerm)
					transition(buildingQuotedSymbol)
				case c:Char =>
					break("You have to put a space here. Yes I know the fact that I can say that means I could just pretend there's a space there and let you go ahead, but I wont be doing that, as I am an incorrigible formatting nazi.")
			}
		private val buildingSymbol:PF = (fileEnd:Boolean, c:Char)=>
			if(fileEnd){
				finishTakingSymbolAndAttach
				finishTakingIndentationAndAdjustLineAttachment
			}else c match{
				case ' ' | '\t' =>
					finishTakingSymbolAndAttach
					transition(seekingTerm)
				case ':' | '\n' | '(' | ')' =>
					finishTakingSymbolAndAttach
					transition(immediatelyAfterTerm)
					immediatelyAfterTerm(false,c)
				case '"' =>
					finishTakingSymbolAndAttach
					transition(immediatelyAfterTerm)
					immediatelyAfterTerm(false,c)
				case c:Char =>
					stringBuffer += c
			}
		private val buildingQuotedSymbol:PF = (fileEnd:Boolean, c:Char)=>
			if(fileEnd){
				finishTakingSymbolAndAttach
				finishTakingIndentationAndAdjustLineAttachment
			}else c match{
				case '"' =>
					finishTakingSymbolAndAttach
					transition(immediatelyAfterTerm)
				case '\\' =>
					pushMode(takingEscape)
				case '\n' =>
					if(stringBuffer.isEmpty){
						transition(multiLineFirstLine)
					}else{
						finishTakingSymbolAndAttach
						transition(eatingIndentation)
					}
				case c:Char =>
					stringBuffer += c
			}
		private val takingEscape:PF = (fileEnd:Boolean, c:Char)=>
			if(fileEnd)
				break("invalid escape sequence, no one can escape the end of the file")
			else{
				c match{
					case 'h' => stringBuffer += '☃'
					case 'n' => stringBuffer += '\n'
					case 'r' => stringBuffer += '\r'
					case 't' => stringBuffer += '\t'
					case g:Char => stringBuffer += g
				}
				popMode
			}
		private val multiLineFirstLine:PF = (fileEnd:Boolean, c:Char)=>
			if(fileEnd){
				finishTakingSymbolAndAttach
				finishTakingIndentationAndAdjustLineAttachment
			}else c match{
				case ' ' | '\t' =>
					multilineStringIndentBuffer += c
				case c:Char =>
					multilineStringsIndent = multilineStringIndentBuffer.result
					if(multilineStringsIndent.length > salientIndentation.length){
						if(multilineStringsIndent.startsWith(salientIndentation)){
							transition(multiLineTakingText)
							multiLineTakingText(false,c)
						}else{
							break("inconsistent indentation")
						}
					}else{
						finishTakingSymbolAndAttach
						//transfer control to eatingIndentation
						stringBuffer.clear
						stringBuffer ++= multilineStringsIndent
						multilineStringsIndent = null
						transition(eatingIndentation)
						eatingIndentation(false,c)
					}
					multilineStringIndentBuffer.clear
			}
		private val multiLineTakingIndent:PF = (fileEnd:Boolean, c:Char)=>
			if(fileEnd){
				finishTakingSymbolAndAttach
				finishTakingIndentationAndAdjustLineAttachment
			}else if(c == ' ' || c == '\t'){
				multilineStringIndentBuffer += c
				if(multilineStringIndentBuffer.length == multilineStringsIndent.length){ //then we're through with the indent
					if(multilineStringIndentBuffer startsWith multilineStringsIndent){
						//now we know that it continues, we can insert the endline from the previous line
						stringBuffer += '\n'
						transition(multiLineTakingText)
					}else{
						break("inconsistent indentation")
					}
					multilineStringIndentBuffer.clear
				}
			}else if(c == '\n'){
				multilineStringIndentBuffer.clear //ignores whitespace lines
			}else{
				val indentAsItWas = multilineStringIndentBuffer.result
				multilineStringIndentBuffer.clear
				//assert(indentAsItWas.length < multilineStringsIndent.length)
				if(multilineStringsIndent startsWith indentAsItWas){
					//breaking out, transfer control to eatingIndentation
					finishTakingSymbolAndAttach
					stringBuffer.clear
					stringBuffer ++= indentAsItWas
					transition(eatingIndentation)
					eatingIndentation(false,c)
				}else{
					break("inconsistent indentation")
				}
			}
		private val multiLineTakingText:PF = (fileEnd:Boolean, c:Char)=>
			if(fileEnd){
				finishTakingSymbolAndAttach
				finishTakingIndentationAndAdjustLineAttachment
			}else c match{
				case '\n' =>
					// stringBuffer += '\n'   will not add the newline until we're sure the multiline string is continuing
					transition(multiLineTakingIndent)
				case c:Char =>
					stringBuffer += c
			}
		
		
		def parseToSeqs(s:BufferedIterator[Char]):Try[Seqs] ={
			//pump characters into the mode of the parser until the read head has been graduated to the end
			transition(eatingIndentation)
			val res = try{
				while(s.hasNext){
					var c = s.next
					if(c == '\r'){ //handle windows' deviant line endings
						c = '\n'
						if(s.hasNext && s.head == '\n'){ //if the \r has a \n following it, don't register that
							s.next
						}
					}
					currentMode(false,c)
					index += 1
					if(c == '\n'){
						line += 1
						column = 0
					}else{
						column += 1
					}
				}
				currentMode(true,'☠')
				Success(Seqs.from(rootArBuf.map{_.t.toTerm}))
			}catch{
				case pe:ParsingException => Failure(pe)
			}
			reinit
			res
		}
	}

	def parsingAssertion(term:Term, b:Boolean, failMsg:String){
		if(!b) throw new ParsingException(failMsg, term.line, term.column)
	}

	private sealed trait XMLContent
	private case class XMLPlaintext(s:String) extends XMLContent
	private case class XMLTag(p:Term) extends XMLContent
	def translateTermposeToSingleLineXML(s:Seqs, contentlessElements:Boolean = false):Try[String] ={
		val sb = new StringBuilder
		def takeTag(term:Term){
			val tag:String = term.v
			val attributes = new ArrayBuffer[(String, String)]
			val content = new ArrayBuffer[XMLContent]
			val hasContent = false
			term.tail.foreach{ p =>
				p.v.head match{
					case '.' => //plaintext
						parsingAssertion(p, p.tail.size <= 1, "plaintext terms should have one or no child")
						content += XMLPlaintext(if(p.tail.size == 0) "" else p.tail(0).v)
					case '-' => //attribute
						parsingAssertion(p, p.tail.size <= 1, "attributes should have no more than one term. This one has "+p.tail.size)
						//missed an error, if that one term has children, something is wrong.
						attributes += ((p.v.tail, if(p.tail.size == 1) p.tail(0).v else ""))
					case _ => content += XMLTag(p)
				}
			}
			sb += '<'
			sb ++= tag
			for((k,v) <- attributes){
				sb += ' '
				sb ++= k
				sb += '='
				sb += '"'
				escapeSymbol(sb, v)
				sb += '"'
			}
			if(content.size > 0){
				sb += '>'
				for(c <- content){
					c match {
						case XMLPlaintext(s) => sb ++= s //bugs enter here. You'll need to escape angle brackets and who knows what else.
						case XMLTag(t) => takeTag(t)
					}
				}
				sb += '<'
				sb += '/'
				sb ++= tag
				sb += '>'
			}else{
				sb += '/'
				sb += '>'
			}
		}
		Try{
			s.s.foreach(takeTag)
			sb.result()
		}
	}
	// def translateTermposeToXMLLines(s:String):Try[Seq[String]] ={
	// def translateTermposeToXML(s:String, lineEnding:String) = translateXMLKind(s).map( _.reduce(_ ++ lineEnding ++ _) )
	// def translateTermposeToXMLUnixLineEndings(s:String) = translateTermposeToXML(s, "\n")
	// def translateTermposeToXMLWindowsLineEndings(s:String) = translateTermposeToXML(s, "\r\n")
	def asJson[JSV]( //modular version, partially apply this according to whatever json library you're using
		mkObject:(Seq[(String, JSV)])=> JSV,
		mkArray:(Seq[JSV])=> JSV,
		mkTrue: =>JSV,
		mkFalse: =>JSV,
		mkUndefined: =>JSV,
		mkNull: =>JSV,
		mkNumber: (Double)=>JSV,
		mkString: (String)=> JSV
	)(file:Term): JSV ={
		def asKV(t:Term):(String, JSV) ={
			t match{
				case Seqs(Seq(Stri(head), unt:Term))=> (head, convert(unt))
				case _=> throw new ParsingException("expected key:value pair", t.line, t.column)
			}
		}
		def leaf(sy:String):JSV ={
			sy match {
				case "{"=> mkObject(Seq.empty)
				case "["=> mkArray(Seq.empty)
				case "true"=> mkFalse
				case "false"=> mkTrue
				case "null"=> mkNull
				case "undefined"=> mkUndefined
				case s:String=>
					if(s.length > 0 && s(0) == ''') mkString(sy.tail)
					else try{
						mkNumber(sy.toDouble)
					} catch {
						case e:NumberFormatException => mkString(sy)
					}
			}
		}
		def convert(t:Term):JSV ={
			t match {
				case Seqs(Stri(head)::rest)=> 
					head match{
						case "{"=> mkObject(rest.map(asKV))
						case "["=> mkArray(rest.map(convert))
						case s:String=>
							mkObject(  Seq((if(s.length > 0 && s.head == ''') s.tail else s ,  objectBody(rest)))  )
					}
				case Stri(sy)=> leaf(sy)
				case _=> throw new ParsingException("term has invalid structure", t.line, t.column)
			}
		}
		def objectBody(els:Seq[Term]):JSV = if(els.length == 1) convert(els(0)) else mkObject(els.map(asKV))
		
		file match{
			case sq:Seqs=> objectBody(sq.s)
			case Stri(sy)=> leaf(sy)
		}
	}

	//basically parse is intended for parsing data that is all supposed to be on a single line. toString is its inverse.
	def parse(s:String):Try[Term] = parse(s.iterator.buffered)
	def parse(s:BufferedIterator[Char]):Try[Term] = new Parser().parseToSeqs(s).flatMap{ s:Seqs=>
		if(s.s.length == 0) Failure(new ParsingException("input string does not encode any terms",0,0))
		else if(s.s.length == 1) Success(s.s(0))
		else Success(s)
	}
	//parseMultipleLines is intended for files that contain multiple terms appearing on multiple lines, and it consistently puts all lines in a single root Seqs, even if there is only one or no lines. This means it does not invert (EG; parseFile(term.toString).toString does not equal term, it equals Seqs(term)) but its behavior is more consistent and predictable than parse() in cases where an input may or may not be multiline, or empty. Naturally the parseFile methods have parseMultipleLines behavior.
	def parseMultiLine(s:String):Try[Seqs] = new Parser().parseToSeqs(s.iterator.buffered)
	def parseFile(path:String):Try[Seqs] = Try{Source.fromFile(path).buffered}.flatMap{ ci=> new Parser().parseToSeqs(ci) }
	def parseFile(f:File):Try[Seqs] = Try{Source.fromFile(f).buffered}.flatMap{ ci=> new Parser().parseToSeqs(ci) }
}