import util.{Try, Success, Failure}
import collection.mutable.{ArrayBuffer, HashMap}
object Termpose{
	trait Stringificable{ //has a method for stringifying through a stringBuilder, provides a toString through this
		def stringify(sb:StringBuilder):Unit
		override def toString ={
			val buf = new StringBuilder
			stringify(buf)
			buf.result
		}
	}
	def asJsonString(ps:Seq[Pose]):String ={
		val sb = new StringBuilder
		sb += '['
		if(!ps.isEmpty){
			ps(0).buildJsonString(sb)
			for(i <- 1 until ps.length){
				sb += ','
				ps(i).buildJsonString(sb)
			}
		}
		sb += ']'
		sb.result
	}
	def minified(ps:Seq[Pose]):String ={
		val sb = new StringBuilder
		for(i <- 0 until ps.length){
			ps(i).stringify(sb)
			sb += '\n'
		}
		sb.result
	}
		
	private def mightBeReservedChar(c:Char) = c < 59 //heuristic, no false negatives
	//history story: private def mightBeReservedChar(c:Char) = (c&(-60)) == 0 //filters out 94.8% of non-matches. I used to use a magic xor shift and a magic number I'd trauled up from brutespace, til I realized this was all they were doing. How it works: It filters everything greater than 64 and every x for which x%8 < 4 //can't use this beauty no more cause of windows lineendings; '\r'%8 is not less than four.
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
	case class TermposeAccessError(msg:String) extends RuntimeException(msg)
	// import language.dynamic
	sealed trait Pose extends Stringificable{
		val v:Symbol
		val line:Int
		val column:Int
		def map:Map[Symbol, Seq[Pose]]
		// def selectDynamic(k:Symbol):Symbol = map.get(k) match{
		// 	case Some(p) =>
		// 		if(p.tail.length == 1){
		// 			p.tail(0)
		// 		}else if(p.tail.length < 1){
		// 			throw new TermposeAccessError("empty field "+k)
		// 		}else{
		// 			throw new TermposeAccessError("multiple possible matches for field "+k)
		// 		}
		// 	case None =>
		// 		throw new TermposeAccessError("no such field")
		// }
		def getValue(k:Symbol):Option[Symbol] = map.get(k).flatMap { p =>
			if(p.tail.length == 1)
				Some(p.tail(0).term)
			else None
		}
		def getSub(k:Symbol):Option[Pose] = map.get(k).flatMap { p =>
			if(p.length == 0)
				Some(p(0))
			else
				None
		}
		def apply(k:Int):Pose = tail(k)
		def term:Symbol = v
		def tail:Seq[Pose]
		protected def stringifySymbol(sb:StringBuilder) =
			if(
				v.name.exists { c =>
					mightBeReservedChar(c) && (
						c match{
							case ' ' => true
							case '(' => true
							case ':' => true
							case '\t' => true
							case '\n' => true
							case '\r' => true
							case ')' => true
							case _:Char => false
						}
					)
				}
			){
				sb += '"'
				escapeSymbol(sb, v.name)
				sb += '"'
			}else{
				sb ++= v.name
			}
		def jsonString:String ={
			val sb = new StringBuilder
			buildJsonString(sb)
			sb.result
		}
		def buildJsonString(sb:StringBuilder){
			sb += '"'
			escapeSymbol(sb, v.name)
			sb += '"'
		}
	}
	
