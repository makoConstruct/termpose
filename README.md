##termpose
A dead simple, pretty, machine-readable format for trees of strings.

###it's like s-expressions, but not
Termpose is extremely regular, but instead of basing all of its data on lists, termpose speaks of terms and their associated sequences, (EG `array(1 2 3 4)`, 'array' is the term, the numbers are the sequence), under the expectation that the host language will interpret sequences as being contained or in some way pertaining to their enclosing term. This is a reasonable expectation, considering that even Lisp gives special meaning to the first term of its lists in the majority of cases. Termpose resolves to an AST, rather than a list of lists. This evenue also eliminates the edge cases of termless empty lists; every entity in the tree has a string at its head.

Termpose pays attention to indentation. Expressions like
```
a
	b
	c
		d
	e
```
will be equivalent to `a(b c(d) e)`. This allows you to lay out your termpose with a minimum of friction and aesthetic noise.

Termpose tries to keep out of your namespace, attributing its own meanings to `":()`, but leaving `\/?-+=[]*&^%$#@!\`~;'.,<>` for your metalanguage to define as it pleases.

syntax by example
```
A B
#A contains B
A B C
#A contains B and C
A
  B
  C
#equivalent
A(B C)
#equivalent
A(B C)
  D E
#A contains B, C and D. D contains E.
A(B C) D
  E
#equivalent
A B C D
  E
#equivalent
A B:C D:E
#A contains B and D, but C and E are contained by their colon buddy, B and D respectively
A:B:C:D:E
#each contains the next
A "a string"
#"a string" is contained within "A" ("A" == A, all symbols are just strings)
A "a string
#equivalent. Closing quote isn't needed if the line ends. What else could such a statement be intended to mean?
A"a string
#equivalent
A "
  a string that
  is multiline
#A contains "a string that↩is multiline"
```
<!-- A B, C D, E F
#A(B C(D E(F))). Commas start a new term within the line. -->

potential translation scheme:
```coffeescript
parse("""
abacus opens
horse
to
  1
  2
  3
  4
  5
whaia(gloria(54 90 142))
goings
  comings("7  9")
  aways(90)
""")
#json:
#[["abacus", ["opens"]], "horse", ["to", ["1", "2", "3", "4", "5"]], ["whaia", ["gloria", ["54", "90", "142"]]], "goings", ["comings", ["7", "9"]], ["aways", ["90"]]]
#lisp:
#(('abacus 'opens) 'horse ('to '1 '2 '3 '4 '5) ('whaia ('gloria ('54 '90 '142))) 'goings ('comings ('7 '9)) ('aways '90))
```

What if I defined a programming language for the termpose syntax?
```
let i 0
while <(i 5)
  print s:Hello
  println s" worldly people
  = i +(i 1)

>Hello worldly people
>Hello worldly people
>Hello worldly people
>Hello worldly people
>Hello worldly people

a or b or c or d or e
//"doesn't work
or(a or(b or(c or(d e))))
//"would work but is unsightly
or: a or: b or: c or: d e
//"doesn't work in the present termpose standard, but it would be kind of nice if it meant what it's supposed to mean..
or a, or b, or c, or d, e
//"is how coffeescript would put it
or a b c:
  open
// "
```

here's me making a start on defining a universal type system:
```termpose
meta
  description "
    notes on metalanguage:
    // is a comment
    - is a list element

type
  name Vessel
  type_parameters E
  mutates
  subscribable
  //"makes for reactive streams, has a current state, state changes, you can subscribe to be notified of changes (this is the point of the thing)
  operation
    name place
    in type:E
  property
    name value
    type Option:E
    //"will return None if queried prior to any running of place()
    
type
  name Number
  constant
  operation
    name add
    in name:other type:Number
    out type:Number
  operation
    name subtract
    params name:other type:Number
    out:Number
  operation name:add other:Number out:Number
      
type
  name Integer
  serves_as Number
type
  name U32
  //"32 bit, signed, little-endian Integer
  serves_as Integer
type
  name U64
  
  ... grossly unfinished but you get the point
```

What would termpose imitating json look like?

```javascript
{
  "highlight_line": true,
  "ignored_packages":
  [
    "Floobits",
    "SublimeLinter",
    "Vintage"
  ],
  "indent_guide_options":
  {
    "draw_active": true
  },
  "overlay scroll bars": "enabled",
  "show tab close buttons": false,
  "tab_size": 2,
  "theme": "Spacegray.sublime-theme",
  "word_wrap": "true"
}

```
```termpose
//"minified
'highlight_line true
'ignored_packages ar('Floobits 'SublimeLinter 'Vintage)
'indent_guide_options('draw_active:true)
"'overlay scroll bars" 'enabled
"'show tab close buttons" false
'tab_size 2
'theme 'Spacegray.sublime-theme
'word_wrap true

//"pretty printed
'highlight_line true
'ignored_packages
  - 'Floobits
  - 'SublimeLinter
  - 'Vintage
'indent_guide_options
  'draw_active true
"'overlay scroll bars" 'enabled
"'show tab close buttons" false
'tab_size 2
'theme 'Spacegray.sublime-theme
'word_wrap true
```
I think that works out very well. Note that `'`s are required for all string keys, more so than in json, in order to differentiate them from primitives like `true`, `2`, and `ar` terms. `"`s alone would not do this, as, once parsed through the termpose filter, there is no perceptible difference between a `"true"` and a `true`.

Caveat: Does not preserve the difference between `"a"` and `'a'`. It also cannot be formatted as a single line due to the unnamed but implicit root element
