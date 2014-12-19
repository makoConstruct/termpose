import collection.mutable.ArrayBuffer
import util.{Try, Success, Failure}
object App{
	val allTests = new ArrayBuffer[Test]
	case class Test(name:Option[String], input:String, output:String)
	def expectsEqual(input:String, output:String) = allTests += Test(None, input, output)
	def expectsEqual(name:String, input:String, output:String) = allTests += Test(Some(name), input, output)
	def main(args:Array[String]){
		expectsEqual("""
		a
		""", "a")
		
		expectsEqual("""
		a(b(c(d)))
		""", "a(b(c(d)))")
		
		expectsEqual("""
		a (  b  (  c  )    )  
		      	d
		""", "a(b(c) d)")
		
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
		expectsEqual("indentation is important.", """
		f
			a:
			b
		""", "f(a b)")
		expectsEqual("windows style line endings are interpreted properly", "\nf\n\ta\r\n\tb", "f(a b)") //apparently multiline strings in scala ignore escape sequences.
		expectsEqual("snowman escape sequence correctly produces a unicode snowman", """
		f "merry christmas, every one \h\h\h\h\h\h oxoxoxox
		""", "f(\"merry christmas, every one ☃☃☃☃☃☃ oxoxoxox\")")
		val p = new Termpose.Parser
		for(i <- 0 until allTests.length){
			val test = allTests(i)
			try{
				p.parse(test.input) match{
					case Success(s)=>
						val str = s.map { _.toString } .reduce { (l,r) => l ++ " " ++ r }
						if(str equals test.output)
							println(i + ": success")
						else{
							val midstring = test.name match{
								case Some(quality)=> ": failure to ensure that {"+quality+"}, expected  "
								case None => ": failure, expected  "
							}
							println(Console.RED + i + midstring + test.output +".  got  "+str+"."+Console.RESET)
						}
					case Failure(e)=>
						println(Console.RED + i + ": failure, parser says:")
						println("  "+e.getMessage+Console.RESET)
				}
			}catch{
				case e:Exception =>
					val name = test.name match{
						case Some(quality)=> "critical error concerning {"+quality+"}. "
						case None => "critical error. "
					}
					println(Console.MAGENTA+ name +e.toString+Console.RESET)
			}
		}
	}
}