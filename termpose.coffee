asList = (ps)->
	root = []
	for term in ps
		if term.tail
			root.push [root.term].concat asList(term.tail)
		else
			root.push [root.term]
	root

mightBeReservedChar = (c)-> c.charCodeAt(0) < 59 #heuristic, no false negatives
escapeSymbol = (str)->
	sb = ''
	for c in str
		switch c
			when '\\'
				sb += '\\\\'
			when '"'
				sb += '\\"'
			when '\n'
				sb += '\\n'
			when '\r'
				sb += '\\r'
			else
				sb += c
	sb

class Term
	constructor: (@v, @tail = [])->
	getMap: ->
		if !@asMap
			mappables = []
			for t in @tail
				if t.tail.length >= 1 and t.tail[0].tail.length == 0
					mappables.push t
			@asMap = new Map mappables.map (t)-> [t.v, t.tail[0].v]
		@asMap
	get: (k)-> @getMap().get(k)
	at: (i)-> @tail[i]
	stringifySymbol: ->
		needsBeEscaped = false
		for c in @v
			needsBeEscaped = mightBeReservedChar(c) &&
				switch c
					when ' ' then true
					when '(' then true
					when ':' then true
					when '\t' then true
					when '\n' then true
					when '\r' then true
					when ')' then true
					else false
			break if needsBeEscaped
		if needsBeEscaped
			'"'+(escapeSymbol @v)+'"'
		else
			@v
	toString: ->
		if @tail.length
			@stringifySymbol()+ '(' + @tail.map((t)-> t.toString()).join(' ') + ')'
		else
			@stringifySymbol()

