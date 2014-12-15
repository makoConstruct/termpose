import util.{Try, Success, Failure}
import collection.mutable.{ArrayBuffer}
object Termpose{
	trait Stringificable{ //has a method for stringifying through a stringBuilder, provides a toString through this
		def stringify(sb:StringBuilder):Unit
		override def toString ={
			val buf = new StringBuilder
			stringify(buf)
			buf.result
		}
	}
	private def escapeSymbol(str:String):String ={
		val sb = new StringBuilder
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
				case c:Char =>
					sb += c
			}
		}
		sb.result
	}
	sealed trait Pose extends Stringificable{
		val v:String
		def stringifySymbol(sb:StringBuilder) =
			if(
				v.indexOf(' ') >= 0 ||
				v.indexOf('(') >= 0 ||
				v.indexOf(':') >= 0 ||
				v.indexOf('\t') >= 0 ||
				v.indexOf('\n') >= 0 ||
				v.indexOf(')') >= 0
			)
				sb += '"' ++= escapeSymbol(v) += '"'
			else
				sb ++= v
	}
	case class Leaf(val v:String) extends Pose{
		def stringify(sb:StringBuilder) = stringifySymbol(sb)
	}
	case class Term(val v:String, val s:Seq[Pose]) extends Pose {
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
		def apply(key:String):Option[Pose] = s.find { t => t.v equals key }
		def each(key:String):Seq[Pose] = s.filter { t => t.v equals key }
	}
	class Parser{
		private class ParsingException(msg:String) extends RuntimeException(msg)
		private class InterTerm(val symbol:String, val line:Int, val column:Int){
			val s = ArrayBuffer[InterTerm]()
			def toPose:Pose = {
				if(s.isEmpty)
					Leaf(symbol)
				else
					Term(symbol, s.map { _.toPose })
			}
		}
		
		private val stringBuffer = new StringBuilder(512)
		private var foremostSymbol:String = ""
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
		private var colonHead:InterTerm = null
		
		private def reinit{
			//=___=
			stringBuffer.clear
			foremostSymbol = ""
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
		
		private def interTerm(symbol:String) = new InterTerm(symbol, line, column)
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
			foremostSymbol = stringBuffer.result
			stringBuffer.clear
			foremostSymbol
		}
		private def finishTakingTermAndAttach ={
			foremostTerm = interTerm(finishTakingSymbol)
			if(colonHead != null){
				colonHead.s += foremostTerm
				colonHead = null
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
			
			colonHead = null
			previousIndentation = salientIndentation
			previousTailestTermSequence = tailestTermSequence
			tailestTermSequence = resultSeq
			parenTermStack.clear
			headTerm = null
			foremostTerm = null
		}
		def finishLineNormally{ //where as finishing abnormally is what multiline reading does because by the time it finishes it has already eaten the indentation and it doesn't ask whether the parenTermStack's empty
			if(parenTermStack.isEmpty){
				finishLine
				transition(eatingIndentation)
			}else{
				break("line end before closing paren")
			}
		}
		private def getColonBuddy{
			colonHead = foremostTerm
			transition(seekingColonBuddy)
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
				getColonBuddy
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
			case ' ' | '\t' | ':' | '\n' | '(' | ')' | '"' =>
				finishTakingTermAndAttach
				transition(seekingInLine)
				seekingInLine(c)
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
				transition(takingQuoteTermThatMustTerminateWithQuote)
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
				tailestTermSequence = colonHead.s
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
		private val takingQuoteTermThatMustTerminateWithQuote:PF = (c:Char)=> c match{
			case '"' =>
				finishTakingTermAndAttach
				transition(seekingInLine)
			case '\n' =>
				break("quote must terminate before newline")
			case '\\' =>
				pushMode(takingEscape)
			case c:Char =>
				stringBuffer += c
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
		private val takingEscape:PF = (c:Char)=> c match{
			case c:Char =>
				c match{
					case 'n' => stringBuffer += '\n'
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
					val c = s.charAt(index)
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
}