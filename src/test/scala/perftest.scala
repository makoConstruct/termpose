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
	val parser = new Termpose.Parser
	val parsedTermpose = parser.parse(indentedTermpose).get
	val jsoninp = Gen.single("jsonInline")(Termpose.asJsonString(parsedTermpose))
	val undtp = Gen.single("inline")(Termpose.minified(parsedTermpose))
	var acc = 0
	var doc = 0
	val om = new ObjectMapper
	
	performance of "Termpose" in {
		measure method "parse" in {
			
			using(indtp) in{ str:String =>
				val res = parser.parse(str)
				// for {
				// 	rel <- res
				// 	rr <- rel.headOption
				// 	h <- rr.getSub("StructVis")
				// 	vis <- h.getSub("vis")
				// 	s <- vis.getSub("StructVis")
				// 	m <- s.getSub("mapEnum")
				// 	d <- m.getSub("Done")
				// } if(d.term == "Tick") acc += 1
				// doc += 1
			}
			
			using(undtp) in{ str:String =>
				val res = parser.parse(str)
				// for {
				// 	rel <- res
				// 	rr <- rel.headOption
				// 	h <- rr.getSub("StructVis")
				// 	vis <- h.getSub("vis")
				// 	s <- vis.getSub("StructVis")
				// 	m <- s.getSub("mapEnum")
				// 	d <- m.getSub("Done")
				// } if(d.term == "Tick") acc += 1
				// doc += 1
			}
		}
	}
	performance of "Jackson" in {
		measure method "parse" in {
			using(jsoninp) in { str:String =>
				val rn = om.readValue(str, classOf[JsonNode])
				// def term(node:JsonNode):String ={
				// 	if(node.isTextual)
				// 		node.asText
				// 	else if(node.isArray)
				// 		node.get(0).asText
				// 	else
				// 		throw new RuntimeException("nah, can't deal")
				// }
				// def asMap(node:JsonNode):Map[String, JsonNode] = {
				// 	val hm = new HashMap[String, JsonNode]
				// 	var i = 1
				// 	val hms = hm.size
				// 	while(i < hms){
				// 		val el = node.get(i)
				// 		hm(term(el)) = el
				// 		i += 1
				// 	}
				// 	hm.toMap
				// }
				// for {
				// 	h <- asMap(rn).get("StructVis")
				// 	vis <- asMap(h).get("vis")
				// 	s <- asMap(vis).get("StructVis")
				// 	m <- asMap(s).get("mapEnum")
				// 	d <- asMap(m).get("Done")
				// } if(term(d) == "Tick") acc += 1
				// doc += 1
			}
		}
	}
}