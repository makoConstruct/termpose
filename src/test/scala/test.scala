import collection.mutable.ArrayBuffer
import org.scalatest._
import util.{Try, Success, Failure}
class TermposeParserSpec extends WordSpec with Matchers with TryValues{
	val gp = new Termpose.Parser //tests share a global parser deliberately. If there's any way we can cause state to leak between tests, I damn well want to elicit it, and I can't imagine a better way to get that to happen.
	def expectsEqual(input:String, output:String) ={
		val pinp = gp.parse(input).success.value
		Termpose.minified(pinp) === output
	}
	"Parser" should{
		"generally work" in{
			expectsEqual("""
			a
			""", "a")
			expectsEqual("""
			a(b(c(d)))
			""", "a(b(c(d)))")
			expectsEqual("""
			a b
			""", "a(b)")
			expectsEqual("""
			a b c
			""", "a(b c)")
			expectsEqual("""
			a
			  b
			  c
			""", "a(b c)")
			expectsEqual("""
			a b
			  c
			""", "a(b c)")
			expectsEqual("""
			Aaron b""", "Aaron(b)")
			expectsEqual("""
			Aaron
				because
				always
					keep
					dogs
						in pens
				keep
				swimming
			""", "Aaron(because always(keep dogs(in(pens))) keep swimming)")
			expectsEqual("""
			Oglomere
			Oglomore
			""", "Oglomere Oglomore")
			expectsEqual("""
			"horrid clown"
				make "my day"
				horrid clown
					jp awesome:ross
			""", "\"horrid clown\"(make(\"my day\") horrid(clown jp(awesome(ross))))")
			expectsEqual("""
			how about a "
				~M~U~L~T~I~L~I~N~E~
				~S~T~R~I~N~G~
			//"YOU LIKE?"
			""", "how(about a \"~M~U~L~T~I~L~I~N~E~\\n~S~T~R~I~N~G~\") //(\"YOU LIKE?\")")
			expectsEqual("""
			hobobo
				green witches:
					glon
					mora
					bonell
				emero
			""", "hobobo(green(witches(glon mora bonell)) emero)")
			expectsEqual("""a(bb(aod(igjd(ss
				ojfjf
				jjj(s:
					2""", "a(bb(aod(igjd(ss))) ojfjf jjj(s(2)))")
		}
		
		
		"handle weird whitespace" in{
			expectsEqual("""
			a (  b  (  c  )    )  
			      	d
			""", "a(b(c) d)")
		}
		
		"put indentation before colon overrun" in{
			expectsEqual("""
			f
				a:
				b
			""", "f(a b)")
		}
		
		"interpret windows style line endings correctly" in{
			expectsEqual("\nf\n\ta\r\n\tb", "f(a b)") //apparently multiline strings in scala ignore escape sequences.
		}
		
		"properly handle escaped unicode snowman" in{
			expectsEqual("""
			f "MERRY HOLIDAY \h\h\h\h\h\h
			""", "f(\"MERRY HOLIDAY ☃☃☃☃☃☃\")")
		}
		
		"be able to output as a json array" in{
			val pinp = gp.parse("f(p l a(9 32 87").success.value
			Termpose.asJsonString(pinp) === ("[\"f\", \"p\", \"l\", [\"a\", \"9\", \"32\", \"87\"]]")
		}
		
		"put quoted terms adjacent to the term before them inside former term" in {
			expectsEqual("""
				f do"alll"
					daba ab"moko
					daba dke:ab"moko
			""", "f(do(alll) daba(ab(moko)) daba(dge(ab(moko))))")
		}
		
		"chain colons according to the spec" in {
			expectsEqual("""
				a:b:c:d:
					d
					d
			""", "a(b(c(d(d d))))")
		}
		
		"kinda mostly translate termpose to XML" in {
			Termpose.translateTermposeToSingleLineXML("html(head(style(.\"body{background-color:black;color:black}\")) body(div(-id(dersite))))").success.value === "<html><head><style>body{background-color:black;color:black}</style></head><body><div id=\"dersite\"></div></html>"
		}
	}
	
	
	// this file used to have no dependencies. Then I started using sbt + maven and it made getting nice and bloated so easy that I just don't care any more. Praise be to the march unto modernity!
	// val p = new Termpose.Parser
	// for(i <- 0 until allTests.length){
	// 	val test = allTests(i)
	// 	try{
	// 		p.parse(test.input) match{
	// 			case Success(s)=>
	// 				val str = s.map { _.toString } .reduce { (l,r) => l ++ " " ++ r }
	// 				if(str equals test.output)
	// 					println(i + ": success")
	// 				else{
	// 					val midstring = test.name match{
	// 						case Some(quality)=> ": failure to ensure that {"+quality+"}, expected  "
	// 						case None => ": failure, expected  "
	// 					}
	// 					println(Console.RED + i + midstring + test.output +".  got  "+str+"."+Console.RESET)
	// 				}
	// 			case Failure(e)=>
	// 				println(Console.RED + i + ": failure, parser says:")
	// 				println("  "+e.getMessage+Console.RESET)
	// 		}
	// 	}catch{
	// 		case e:Exception =>
	// 			val name = test.name match{
	// 				case Some(quality)=> "critical error concerning {"+quality+"}. "
	// 				case None => "critical error. "
	// 			}
	// 			println(Console.MAGENTA+ name +e.toString+Console.RESET)
	// 	}
	// }
}