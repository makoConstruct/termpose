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
	parse = (str)-> new Parser().parse(str)
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
			@indentStack[@indentStack.length - 1][1].push @foremostTerm
			@tailestTermSequence = @foremostTerm.tail
		else if @parenTermStack.length == 0
			@headTerm.tail.push @foremostTerm
		else
			@parenTermStack[0].tail.push @foremostTerm
	finishTakingIndentation: =>
		@previousIndentation = @salientIndentation
		@salientIndentation = @stringBuffer
		if @salientIndentation.length > @previousIndentation.length
			if ! startsWith @salientIndentation, @previousIndentation
				@die("inconsistent indentation at", @headTerm.line, @headTerm.column)
			@indentStack.push([@salientIndentation.length, @tailestTermSequence])
		else if @salientIndentation.length < @previousIndentation.length
			if ! startsWith @previousIndentation, @salientIndentation
				@die("inconsistent indentation", @headTerm.line, @headTerm.column)
			#pop to enclosing scope
			while @indentStack[@indentStack.length-1][0] > @salientIndentation.length
				if @indentStack[@indentStack.length-1][0] < @salientIndentation.length
					@die("inconsistent indentation, sibling elements have different indentation", @headTerm.line, @headTerm.column)
				@indentStack.pop()
		@stringBuffer = ''
		@salientIndentation
	finishLine: =>
		@containsImmediateNext = null
		@parenTermStack = []
		@headTerm = null
		@foremostTerm = null
	closeParen: =>
		if @parenTermStack.length == 0
			@die("unbalanced paren")
		@parenTermStack.pop()
		@transition(@seekingInLine)
	openParen: =>
		@parenTermStack.push @foremostTerm
		@transition(@enteringParen)
	eatingIndentation: (fileEnd,c)=>
		if !fileEnd
			switch c
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
					@takeTerm(false,c)
	seekingInLine: (fileEnd,c)=> if !fileEnd then switch c
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
			@finishLine()
			@transition(@eatingIndentation)
		when '"'
			@transition(@startingToTakeQuoteTerm)
		else
			@transition(@takeTerm)
			@takeTerm(false,c)
	takeTerm: (fileEnd,c)=>
		if fileEnd
			@finishTakingTermAndAttach()
		else
			breakToSeekingInline = =>
				@finishTakingTermAndAttach()
				@transition(@seekingInLine)
				@seekingInLine(false,c)
			switch c
				when ' ' then breakToSeekingInline()
				when '\t' then breakToSeekingInline()
				when ':' then breakToSeekingInline()
				when '\n' then breakToSeekingInline()
				when '(' then breakToSeekingInline()
				when ')' then breakToSeekingInline()
				when '"'
					@finishTakingTermAndAttach()
					@containsImmediateNext = @foremostTerm
					@transition(@startingToTakeQuoteTerm)
				else
					@stringBuffer += c
	enteringParen: (fileEnd,c)=>
		if fileEnd
			@die "end of file before paren close, this has no meaning and does not make sense."
		else switch c
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
				@takeTerm(false,c)
	seekingColonBuddy: (fileEnd,c)=>
		if fileEnd
			@die "end of file after a colon, this has no meaning and does not make sense"
		else switch c
			when ' ' then
			when '\t' then
			when ':'
				@die "double colon wat"
			when '\n'
				@tailestTermSequence = @containsImmediateNext.tail
				@finishLine()
				@transition(@eatingIndentation)
			when '('
				#fine whatever I'll just forget the colon ever happened
				@openParen()
			when ')'
				@die "closebracket after colon wat"
			when '"'
				@transition(@startingToTakeQuoteTerm)
			else
				@transition(@takeTerm)
				@takeTerm(false,c)
	startingToTakeQuoteTerm: (fileEnd,c)=>
		if fileEnd
			@finishTakingTermAndAttach()
		else switch c
			when '"'
				@finishTakingTermAndAttach()
				@transition(@seekingInLine)
			when '\n'
				@transition(@multiLineFirstLine)
			else
				@transition(@takingSingleLineQuoteTerm)
				@takingSingleLineQuoteTerm(false,c)
	takingSingleLineQuoteTerm: (fileEnd,c)=>
		if fileEnd
			@finishTakingTermAndAttach()
		else switch c
			when '"'
				@finishTakingTermAndAttach()
				@transition(@seekingInLine)
			when '\\'
				@pushMode(@takingEscape)
			when '\n'
				@finishTakingTermAndAttach()
				@transition(@seekingInLine)
				@seekingInLine(fileEnd,'\n')
			else
				@stringBuffer += c
	#history story: takingQuoteTermThatMustTerminateWithQuote was an artifact from when line dies wern't allowed during parens, it was really just the above with a dieer response to '\n's instead of a handler. We now handle such events reasonably.
	takingEscape: (fileEnd,c)=>
		if fileEnd
			@die("invalid escape sequence, no one can escape the end of the file")
		else
			switch c
				when 'h' then @stringBuffer += '☃'
				when 'n' then @stringBuffer += '\n'
				when 'r' then @stringBuffer += '\r'
				when 't' then @stringBuffer += '\t'
				else @stringBuffer += c
			@popMode()
	multiLineFirstLine: (fileEnd,c)=>
		if fileEnd
			@finishTakingTermAndAttach()
		else switch c
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
						@multiLineTakingText(false,c)
					else
						@die("inconsistent indentation")
				else
					@finishTakingTermAndAttach()
					@finishLine()
					#transfer control to eatingIndentation
					@stringBuffer = @mlIndent
					@mlIndent = null
					@transition(@eatingIndentation)
					@eatingIndentation(false,c)
	multiLineTakingIndent: (fileEnd,c)=>
		if fileEnd
			@finishTakingTermAndAttach()
		else if c == ' ' or c == '\t'
			@multiLineIndent += c
			minb = @multiLineIndent
			if minb.length == @mlIndent.length #then we're through with the indent
				if startsWith minb, @mlIndent
					@stringBuffer += '\n' #add the newline from the previous line, safe in the knowledge that the multiline string continues up to and beyond that point
					@multiLineIndent = ''
					@transition(@multiLineTakingText)
				else
					@die("inconsistent indentation")
		else if startsWith @mlIndent, minb
			#breaking out, returning to usual scanning
			@finishTakingTermAndAttach()
			@finishLine()
			@stringBuffer = @mlIndent
			@transition(@eatingIndentation)
			@eatingIndentation(false,c)
		else
			@die("inconsistent indentation")
	multiLineTakingText: (fileEnd,c)=>
		if fileEnd then @finishTakingTermAndAttach()
		else switch c
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
			@currentMode(false,c)
			@index += 1
			if c == '\n'
				@line += 1
				@column = 0
			else
				@column += 1
		@currentMode(true,'☠')
		res = @resultSeq
		@init()
		res

parseTermpose = (s)-> #Term
	new Parser().parse(s)


exports.parseTermpose = parseTermpose
exports.Term = Term
exports.Parser = Parser