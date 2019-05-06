# nakedlist spec

**:keywords**, words to which we assign special definitions, are written with a colon. :keywords will be bolded when mentioned for the first time in the document

We will name the elements of the syntax, then explain how those elements are mapped by a parsing method into **:term** data

"any number of" means zero or more

"some things" means one or more



### data

the process of parsing a **:file** produces a :term

a :term is either either a **:list** or a string. In a Rust API, for instance, it is defined as `enum Term { Listv(List), Atomv(Atom) }`. In the C API, it is a tagged union

a :list is a sequence type containing any number of :terms





### elements of syntax

:file = any number of **:lines**

:line = an **:indentation**, optional **:linecontent**, then a **:newline** or `EOF`

:newline = a `\n` and an optional `\r`

:indentation = any number of `space` or `tab`

:linecontent = some **:items** separated by **:whitespace**

:whitespace = some `space` or `tab`

:item = one of **:slist** **:word** **:quoted**



:slist = `(` followed by any number of :items, :whitespaces, :newlines followed by `)`

:word = some **:letters** other than :whitespace

:quoted = `"` followed by any number of :letters and :newlines, then `"` or `EOF`

:letter = an **:escaped**, or any unicode letter other than `:` `(` `)` `"` `\n` `\r` `\`

:escaped = `\\`, `\"`, `\n` (newline), `\r` (other newline), `\t` (tab)

**:indental** of a line = the :indental is the set of lines with content that are indented beneath the line. More formally, it is the lines that appear after the head line that have longer indentation than the head line does, and before the next line with content that has an indentation that is shorter than or equal to the indentation of the head line


### misc requirements/exceptions

each :indentation must be either prefixed by the :indentation of the previous :line, or itself prefix the :indentation of the next :line. This allows common deviations in indenting without allowing any inconsistent or misleading :indentation patterns



### term data defined as a function of syntactical elements

data(:file) → parsing will return a :list containing data(:linecontent) at root level (that is, it maps over the :lines that have an indent of zero length)

data(:linecontent) → the content of the :linecontent is the sequence of the data of the :items contained in it, and the data of each :indental, if there are any. If this content contains multiple items, the result is a :list of the data of those items. If it contains just one item, it will result in just the data of that item without an encompassing list

data(:item) → see the following :item types

data(:slist) → a :list containing the data of each contained :item

data(:word) → a string

data(:quoted) → a string. If the string starts with a :newline, it drops the :newline (the reason is, this allows you to start a multiline string on the first column, instead of putting the first line out wherever the string starts, which looks weird)