	class Leaf(val v:Symbol, val line:Int, val column:Int) extends Pose{
		def tail:Seq[Pose] = Seq.empty
		lazy val _map:Map[Symbol, Seq[Pose]] = Map.empty
		def stringify(sb:StringBuilder) = stringifySymbol(sb)
		def map = _map
	}
	class Term(val v:Symbol, val line:Int, val column:Int, val s:Seq[Pose]) extends Pose {
		def tail:Seq[Pose] = s
		lazy val _map:Map[Symbol, Seq[Pose]] = {
			val m = new HashMap[Symbol, Seq[Pose]]
			for(p <- s){
				m(p.term) = p.tail
			}
			m.toMap
		}
		def map = _map
		def stringify(sb:StringBuilder) ={
			if(s.isEmpty){
				stringifySymbol(sb)
			}else{
				stringifySymbol(sb)
				sb += '('
				s.head.stringify(sb)
				for(i <- 1 until s.length){
					sb += ' '
					s(i).stringify(sb)
				}
				sb += ')'
			}
		}
		def apply(key:Symbol):Option[Pose] = s.find { t => t.v equals key }
		def each(key:Symbol):Seq[Pose] = s.filter { t => t.v equals key }
		override def buildJsonString(sb:StringBuilder){
			sb += '['
			sb += '"'
			escapeSymbol(sb, v.name)
			sb += '"'
			for(sub <- s){
				sb += ','
				sub.buildJsonString(sb)
			}
			sb += ']'
		}
	}
	
	
	
	class Parser{
		private class ParsingException(msg:String) extends RuntimeException(msg)
		private class InterTerm(val symbol:Symbol, val line:Int, val column:Int){
			val s = ArrayBuffer[InterTerm]()
			def toPose:Pose = {
				if(s.isEmpty)
					new Leaf(symbol, line, column)
				else
					new Term(symbol, line, column, s.map { _.toPose })
			}
		}
		
		private val stringBuffer = new StringBuilder(512)
		private var foremostSymbol:Symbol = Symbol("")
		private var salientIndentation:String = ""
		private var resultSeq = new ArrayBuffer[InterTerm]
		private var headTerm:InterTerm = null
		private var foremostTerm:InterTerm = null
		private val parenTermStack = new ArrayBuffer[InterTerm]
		private var tailestTermSequence:ArrayBuffer[InterTerm] = resultSeq
		private val parenParentStack = new ArrayBuffer[InterTerm]
		private var line = 0
		private var column = 0
		private var index = 0
		private type PF = Char => Unit
		private var currentMode:PF = null
		private val modes = new ArrayBuffer[PF]
		private val indentStack = ArrayBuffer[(Int, ArrayBuffer[InterTerm])]((0, resultSeq))  //indentation length, tailestTermSequence. It is enough to keep track of length. We can derive ∀(a,b∈Line.Indentation) a.length = b.length → a = b from the prefix checking that is done before adding to the stack
		private var previousIndentation = ""
		private var previousTailestTermSequence = resultSeq
		private val multiLineIndent = new StringBuilder
		private var mlIndent:String = null
		private var containsImmediateNext:InterTerm = null
		
