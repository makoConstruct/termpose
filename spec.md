# termpose spec

**:keywords**, words to which we assign special definitions, are written with a colon. :keywords will be bolded when mentioned for the first time in the document

We will name the elements of the syntax, then explain how those elements are mapped by a parsing method into **:term** data



### data

the process of parsing a **:file** produces a **:term**

a :term is either either a **:list** or a string. In the fast Rust API, for instance, it is defined as `enum Term { Listv(List), Atomv(Atom) }`. In the C API, it is a tagged union

a **:list** is a sequence type containing any number of :terms





### elements of syntax

:file = one or more **:lines**

:line = an **:indentation**, optional **:linecontent**, then a **:newline** or the end of the :file

:newline = one or more `\n` `\r`

:indentation = any number of `space` `tab`

:linecontent = one or more **:items** separated by **:whitespace**

:whitespace = one or more `space` `tab`

:item = one of **:slist** **:word** **:pair** **:quoted** **:invocation** **:quonvokation**



:slist = `(` followed by any number of :items followed by `)`

:word = some **:letters** other than :whitespace

:pair = a non-:pair :item followed by a `:`, followed by an :item

:quoted = `"` followed by any number of :letters other than a :newline or an `EOF`. Can end with a `"`

:invocation = an :item followed immediately by a :slist

:quonvokation = an :item followed immediately by a :quoted



:letter = an **:escaped**, or any unicode letter other than `:` `(` `)` `"` `\n` `\r` `\`

:escaped = `\\`, `\"`, `\n` (newline), `\r` (other newline), `\t` (tab)

**:indental** of a line = the :indental is the set of lines with content that are indented beneath the line. More formally, it is the lines that appear after the head line that have longer :indentation than the head line does, and before the next line with content that has an :indentation that is shorter than or equal to the :indentation of the head line


### misc requirements/exceptions

each :indentation must be either prefixed by the :indentation of the previous line, or be the prefix of the :indentation of the next line. This permits deviations in indenting style that are common, without allowing any inconsistent or misleading :indentation patterns

during **:multilinestrings**, the :indentation is set by the first line of the multiline, then remains at that length until the :multilinestring ends (that is, when a line with content and a shorter :indentation is encountered.)

any item can be **:interrupted** by a :newline. Through this, :slists will not always close, :quoteds wont always have an end-quote, and :pairs wont always have their second :item. The meaning of :interrupted :items will be explained below



### term data defined as a function of syntactical elements

:file → parsing will return a :list with a :term for each :linecontent at root level (that is, each :line having an indent of zero length)

:linecontent → if the :linecontent contains multiple items, it will result in a :list of the :terms of those items. If it contains just one item, it will result in the term of that :item. If the line has :indental, wrap the results in a list and add the :terms of the :indental lines to it and return that resultant list

:item → see the following :item types

:slist → a :list containing a :term for each :item. If :interrupted, the slist will contain any :indental

:word → a string

:pair → a :list containing two :terms, one for each :item. If :interrupted after the `:` and before the second term, the :list will contain the :indental

:quoted → a string. If :interrupted, and the string contains non-:whitespace content, then the string will end normally at the end of the line. If there is only whitespace, the whitespace will be stripped out and if the line has :indental, it will be parsed as a **:multilinestring**

:multilinestring → Once a non-:whitespace character is encountered on the first indented line of the :multilinestring, the **:mindnt** of the multiline string is set as the preceeding :whitespace string. On each line within the multiline string, this :mindnt is repeated. everything beyond each :mindnt is included in the resultant string. The string will not be terminated with a newline unless a final empty :mindnt is stated.

:invocation → the :item part at the head of the invocation will be inserted as the first of the immediately following :slist part

:quonvokation → a :list containing the :term of the :item and the :term of the :quoted