class Parser
	constructor: -> @init()
	init: ->
		@stringBuffer = ""
		@foremostSymbol = ""
		@salientIndentation = ""
		@resultSeq = []
		@headTerm = null
		@foremostTerm = null
		@parenTermStack = []
		@tailestTermSequence = @resultSeq
		@parenParentStack = []
		@line = 0 
		@column = 0
		@index = 0
		@currentMode = null
		@modes = []
		@indentStack = [[0, @resultSeq]]
		@previousIndentation = ""
		@previousTailestTermSequence = @resultSeq
		@multiLineIndent = ""
		@mlIndent = null
		@containsImmediateNext = null
	
	
	#general notes:
	#the parser consists of a loop that pumps chars from the input string into whatever the current Mode is. Modes jump from one to the next according to cues, sometimes forwarding the cue onto the new mode before it gets any input from the input loop. There is a mode stack, but it is rarely used. Modes are essentially just a Char => Unit (named 'PF', short for Processing Funnel(JJ it's legacy from when they were Partial Functions)). An LF ('\n') is inserted artificially at the end of input to ensure that line end processes are called. CR LFs ("\r\n"s) are filtered to a single LF (This does mean that a windows formatted file will have unix line endings when parsing multiline strings, that is not a significant issue, ignoring the '\r's will probably save more pain than it causes and the escape sequence \r is available if they're explicitly desired).
	
	startsWith = (string, prefix)->
		if prefix.length > string.length
			true
		else
			i=0
			while i < prefix.length
				eqs = string.charCodeAt(i) == prefix.charCodeAt(i)
				return false if !eqs
				i += 1
			true
	
	interTerm: (symbol)->
		t = new Term(symbol, [])
		t.line = @line
		t.column = @column
		t
	transition: (nm)-> @currentMode = nm
	pushMode: (nm)->
		@modes.push @currentMode
		@transition(nm)
	popMode: ->
		@currentMode = @modes.pop()
	die: (message, l = @line, c = @column)-> throw new Error("line:"+l+" column:"+c+", no, bad: "+message)
	finishTakingSymbol: =>
		@foremostSymbol = @stringBuffer
		@stringBuffer = ''
		@foremostSymbol
	finishTakingTermAndAttach: =>
		@foremostTerm = @interTerm(@finishTakingSymbol())
		if @containsImmediateNext
			@containsImmediateNext.tail.push @foremostTerm
			@containsImmediateNext = null
		else if @headTerm == null
			@headTerm = @foremostTerm
			@tailestTermSequence = @foremostTerm.tail
		else if @parenTermStack.length == 0
			@headTerm.tail.push @foremostTerm
		else
			@parenTermStack[0].tail.push @foremostTerm
	finishTakingIndentation: =>
		@salientIndentation = @stringBuffer
		@stringBuffer = ''
		@salientIndentation
	finishLine: =>
		#now knit this line's data into the previous
		if @salientIndentation.length > @previousIndentation.length
			if ! startsWith @salientIndentation, @previousIndentation
				@die("inconsistent indentation at", @headTerm.line, @headTerm.column)
			@indentStack.push([@salientIndentation.length, @previousTailestTermSequence])
		else if @salientIndentation.length < @previousIndentation.length
			if ! startsWith @previousIndentation, @salientIndentation
				@die("inconsistent indentation", @headTerm.line, @headTerm.column)
			#pop to enclosing scope
			while @indentStack[0][0] > @salientIndentation.length
				if @indentStack[0][0] < @salientIndentation.length
					@die("inconsistent indentation, sibling elements have different indentation", @headTerm.line, @headTerm.column)
				@indentStack.pop()
		
		@indentStack[@indentStack.length-1][1].push @headTerm
		
		@containsImmediateNext = null
		@previousIndentation = @salientIndentation
		@previousTailestTermSequence = @tailestTermSequence
		@tailestTermSequence = @resultSeq
		@parenTermStack = []
		@headTerm = null
		@foremostTerm = null
	finishLineNormally: => #where as finishing abnormally is what multiline reading does because by the time it finishes it has already eaten the indentation and it doesn't ask whether the parenTermStack's empty
		# if(parenTermStack.isEmpty){
			@finishLine()
			@transition(@eatingIndentation)
		# }else{
		# 	die("line end before closing paren")
		# }
	closeParen: =>
		if @parenTermStack.length == 0
			@die("unbalanced paren")
		@parenTermStack.pop()
		@transition(@seekingInLine)
	openParen: =>
		@parenTermStack.push @foremostTerm
		@transition(@enteringParen)
	eatingIndentation: (c)=> switch c
		when '\n'
			stringBuffer = ''
		when ':'
			@die("colon not allowed here")
		when '('
			@die("what are you openingggg")
		when ')'
			@die("what are you closingggg")
		when '"'
			@finishTakingIndentation()
			@transition(@startingToTakeQuoteTerm)
		when ' '
			@stringBuffer += c
		when '\t'
			@stringBuffer += c
		else
			@finishTakingIndentation()
			@transition(@takeTerm)
			@takeTerm(c)
	seekingInLine: (c)=> switch c
		when '('
			@openParen()
		when ')'
			@closeParen()
		when ':'
			@containsImmediateNext = @foremostTerm
			@transition(@seekingColonBuddy)
		when ' ' then
		when '\t' then
		when '\n'
			@finishLineNormally()
		when '"'
			@transition(@startingToTakeQuoteTerm)
		else
			@transition(@takeTerm)
			@takeTerm(c)
	takeTerm: (c)=>
		breakToSeekingInline = =>
			@finishTakingTermAndAttach()
			@transition(@seekingInLine)
			@seekingInLine(c)
		switch c
			when ' ' then breakToSeekingInline()
			when '\t' then breakToSeekingInline()
			when ':' then breakToSeekingInline()
			when '\n' then breakToSeekingInline()
			when '(' then breakToSeekingInline()
			when ')' then breakToSeekingInline()
			when '"'
				@finishTakingTermAndAttach()
				containsImmediateNext = @foremostTerm
				@transition(@startingToTakeQuoteTerm)
			else
				@stringBuffer += c
	enteringParen: (c)=> switch c
		when ' ' then
		when '\t' then
		when ':'
			@die("colon wat")
		when '('
			@die("paren opens nothing")
		when ')'
			@closeParen()
		when '"'
			@transition(@takingSingleLineQuoteTerm)
		when '\n'
			@die("newline before paren completion (parens are only for single lines)")
		else
			@transition(@takeTerm)
			@takeTerm(c)
	seekingColonBuddy: (c)=> switch c
		when ' ' then
		when '\t' then
		when ':'
			@die("double colon wat")
		when '\n'
			@tailestTermSequence = @containsImmediateNext.tail
			@finishLineNormally
		when '('
			#fine whatever I'll just forget the colon ever happened
			@openParen()
		when ')'
			@die("closebracket after colon wat")
		when '"'
			@transition(@startingToTakeQuoteTerm)
		else
			@transition(@takeTerm)
			@takeTerm(c)
	startingToTakeQuoteTerm: (c)=> switch c
		when '"'
			@finishTakingTermAndAttach()
			@transition(@seekingInLine)
		when '\n'
			if !@parenTermStack.length == 0
				die("newline with unbalanced parens(if you want multiline string syntax, that's an indented block under a single quote)")
			@stringBuffer = ''
			@transition(@multiLineFirstLine)
		when ' '
			@stringBuffer += c
		when '\t'
			@stringBuffer += c
		else
			@transition(@takingSingleLineQuoteTerm)
			@takingSingleLineQuoteTerm(c)
	takingSingleLineQuoteTerm: (c)=> switch c
		when '"'
			@finishTakingTermAndAttach()
			@transition(@seekingInLine)
		when '\\'
			@pushMode(@takingEscape)
		when '\n'
			@finishTakingTermAndAttach()
			@transition(@seekingInLine)
			@seekingInLine('\n')
		else
			@stringBuffer += c
	#history story: takingQuoteTermThatMustTerminateWithQuote was an artifact from when line dies wern't allowed during parens, it was really just the above with a dieer response to '\n's instead of a handler. We now handle such events reasonably.
	takingEscape: (c)=>
		switch c
			when 'n' then @stringBuffer += '\n'
			when 'r' then @stringBuffer += '\r'
			when 't' then @stringBuffer += '\t'
			when 'h' then @stringBuffer += '☃'
			else @stringBuffer += c
		@popMode()
	multiLineFirstLine: (c)=> switch c
		when ' '
			@multiLineIndent += c
		when '\t'
			@multiLineIndent += c
		else
			@mlIndent = @multiLineIndent
			if @mlIndent.length > @salientIndentation.length
				if startsWith @mlIndent, @salientIndentation
					@multiLineIndent = ''
					@transition(@multiLineTakingText)
					@multiLineTakingText(c)
				else
					@die("inconsistent indentation")
			else
				@die("multiline string must be indented")
	multiLineTakingIndent: (c)=>
		if c == ' ' or c == '\t'
			@multiLineIndent += c
		else
			nin = @multiLineIndent
			@multiLineIndent = ''
			if nin.length >= @mlIndent.length
				if startsWith nin, @mlIndent
					@stringBuffer += '\n' #add the newline from the previous line, safe in the knowledge that the multiline string continues up to and beyond that point
					if nin.length > @mlIndent.length
						@stringBuffer += @mlIndent.substring(nin.length, @mlIndent.length)
					@transition(@multiLineTakingText)
					@multiLineTakingText(c)
				else
					@die("inconsistent indentation")
			else
				if startsWith @mlIndent, nin
					#breaking out, returning to usual scanning
					@finishTakingTermAndAttach()
					@finishLine()
					@salientIndentation = nin
					@transition(@takeTerm)
					@takeTerm(c)
				else
					@die("inconsistent indentation")
	multiLineTakingText: (c)=> switch c
		when '\n'
			#stringBuffer += '\n'   will not add the newline until we're sure the multiline string is continuing
			@transition(@multiLineTakingIndent)
		else
			@stringBuffer += c
	
	parse: (s)-> #throws a parsing exception if failure
		#pump characters into the mode of the parser until the read head has been graduated to the end
		@transition(@eatingIndentation)
		while @index < s.length
			c = s[@index]
			if c == '\r' #handle windows' deviant line endings
				if s[@index+1] == '\n'
					@index += 1
				c = '\n'
			@currentMode(c)
			@index += 1
			if c == '\n'
				@line += 1
				@column = 0
			else
				@column += 1
		if s[s.length-1] != '\n'
			@currentMode('\n') #simulated final newline
		res = @resultSeq
		@init()
		res

parseTermpose = (s)-> #Term
	new Parser().parse(s)


exports.parseTermpose = parseTermpose
exports.Term = Term
exports.Parser = Parser