		private def reinit{
			//=___=
			stringBuffer.clear
			foremostSymbol = Symbol("")
			salientIndentation = ""
			resultSeq = new ArrayBuffer[InterTerm]
			headTerm = null
			foremostTerm = null
			parenTermStack.clear
			tailestTermSequence = resultSeq
			parenParentStack.clear
			line = 0 
			column = 0
			index = 0
			currentMode = null
			modes.clear
			indentStack.clear
			indentStack += ((0, resultSeq))
			previousIndentation = ""
			previousTailestTermSequence = resultSeq
			multiLineIndent.clear
			mlIndent = null
		}
		
		
		//general notes:
		//the parser consists of a loop that pumps chars from the input string into whatever the current Mode is. Modes jump from one to the next according to cues, sometimes forwarding the cue onto the new mode before it gets any input from the input loop. There is a mode stack, but it is rarely used. Modes are essentially just a Char => Unit (named 'PF', short for Processing Funnel(JJ it's legacy from when they were Partial Functions)). An LF ('\n') is inserted artificially at the end of input to ensure that line end processes are called. CR LFs ("\r\n"s) are filtered to a single LF (This does mean that a windows formatted file will have unix line endings when parsing multiline strings, that is not a significant issue, ignoring the '\r's will probably save more pain than it causes and the escape sequence \r is available if they're explicitly desired).
		
		
		private def interTerm(symbol:Symbol) = new InterTerm(symbol, line, column)
		private def transition(nm:PF){ currentMode = nm }
		private def pushMode(nm:PF){
			modes += currentMode
			transition(nm)
		}
		private def popMode{
			currentMode = modes.last
			modes.trimEnd(1)
		}
		private def break(message:String):Unit = breakAt(line, column, message)
		private def breakAt(l:Int, c:Int, message:String):Unit = throw new ParsingException("line:"+l+" column:"+c+", no, bad: "+message)
		private def finishTakingSymbol ={
			foremostSymbol = Symbol(stringBuffer.result)
			stringBuffer.clear
			foremostSymbol
		}
		private def finishTakingTermAndAttach ={
			foremostTerm = interTerm(finishTakingSymbol)
			if(containsImmediateNext != null){
				containsImmediateNext.s += foremostTerm
				containsImmediateNext = null
			}else if(headTerm == null){
				headTerm = foremostTerm
				tailestTermSequence = foremostTerm.s
			}else if(parenTermStack.isEmpty){
				headTerm.s += foremostTerm
			}else{
				parenTermStack.last.s += foremostTerm
			}
		}
		private def finishTakingIndentation ={
			salientIndentation = stringBuffer.result
			stringBuffer.clear
			salientIndentation
		}
		private def finishLine ={
			//now knit this line's data into the previous
			if(salientIndentation.length > previousIndentation.length){
				if(! salientIndentation.startsWith(previousIndentation)){
					breakAt(headTerm.line, headTerm.column, "inconsistent indentation at")
				}
				indentStack += ((salientIndentation.length, previousTailestTermSequence))
			}else if(salientIndentation.length < previousIndentation.length){
				if(! previousIndentation.startsWith(salientIndentation)){
					breakAt(headTerm.line, headTerm.column, "inconsistent indentation")
				}
				//pop to enclosing scope
				while(indentStack.last._1 > salientIndentation.length){
					if(indentStack.last._1 < salientIndentation.length){
						breakAt(headTerm.line, headTerm.column, "inconsistent indentation, sibling elements have different indentation")
					}
					indentStack.trimEnd(1)
				}
			}
			
			indentStack.last._2 += headTerm
			
			containsImmediateNext = null
			previousIndentation = salientIndentation
			previousTailestTermSequence = tailestTermSequence
			tailestTermSequence = resultSeq
			parenTermStack.clear
			headTerm = null
			foremostTerm = null
		}
		def finishLineNormally{ //where as finishing abnormally is what multiline reading does because by the time it finishes it has already eaten the indentation and it doesn't ask whether the parenTermStack's empty
			// if(parenTermStack.isEmpty){
				finishLine
				transition(eatingIndentation)
			// }else{
			// 	break("line end before closing paren")
			// }
		}
		private def closeParen ={
			if(parenTermStack.isEmpty)
				break("unbalanced paren")
			parenTermStack.trimEnd(1)
			transition(seekingInLine)
		}
		private def openParen ={
			parenTermStack += foremostTerm
			transition(enteringParen)
		}
		private val eatingIndentation:PF = (c:Char)=> c match{
			case '\n' =>
				stringBuffer.clear
			case ':' =>
				break("colon not allowed here")
			case '(' =>
				break("what are you openingggg")
			case ')' =>
				break("what are you closingggg")
			case '"' =>
				finishTakingIndentation
				transition(startingToTakeQuoteTerm)
			case ' ' | '\t' =>
				stringBuffer += c
			case c:Char =>
				finishTakingIndentation
				transition(takeTerm)
				takeTerm(c)
		}
		private val seekingInLine:PF = (c:Char)=> c match{
			case '(' =>
				openParen
			case ')' =>
				closeParen
			case ':' =>
				containsImmediateNext = foremostTerm
				transition(seekingColonBuddy)
			case ' ' | '\t' => {}
			case '\n' =>
				finishLineNormally
			case '"' =>
				transition(startingToTakeQuoteTerm)
			case c:Char =>
				transition(takeTerm)
				takeTerm(c)
		}
		private val takeTerm:PF = (c:Char)=> c match{
			case ' ' | '\t' | ':' | '\n' | '(' | ')' =>
				finishTakingTermAndAttach
				transition(seekingInLine)
				seekingInLine(c)
			case '"' =>
				finishTakingTermAndAttach
				containsImmediateNext = foremostTerm
				transition(startingToTakeQuoteTerm)
			case c:Char =>
				stringBuffer += c
		}
		private val enteringParen:PF = (c:Char)=> c match{
			case ' ' | '\t' => {}
			case ':' =>
				break("colon wat")
			case '(' =>
				break("paren opens nothing")
			case ')' =>
				closeParen
			case '"' =>
				transition(takingSingleLineQuoteTerm)
			case '\n' =>
				break("newline before paren completion (parens are only for single lines)")
			case c:Char =>
				transition(takeTerm)
				takeTerm(c)
		}
		private val seekingColonBuddy:PF = (c:Char)=> c match{
			case ' ' | '\t' => {}
			case ':' =>
				break("double colon wat")
			case '\n' =>
				tailestTermSequence = containsImmediateNext.s
				finishLineNormally
			case '(' =>
				//fine whatever I'll just forget the colon ever happened
				openParen
			case ')' =>
				break("closebracket after colon wat")
			case '"' =>
				transition(startingToTakeQuoteTerm)
			case c:Char =>
				transition(takeTerm)
				takeTerm(c)
		}
		private val startingToTakeQuoteTerm:PF = (c:Char)=> c match{
			case '"' =>
				finishTakingTermAndAttach
				transition(seekingInLine)
			case '\n' =>
				if(!parenTermStack.isEmpty) break("newline with unbalanced parens(if you want multiline string syntax, that's an indented block under a single quote)")
				stringBuffer.clear
				transition(multiLineFirstLine)
			case ' ' | '\t' =>
				stringBuffer += c
			case c:Char =>
				transition(takingSingleLineQuoteTerm)
				takingSingleLineQuoteTerm(c)
		}
		private val takingSingleLineQuoteTerm:PF = (c:Char)=> c match{
			case '"' =>
				finishTakingTermAndAttach
				transition(seekingInLine)
			case '\\' =>
				pushMode(takingEscape)
			case '\n' =>
				finishTakingTermAndAttach
				transition(seekingInLine)
				seekingInLine('\n')
			case c:Char =>
				stringBuffer += c
		}
		//history story: takingQuoteTermThatMustTerminateWithQuote was an artifact from when line breaks wern't allowed during parens, it was really just the above with a breaker response to '\n's instead of a handler. We now handle such events reasonably.
		private val takingEscape:PF = (c:Char)=> {
			c match{
				case 'n' => stringBuffer += '\n'
				case 'r' => stringBuffer += '\r'
				case 't' => stringBuffer += '\t'
				case 'h' => stringBuffer += '☃'
				case g:Char => stringBuffer += g
			}
			popMode
		}
		private val multiLineFirstLine:PF = (c:Char)=> c match{
			case ' ' | '\t' =>
				multiLineIndent += c
			case c:Char =>
				mlIndent = multiLineIndent.result
				if(mlIndent.length > salientIndentation.length){
					if(mlIndent.startsWith(salientIndentation)){
						multiLineIndent.clear
						transition(multiLineTakingText)
						multiLineTakingText(c)
					}else{
						break("inconsistent indentation")
					}
				}else{
					break("multiline string must be indented")
				}
		}
		private val multiLineTakingIndent:PF = (c:Char)=> c match{
			case c:Char =>
				if(c == ' ' || c == '\t'){
					multiLineIndent += c
				}else{
					val nin = multiLineIndent.result
					multiLineIndent.clear
					if(nin.length >= mlIndent.length){
						if(nin.startsWith(mlIndent)){
							stringBuffer += '\n' //add the newline from the previous line, safe in the knowledge that the multiline string continues up to and beyond that point
							if(nin.length > mlIndent.length)
								stringBuffer ++= mlIndent.substring(nin.length, mlIndent.length)
							transition(multiLineTakingText)
							multiLineTakingText(c)
						}else{
							break("inconsistent indentation")
						}
					}else{
						if(mlIndent.startsWith(nin)){
							//breaking out, returning to usual scanning
							finishTakingTermAndAttach
							finishLine
							salientIndentation = nin
							transition(takeTerm)
							takeTerm(c)
						}else{
							break("inconsistent indentation")
						}
					}
				}
		}
		private val multiLineTakingText:PF = (c:Char)=> c match{
			case '\n' =>
				// stringBuffer += '\n'   will not add the newline until we're sure the multiline string is continuing
				transition(multiLineTakingIndent)
			case c:Char =>
				stringBuffer += c
		}
		
		
		
