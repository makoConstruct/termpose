object App{
	def main(args:Array[String]){
		import util.{Try, Success, Failure}
		val tests = Array(
		"""
		a
		""", "a",
		
		"""
		a(b(c(d)))
		""", "a(b(c(d)))",
		
		"""
		a (  b  (  c  )    )  
		      	d
		""", "a(b(c) d)",
		
		"""
		a b
		""", "a(b)",
		
		"""
		a b c
		""", "a(b c)",
		
		"""
		a
		  b
		  c
		""", "a(b c)",
		
		"""
		a b
		  c
		""", "a(b c)",
		
		"""
		Aaron b""", "Aaron(b)",
		
		"""
		Aaron
			because
			always
				keep
				dogs
					in pens
			keep
			swimming
		""", "Aaron(because always(keep dogs(in(pens))) keep swimming)",
		
		"""
		Oglomere
		Oglomore
		""", "Oglomere Oglomore",
		
		"""
		"horrid clown"
			make "my day"
			horrid clown
				jp awesome:ross
		""", "\"horrid clown\"(make(\"my day\") horrid(clown jp(awesome(ross))))",
		
		"""
		how about a "
			~M~U~L~T~I~L~I~N~E~
			~S~T~R~I~N~G~
		//"YOU LIKE?"
		""", "how(about a \"~M~U~L~T~I~L~I~N~E~\\n~S~T~R~I~N~G~\") //(\"YOU LIKE?\")",
		"""
		hobobo
			green witches:
				glon
				mora
				bonell
			emero
		""", "hobobo(green(witches(glon mora bonell)) emero)",
		"""a(bb(aod(igjd(ss
			ojfjf
			jjj(s:
				2""", "a(bb(aod(igjd(ss))) ojfjf jjj(s(2)))",
		"""
		f
			a:
			b
		""", "f(a b)"
		)

		var i = 0
		val p = new Termpose.Parser
		while(i < tests.length){
			try{
				p.parse(tests(i)) match{
					case Success(s)=>
						val str = s.map { _.toString } .reduce { (l,r) => l ++ " " ++ r }
						if(str equals tests(i+1))
							println(i/2 + ": success")
						else
							println(Console.RED + i/2 + ": failure, expected  "+tests(i+1)+".  got  "+str+"."+Console.RESET)
					case Failure(e)=>
						println(Console.RED + i/2 + ": failure, parser says:")
						println("  "+e.getMessage+Console.RESET)
				}
			}catch{
				case e:Exception =>
					println(Console.MAGENTA + i/2 + ": critical error. "+e.toString+Console.RESET)
			}
			i += 2
		}
	}
}