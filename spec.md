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

here, we will describe the function mapping our named syntactical elements to :terms. This funciton's name will be "data". We will use member syntax to refer to the parts of the syntactical elements. EG, :line.:linecontent refers to the linecontent syntactical element that must be present in each :line. If two syntactical elements of the same type are present, they will be numbered, for instance, the two :items of a :pair will be called :pair.:item_0 and :pair.:item_1

data(:file) → a :list  data(:linecontent) at root level (that is, each :line having an indent of zero length)
```
:file.:lines.filter(l → l.:indent.len = 0).map(l → data(l.:linecontent)) as :list
```

data(:linecontent) → if the :linecontent contains multiple :items, it will result in a :list of the data of those items. If it contains just one item, it will result in just the data of that :item, without putting a list around it. If the line has :indental, wrap the data in a list and add the data of the :indental :linecontents to it and return that resultant list
```
if :linecontent.:items.len = 1
  data(:linecontent.:items[0])
else
  :linecontent.:items.map(data) as :list
```

data(:item) → see the following :item variants

data(:slist) → a list with the data of each contained :item. If :interrupted, the slist will contain any :indental
```
:slist.:items.map(data) as :list
```

data(:word) → a string

data(:pair) → a :list containing the data of each of the two :items. If :interrupted after the `:` (if there is no second term), the resultant :list will contain the :indental
```
:list(data(:pair.:item_0) data(:pair.:item_1))
```

data(:quoted) → a string. If :interrupted, and the string contains non-:whitespace content, then the string will end normally at the end of the line. If there is nothing, or if there is only whitespace, the whitespace will be stripped out and if the line has :indental, it will be parsed as a **:multilinestring**

data(:multilinestring) → Once a non-:whitespace character is encountered on the first indented line of the :multilinestring, the **:mindnt** of the multiline string is set as the preceeding :whitespace string. On each line within the multiline string, this :mindnt is repeated. Everything beyond each :mindnt is included in the resultant string. The string will not be terminated with a newline unless a final empty :mindnt is present

data(:invocation) → the data of the :item part at the head of the invocation will be inserted as the first of the immediately following data(:slist) :list
```
concat(:list(data(:invocation.:item)) data(:invocation.:slist))
```

data(:quonvokation) → a :list containing the data(:item) and the data(:quoted)
```
:list(data(:quonvokation.:item) data(:quonvokation.:quoted))
```
