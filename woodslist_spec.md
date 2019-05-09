# woodslist spec

**:keywords**, words to which we assign special definitions, are written with a colon. :keywords will be bolded when mentioned for the first time in the document

We will name the elements of the syntax, then explain how those elements are mapped by a parsing method into **:wood** data

"any number of" means zero or more

"some things" means one or more



### data

the process of parsing a **:file** produces a :wood

a :wood is either either a **:branch** or a **:leaf**. In a Rust API, for instance, it is defined as `enum Wood { Branchv(Branch), Leafv(Leaf) }`. In the C API, it is a tagged union

a :branch is a sequence type containing any number of :woods

a :leaf contains a String



### elements of syntax

:file = any number of **:items**, separated by **:whitespace**

:whitespace = some `space`, `tab`, or **:newline**

:newline = "`\n`", "`\r`", or if a `\n` is immediately followed by a `\r`, this pair of characters constitutes only a single :newline "`\n``\r`" (because of a weird convention windows has)

:item = one of **:slist** **:word** **:quoted**



:slist = `(` followed by any number of :whitespaces, :items and :newlines followed by `)`

:word = some **:letters** without :whitespace, :newlines, `(`, `)`, `"`

:letter = an **:escaped**, or any unicode letter except `\`. Includes :newlines

:quoted = `"` followed by any number of :letters, then `"`

:escaped = `\\`, `\"`, `\n` (newline), `\r` (other newline), `\t` (tab)



### term data defined as a function of syntactical elements

data(:file) → parsing will return a :branch containing data(:items), of all :items that are not contained within another :item

data(:item) → see the following :item types

data(:slist) → a :branch containing the data of each contained sub-:item

data(:word) → a leaf string. Each :escaped is translated, and each :newline resolves as a `\n` regardless of what went in.

data(:quoted) → a leaf string. If the string starts with a :newline, it drops the :newline (the reason is, this allows you to start a multiline string on the first column, instead of putting the first line out wherever the string starts, which looks weird)
