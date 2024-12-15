# woodslist spec

Concerning [woodslist](./woodslist.md).

# claraifying examples

`("quoted"then)` goes to `list(atom("quoted") atom("then"))`

`("first""second")` goes to `list(atom("first") atom("second"))`.

So does `"first""second"` (without parens). If there are multiple root items in a file/string being parsed, it produces a list.

```
(a "
  first
  second
")
```
goes to `list(atom(a) atom("\n  first\n  second\n"))`

You should now fully understand how to parse woodslist. Afaik there are no other counterintuitive cases.

# formal definition of syntax

I'm not sure this is really useful at all, but here's a formal definition of what kinds of strings are allowed which also strongly suggests a particular way of translating that text into structure.

**:keywords**, words to which we assign special definitions, are written with a colon. :keywords will be bolded when defined.

We will name the elements of the syntax, then explain how those elements are mapped by a parsing method into :wood data

"any number of" means zero or more

"some" means one or more



### data

the process of parsing a :file produces a :wood

a **:wood** is either either a :branch or a :leaf. In a Rust API, for instance, it is defined as `enum Wood { Branchv(Branch), Leafv(Leaf) }`. In the C API, it is a tagged union

a **:branch** is a sequence type containing any number of :woods

a **:leaf** contains a String



### syntax generating rules

A woodslist file is valid if and only if it can be generated according to these rules.

**:file** = any number of :items, separated by any amount of :whitespace

**:whitespace** = some `space`, `tab`, or :newline

**:newline** = "`\n`", "`\r`", or if a `\r` is immediately followed by a `\n`, this pair of characters constitutes only a single :newline "`\r` `\n`" (it's an ancient standard that, AFAIK, windows still uses)

**:item** = one of :slist, :word, pr :quoted

**:slist** = `(` followed by any :whitespace, :items and :newlines followed by `)`

**:word** = some :letters without :whitespace, :newlines, `(`, `)`, `"`

**:letter** = an :escaped, or any unicode letter except `\`. Includes :newlines

**:quoted** = `"` followed by any number of :letters, then `"`

**:escaped** = `\\`, `\"`, `\n` (newline), `\r` (other newline), `\t` (tab)
