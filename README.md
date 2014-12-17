##termpose
A dead simple, pretty, machine-readable format for trees of strings.

###It's arguably more regular than s-expressions
Termpose speaks of terms and their associated sequences, (EG `array(1 2 3 4)`, 'array' is the term, the numbers are the sequence), under the expectation that the host language will interpret sequences as being contained or in some way pertaining to their enclosing term. This is a reasonable expectation, considering that even Lisp gives special meaning to the first term of its lists in the majority of cases. This eliminates the edge cases of termless empty lists and heterogynous element types: every entity in the tree is a term with a string at its head.

Termpose pays attention to indentation. Expressions like
```
a
	b
	c
		d
	e
```
will be equivalent to `a(b c(d) e)`.

Termpose tries to keep out of your namespace, attributing its own meanings to `":()`, but leaving `\/?-+=[]*&^%$#@!\`~;'.,<>` for your domain-specific language to define as it pleases.

syntax by example
```python
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
A(B C
  D E
#equivalent (crazy, right? Thing is: Support for multiline indentation syntax makes the use of paren blocks spanning multiple lines unnecessary, against convention, and thus disrecommended. As a result, it'd be really dumb to require people to close their parens. It'd just be tricky, inhumane pedantry.)
A(B C) D
  E
  F
#A contains everything.
A B C D
  E
  F
#equivalent
A B C D:
  E
  F
#A contains B, C and D. D contains E and F
A B:C D:E
#A contains B and D, but C and E are contained by their colon buddy, B and D respectively
A:B:C:D:E
#each contains the next
A(B(C(D(E))))
#equivalent
A(B(C(D(E
#equivalent
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
#[["abacus", ["opens"]], "horse", ["to", ["1", "2", "3", "4", "5"]], ["whaia", ["gloria", ["54", "90", "142"]]], "goings", ["comings", ["7  9"]], ["aways", ["90"]]]
#lisp:
#'((abacus opens) horse (to 1 2 3 4 5) (whaia (gloria (54 90 142))) goings (comings (7 9)) (aways 90))
```

What if I defined a programming language for the termpose syntax?
```
let i 0
while <(i 5)
  print 'Hello
  println "' worldly people
  =(i +(i 1))

>Hello worldly people
>Hello worldly people
>Hello worldly people
>Hello worldly people
>Hello worldly people

class Dog type_parameters(<(WillBite Professional) <(FavFood Food)) extends:BitingAnimal
  def meets prof:Professional !:
    if eq type(prof) WillBite
      then
        bite prof
      else
        bark prof
  def bark prof:Professional !:
    utter prof "he comes, beware, he comes
  def beg !:
    utter nearbyPeople "give me {}"(FavFood.name)

//"hmm... there's a case we might not handle so well;
a or b or c or d or e
//"No infix exprs. Doesn't work at all.
or(a or(b or(c or(d e))))
//"would work but is unsightly
or(a or(b or(c or(d e
//"Closing parens at the end of a line is not required, so this is reasonably nice
or a, or b, or c, or d, e
//"like coffeescript
or a, or b, or c, or d e
//"would work if we cast the comma as being equivalent to an indent, but it's not very readable.
reduce or a b c d e
//"lol. Maybe that's best.
```

here's me making a start on defining a universal type system:
```termpose
meta
  description "
    notes on DSL:
    // is a comment
    - is a list element

type Vessel type_parameters:E mutates subscribable
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
  fixed
      
type
  name Integer
  serves_as Number
type
  name U32
  //"32 bit, signed, little-endian Integer
  serves_as Integer
type
  name U64
  
  ... you get the point
```

What would termpose imitating json look like?

```javascript
{
  "highlight_line": true,
  "ignored_packages":
  [
    "Floobits",
    "SublimeLinter",
    "Vintage",
    {
      "ref":"http://enema.makopool.com/",
      "name":"enema",
      "update":"yes"
    },
    [1, 2, 3]
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
highlight_line 'true
ignored_packages '[(Floobits SublimeLinter Vintage '{(ref:"http://enema.makopool.com/" name:enema update:yes) '[('1 '2 '3
indent_guide_options(draw_active:'true)
"overlay scroll bars" enabled
"show tab close buttons" 'false
tab_size '2
theme Spacegray.sublime-theme
word_wrap 'true

//"pretty printed
highlight_line 'true
ignored_packages '[:
  Floobits
  SublimeLinter
  Vintage
  '{ ref:"http://enema.makopool.com/" name:enema update:yes
  '[ '1 '2 '3
indent_guide_options
  draw_active 'true
"overlay scroll bars" enabled
"show tab close buttons" 'false
tab_size '2
theme Spacegray.sublime-theme
word_wrap 'true
```
I think that works out very well. Note that `'`s are required for all non-string keys in order to, for instance, differentiate `true` from the string `"true"`, and `"`s alone would not do this, as, once parsed through the termpose filter, there is no perceptible difference between the terms `"true"` and a `true`.

Caveat: Does not preserve the difference between js inputs `"a"` and `'a'`. It also cannot be formatted as a single line due to the unnamed but implicit root element.
