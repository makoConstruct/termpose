import termpose._
import util.{Try, Success, Failure}
import collection.mutable.{ArrayStack}
object Termpose{
	trait Stringificable{ //has a method for stringifying through a stringBuilder, provides a toString through this
		def stringify(sb:StringBuilder):Unit
		override def toString ={
			val buf = new StringBuilder
			stringify(buf)
			buf.result
		}
	}
	sealed trait Pose extends Stringificable{
		val v:String
		def stringifySymbol(sb:StringBuilder) =
			if(
				v.indexOf(' ') >= 0 ||
				v.indexOf('\t') >= 0 ||
				v.indexOf(':') >= 0 ||
				v.indexOf(')') >= 0 ||
				v.indexOf('(') >= 0
			)
				sb += '"' ++= v.replaceAll("\"", "\\\"").replaceAll("\n", "\\n").replaceAll("\\", "\\\\") += '"'
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
				s(0).stringify(sb)
				for(i <- 1 until s.length){
					s(i).stringify(sb)
					sb += ' '
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
			val s = ArrayStack[InterTerm]()
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
		private var resultSeq = new ArrayStack[InterTerm]
		private var headTerm:InterTerm = null
		private var foremostTerm:InterTerm = null
		private val parenTermStack = new ArrayStack[InterTerm]
		private var tailestTermSequence:ArrayStack[InterTerm] = resultSeq
		private val parenParentStack = new ArrayStack[InterTerm]
		private var line = 0
		private var column = 0
		private var index = 0
		private type PF = PartialFunction[Char, Unit]
		private var currentMode:PF = null
		private val modes = new ArrayStack[PF]
		private val indentStack = ArrayStack[(Int, ArrayStack[InterTerm])]((0, resultSeq))  //indentation length, tailestTermSequence. It is enough to keep track of length. We can derive ∀(a,b∈Line.Indentation) a.length = b.length → a = b from the prefix checking that is done before adding to the stack
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
			resultSeq = new ArrayStack[InterTerm]
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
			currentMode = modes.pop
		}
		private def nline = {
			line += 1
			column = 0
			index += 1
		}
		private def ncol = {
			column += 1
			index += 1
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
				tailestTermSequence = foremostTerm.s
			}else{
				parenTermStack.head.s += foremostTerm
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
				previousTailestTermSequence += headTerm
			}else if(salientIndentation.length < previousIndentation.length){
				if(! previousIndentation.startsWith(salientIndentation)){
					breakAt(headTerm.line, headTerm.column, "inconsistent indentation")
				}
				//pop to enclosing scope
				while(indentStack.head._1 > salientIndentation.length){
					if(indentStack.head._1 < salientIndentation.length){
						breakAt(headTerm.line, headTerm.column, "inconsistent indentation, sibling elements have different indentation")
					}
					indentStack.pop
				}
				indentStack.head._2 += headTerm
			}
			