		def parse(s:String):Try[Seq[Pose]] ={
			//pump characters into the mode of the parser until the read head has been graduated to the end
			transition(eatingIndentation)
			val res = try{
				while(index < s.length){
					var c = s.charAt(index)
					if(c == '\r'){ //handle windows' deviant line endings
						if(s.charAt(index+1) == '\n'){
							index += 1
						}
						c = '\n'
					}
					currentMode(c)
					index += 1
					if(c == '\n'){
						line += 1
						column = 0
					}else{
						column += 1
					}
				}
				if(s.last != '\n')
					currentMode('\n') //simulated final newline
				Success(resultSeq.map { _.toPose })
			}catch{
				case pe:ParsingException =>
					Failure(pe)
			}
			reinit
			res
		}
	}
	
	def parsingAssertion(pose:Pose, b:Boolean, failMsg:String){
		if(!b) throw new RuntimeException("parsing failure at line:"+pose.line+" column:"+pose.column+". "+failMsg)
	}
	
	private sealed trait XMLContent
	private case class XMLPlaintext(s:String) extends XMLContent
	private case class XMLTag(p:Pose) extends XMLContent
	def translateTermposeToSingleLineXML(s:String):Try[String] ={
		val sb = new StringBuilder
		def takeTag(pose:Pose){
			val tag:String = pose.term.name
			val attributes = new ArrayBuffer[(String, String)]
			val content = new ArrayBuffer[XMLContent]
			val hasContent = false
			pose.tail.map{ p =>
				p.term.name.head match{
					case '.' => //plaintext
						parsingAssertion(p, p.tail.size <= 1, "plaintext terms should have one or no child")
						content += XMLPlaintext(if(p.tail.size == 0) "" else p.tail(0).term.name)
					case '-' => //attribute
						parsingAssertion(p, p.tail.size <= 1, "attributes should have no more than one term. This one has "+p.tail.size)
						//missed an error, if that one term has children, something is wrong.
						attributes += ((p.term.name.tail, if(p.tail.size == 1) p.tail(0).term.name else ""))
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
		new Parser().parse(s).map{ poses =>
			poses.foreach(takeTag)
			sb.result
		}
	}
	// def translateTermposeToXMLLines(s:String):Try[Seq[String]] ={
	// def translateTermposeToXML(s:String, lineEnding:String) = translateXMLKind(s).map( _.reduce(_ ++ lineEnding ++ _) )
	// def translateTermposeToXMLUnixLineEndings(s:String) = translateTermposeToXML(s, "\n")
	// def translateTermposeToXMLWindowsLineEndings(s:String) = translateTermposeToXML(s, "\r\n")
}