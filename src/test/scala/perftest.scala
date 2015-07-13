import org.scalameter.api._
import com.fasterxml.jackson._
import com.fasterxml.jackson.databind._
import collection.mutable.HashMap
object Termposition extends PerformanceTest.Quickbenchmark{
	val indentedTermpose = """
	vis TodoItemVis(TodoItem)
	StructVis destruct:
		vis TodoItem
			StructVis destructuring(description:Text done:vis:
				CheckBox
				mapEnum(Done:Tick InProgress:Dot Unfinished:Unticked
	"""
	val indtp = Gen.single("indented")(indentedTermpose)
	val parsedTermpose = Termpose.parse(indentedTermpose).get
	val jsoninp = Gen.single("jsonInline")(parsedTermpose.jsonString)
	println(parsedTermpose.jsonString)
	val undtp = Gen.single("inline")(parsedTermpose.toString)
	var acc = 0
	var doc = 0
	val om = new ObjectMapper
	
	performance of "Termpose" in {
		measure method "parse" in {
			
			using(indtp) in{ str:String =>
				val res = Termpose.parse(str)
			}
			
			using(undtp) in{ str:String =>
				val res = Termpose.parse(str)
			}
		}
	}
	performance of "Jackson" in {
		measure method "parse" in {
			using(jsoninp) in { str:String =>
				val rn = om.readValue(str, classOf[JsonNode])
			}
		}
	}
}