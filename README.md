##termpose
The ultimate markup language, combining an absolutely minimal syntax with an extremely regular, flexible in-memory representation.

###It's arguably more regular than s-expressions
Parsed termpose resolves to a tree of terms and their associated sequences (EG `array(a b c d e)`, 'array' is the term, the letters are the terms in its sequence). This convention eliminates the edge cases of termless empty lists and heterogynous element types: every entity in the tree is a term with a string at its head which tells the parser how to take it. Symbols are not a separate type, but a component of the fundamental unit. In practice, these differences do not set termpose sublanguages apart from other languages, as with the right conventions, anything had by json or lisp or yaml can be easily replicated. Termpose mimicing yaml looks (marginally)better than yaml. Termpose mimicing JSON looks (unambiguously)better than json. Termpose mimicking xml is not a contest. Termpose mimicking lisp.. well, if you want that I should refer you to http://srfi.schemers.org/srfi-110/srfi-110.html they do it better than termpose could.


##Syntax

Termpose pays attention to indentation. Expressions like
```
a
	b
	c
		d
	e
```
will be equivalent to `a(b c(d) e)`.

Termpose tries to keep out of your namespace, attributing its own meanings to the characters `":()`, but leaving ```\/?-+=[]*&^%$#@!`~;'.,<>``` for your domain-specific languages to define as they please.

###Spec by example
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
A B(C)
  D
#A contains B and D. B contains C.
A B(C
  D
#equivalent (crazy, right? Thing is: Support for multiline indentation syntax makes the use of paren blocks spanning multiple lines completely unnecessary and against convention, thus unjustifiable. As a result, it'd be really dumb to require people to close their parens. It'd just be tricky, inhumane pedantry.)
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

###What if I defined a programming language for the termpose syntax?
```
let i 0
while <(i 5
  print 'Hello
  println "' worldly people
  =(i +(i 1

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