			previousTailestTermSequence = tailestTermSequence
			previousIndentation = salientIndentation
			tailestTermSequence = resultSeq
			parenTermStack.clear
			headTerm = null
			foremostTerm = null
			nline
		}
		private def getColonBuddy{
			ncol
			colonHead = foremostTerm
			transition(seekingColonBuddy)
		}
		private def closeParen ={
			if(parenTermStack.isEmpty)
				break("unbalanced paren")
			parenTermStack.pop
			ncol
			transition(seekingInLine)
		}
		private def openParen ={
			parenTermStack += foremostTerm
			ncol
			transition(enteringParen)
		}
		private def eatingIndentation:PF ={
			case '\n' =>
				stringBuffer.clear
				nline
			case ':' =>
				break("colon not allowed here")
			case '(' =>
				break("what are you openingggg")
			case ')' =>
				break("what are you closingggg")
			case '"' =>
				finishTakingIndentation
				ncol
				transition(startingToTakeQuoteTerm)
			case c:Char =>
				if(c == ' ' || c == '\t'){
					stringBuffer += c
					ncol
				}else{
					finishTakingIndentation
					transition(takeTerm)
				}
		}
		private def seekingInLine:PF ={
			case '(' =>
				openParen
			case ')' =>
				if(parenTermStack.isEmpty){
					break("unbalanced paren")
				}else{
					ncol
					closeParen
				}
			case ':' =>
				getColonBuddy
			case ' ' | '\t' =>
				ncol
			case '\n' =>
				if(parenTermStack.isEmpty){
					finishLine
					transition(eatingIndentation)
				}else{
					break("line end before closing paren")
				}
			case '"' =>
				ncol
				transition(startingToTakeQuoteTerm)
			case c:Char =>
				transition(takeTerm)
		}
		private def takeTerm:PF ={
			case ' ' | '\t' | ':' | '\n' | '(' | ')' | '"' =>
				finishTakingTermAndAttach
				transition(seekingInLine)
			case c:Char =>
				stringBuffer += c
				ncol
		}
		private def enteringParen:PF ={
			case ' ' | '\t' =>
				ncol
			case ':' =>
				break("colon wat")
			case '(' =>
				break("paren opens nothing")
			case ')' =>
				ncol
				closeParen
			case '"' =>
				ncol
				transition(takingQuoteTermThatMustTerminateWithQuote)
			case '\n' =>
				break("newline before paren completion (parens are only for single lines)")
			case c:Char =>
				transition(takeTerm)
		}
		private def seekingColonBuddy:PF ={
			case ' ' | '\t' =>
				ncol
			case ':' =>
				break("double colon wat")
			case '\n' =>
				break("special end of line colon syntax update has not yet come")
			case '(' =>
				//fine whatever I'll just forget the colon ever happened
				openParen
			case ')' =>
				break("closebracket after colon wat")
			case '"' =>
				ncol
				transition(startingToTakeQuoteTerm)
			case c:Char =>
				transition(takeTerm)
		}
		private def takingQuoteTermThatMustTerminateWithQuote:PF ={
			case '"' =>
				finishTakingTermAndAttach
				ncol
				transition(seekingInLine)
			case '\n' =>
				break("quote must terminate before newline")
			case '\\' =>
				ncol
				pushMode(takingEscape)
			case c:Char =>
				stringBuffer += c
				ncol
		}
		private def startingToTakeQuoteTerm:PF ={
			case '"' =>
				finishTakingTermAndAttach
				ncol
				transition(seekingInLine)
			case '\n' =>
				if(!parenTermStack.isEmpty) break("newline with unbalanced parens(if you want multiline string syntax, that's an indented block under a single quote)")
				stringBuffer.clear
				nline
				transition(multiLineFirstLine)
			case c:Char =>
				if(c == ' ' || c == '\t'){
					stringBuffer += c
					ncol
				}else{
					transition(takingSingleLineQuoteTerm)
				}
		}
		private def takingSingleLineQuoteTerm:PF ={
			case '"' =>
				finishTakingTermAndAttach
				ncol
				transition(seekingInLine)
			case '\\' =>
				ncol
				pushMode(takingEscape)
			case '\n' =>
				finishTakingTermAndAttach
				transition(seekingInLine)
			case c:Char =>
				stringBuffer += c
				ncol
		}
		private def takingEscape:PF ={
			case c:Char =>
				c match{
					case 'n' => stringBuffer += '\n'
					case 't' => stringBuffer += '\t'
					case 'h' => stringBuffer += '☃'
					case g:Char => stringBuffer += g
				}
				ncol
				popMode
		}
		private def multiLineFirstLine:PF ={
			case c:Char =>
				if(c == ' ' || c == '\t'){
					ncol
					multiLineIndent += c
				}else{
					mlIndent = multiLineIndent.result
					if(mlIndent.length > salientIndentation.length){
						if(salientIndentation.startsWith(mlIndent)){
							multiLineIndent.clear
							transition(multiLineTakingText)
						}else{
							break("inconsistent indentation")
						}
					}else{
						break("multiline string must be indented")
					}
				}
		}
		private def multiLineTakingIndent:PF ={
			case c:Char =>
				if(c == ' ' || c == '\t'){
					ncol
					multiLineIndent += c
				}else{
					val nin = multiLineIndent.result
					multiLineIndent.clear
					if(nin.length >= mlIndent.length){
						if(nin.startsWith(mlIndent)){
							if(nin.length > mlIndent.length)
								stringBuffer ++= mlIndent.substring(nin.length, mlIndent.length)
							transition(multiLineTakingText)
						}else{
							break("inconsistent indentation")
						}
					}else{
						if(mlIndent.startsWith(nin)){
							//breaking out, returning to usual scanning
							finishTakingTermAndAttach
							salientIndentation = mlIndent
							transition(takeTerm)
						}else{
							break("inconsistent indentation")
						}
					}
				}
		}
		private def multiLineTakingText:PF ={
			case '\n' =>
				stringBuffer += '\n'
				nline
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
					if(currentMode.isDefinedAt(c))
						currentMode(c)
					else
						break("can't handle a '"+c+"' at this point")
				}
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