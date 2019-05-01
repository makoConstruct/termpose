# definition of syntax and data

this document provides a (mostly) formal definition of the generating rules that define the syntax, and the data that results from the identified syntactical elements

**:keywords**, words to which we assign special definitions, are written with a colon. :keywords will be bolded when mentioned for the first time in the document

we will name the elements of the syntax, then explain how those elements are mapped by a parsing method into **:term** data


### syntax rules

:file = any number of **:lines**

:line = an **:indentation**, optional **:linecontent**, then a **:newline** or `EOF`

:newline = one or more `\n` `\r`

:indentation = any number of `space` `tab`

:linecontent = one or more **:items** separated by **:whitespace**

:whitespace = one or more `space` `tab`

:item = one of **:slist** **:word** **:pair** **:quoted** **:invocation** **:quonvokation**



:slist = `(` followed by any number of :items followed by `)`

:word = some **:letters** other than :whitespace

:pair = a non-:pair :item followed by a `:`, followed by an :item

:quoted = `"` followed by any number of :letters other than a :newline or `EOF`. Can end with a `"`

:invocation = an :item followed immediately by a :slist

:quonvokation = an :item followed immediately by a :quoted



:letter = an **:escaped**, or any unicode letter other than `:` `(` `)` `"` `\n` `\r` `\`

:escaped = `\\`, `\"`, `\n` (newline), `\r` (other newline), `\t` (tab)

**:indental** of a line = the :indental is the set of lines with content that are indented beneath the line. More formally, it is the lines that appear after the head line that have longer :indentation than the head line does, and before the next line with content that has an :indentation that is shorter than or equal to the :indentation of the head line


### misc requirements/exceptions

the first :line with :linecontent must have an indent of zero length

each :indentation must be either prefixed by the :indentation of the previous line, or be the prefix of the :indentation of the next line. This permits common deviations in indenting style, without allowing any inconsistent or misleading :indentation patterns

any :item can be **:interrupted** by a :newline. Through this, :slists will not always close, :quoteds wont always have an end-quote, and :pairs wont always have their second :item. The meaning of :interrupted :items will be explained below



### data as a function of syntax

the process of parsing a **:file** produces a **:term**

a :term is a tagged union of a **:list** type or a string. In the fast Rust API, for instance, a :term is defined with a signature similar to `enum Term { Listv(Vec<Term>), Atomv(String) }`. In the C API, it is a tagged union

a **:list** is a sequence type containing any number of :terms

here, we will describe the function mapping our named syntactical elements to :terms. This function's name will be "data"

data(:file) → the data of the :linecontent of each of the root :lines, that is, of each of the lines with an :indent of zero length

data(:linecontent) → if the :linecontent contains multiple :items, it will result in a :list of the data of those items. If it contains just one item, it will result in just the data of that :item, without putting a list around it. If the line has :indental, wrap the data in a list and add the data of the :indental :linecontents to it and return that resultant list

data(:item) → see the following :item variants

data(:slist) → a list with the data of each contained :item. If :interrupted, the slist will contain any :indental

data(:word) → a string

data(:pair) → a :list containing the data of both of the :items. The :list of a :pair interrupted by a `)` will close with just one :term. The :list of the :pair left open and :interrupted by the line end will contain the :indental.

data(:quoted) → a string. If :interrupted, and the string contains non-:whitespace content, then the string will end normally at the end of the line. If there is nothing, or if there is only whitespace, the whitespace will be stripped out and if the line has :indental, it will be parsed as a **:multilinestring**

data(:multilinestring) → Once a non-:whitespace character is encountered on the first indented line of the :multilinestring, the **:mindnt** of the multiline string is set as the preceeding :whitespace string. On each line within the multiline string, this :mindnt is repeated. Everything beyond each :mindnt is included in the resultant string. The string will not be terminated with a newline unless a final empty :mindnt is present

data(:invocation) → the data of the :item part at the head of the invocation will be inserted as the first of the immediately following data(:slist) :list

data(:quonvokation) → a :list containing the data(:item) and the data(:quoted)
