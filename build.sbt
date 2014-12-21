name := "Termpose"

version := "1.0"

sbtVersion := "0.13.1"

libraryDependencies += "com.storm-enroute" %% "scalameter" % "0.6" % "test"

libraryDependencies += "com.fasterxml.jackson.core" % "jackson-databind" % "2.4.4"

libraryDependencies += "org.scalatest" %% "scalatest" % "2.2.1" % "test"

testFrameworks += new TestFramework("org.scalameter.ScalaMeterFramework")

parallelExecution in Test := false


logBuffered := false