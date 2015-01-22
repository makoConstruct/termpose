##termpose
A markup language, and symbol tree representation format. Noise-free syntax and pure, flexible data models.

###It's arguably more regular than s-expressions
Parsed termpose resolves to a tree of terms and their associated sequences (EG `array(a b c d e)`, 'array' is the term, the letters are the terms in its sequence, and those letters sequences are empty). This convention eliminates the edge cases of termless empty lists and varying element types: every entity in the tree is a term with a string at its head which tells the parser how to take it, and a tail of similar entities pertaining to it. Symbols are not a separate type, but a component of the fundamental unit. In practice, these differences do not set termpose sublanguages apart from other languages, as, with the right domain-specific language, anything had by json or lisp or yaml can be easily replicated. Termpose mimicing yaml looks (marginally)better than yaml. Termpose mimicing JSON looks (unambiguously)better than json. Termpose mimicking xml is not even a contest. Termpose mimicking lisp.. well, if you want that I should refer you to http://srfi.schemers.org/srfi-110/srfi-110.html , but termpose would do a commendable job.


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
#equivalent to A(B)
A B C
#equivalent to A(B C)
A
  B
  C
#equivalent
A(B C)
#equivalent
A(B C)
  D E
#A(B C D(E)).
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
#A(B C D(E F)). If the line sequence is terminated by a colon, the last element takes the indent sequence, rather than the first.
A B(C)
  D
#A(B(C) D)
A B(C
  D
#equivalent (crazy? Thing is: Since indentation is supported and suffient for expressing containments spread over multiple lines, there is no need to allow brackets to do this. So we don't. Stemming from that, since a bracket cannot possibly continue over multiple lines, we just close them automatically at the end of the line.)
A B:C D:E
#A contains B and D, but C and E are contained by their colon buddy, B and D respectively
A:B:C:D:E
#A(B(C(D(E))))
A(B(C(D(E
#equivalent, in this case
A "a string"
#A("a string"). (all symbols are just strings, but this one contains a space, so it needs quotes.)
A "a string
#equivalent. Closing quote isn't needed if the line ends.
A"a string
#equivalent
A "
  a string that
  is multiline
#A contains "a string that↩is multiline"
```
<!-- A B, C D, E F
#A(B C(D E(F))). Commas start a new term within the line. -->

##Surely as a burgeoning format the parser is woefully inefficient?
The community has lead me unto the impression that Jackson, the Json parser for Java is, like, *super fast*. I compared my early, unoptimized, Scala-based termpose parser with Jackson, and there is basically no discernable difference between their performance. Maybe I just have efficient coding habits. Maybe it's the noodley graphic way I architected the thing. Either way, the termpose parser is perfectly competitive already. On that dimension.

###What if you defined a programming language for the termpose syntax? What would that look like?

Probably something like this
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

class Dog type_parameters(<(WillBite Professional) <(FavFood Food) //"Note, the one on the right of the < exprs is a type bound, IE, it's saying FavFood must be a subclass of Food") extends:BitingAnimal
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

here's me making a start on defining a cross-language type system(see Cap'n Proto if you're interested in that kind of thing):
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

##Termpose is better for typing json than json

for
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
highlight_line true
ignored_packages [(Floobits SublimeLinter Vintage {(ref:"http://enema.makopool.com/" name:enema update:yes) [(1 2 3
indent_guide_options draw_active:true
"overlay scroll bars" enabled
"show tab close buttons" false
tab_size 2
theme Spacegray.sublime-theme
word_wrap true

//"pretty printed
highlight_line true
ignored_packages [:
  Floobits
  SublimeLinter
  Vintage
  { ref:"http://enema.makopool.com/" name:enema update:yes
  [ 1 2 3
indent_guide_options
  draw_active true
"overlay scroll bars" enabled
"show tab close buttons" false
tab_size 2
theme Spacegray.sublime-theme
word_wrap true
```
I think that works out very well. `'` may be used an escape for instances like, say, representing the string `"true"`(parsed termpose has no record of whether there were double quotes, so that alone wouldn't do).


##How about XML?
Representing my homepage:
```python
html
  head
    title ."about mako
    meta -charset:utf-8
    link -rel:"shortcut icon" -href:DGFavicon.png
    style ."
      CSS plaintext...
    script -language:javascript ."
      Javascript plaintext...
  body
    div -id:centerControlled
      table tbody:
        tr
          td
          td
            div -id:downgust
              canvas -height:43 -width:43
        tr
          td -class:leftCol ."who
          td -class:rightCol ."mako yass, global handle @makoConstruct.
        ...
```
This doesn't work yet, though, I need to fix `c a"b` to mean `c(a(b))` rather than `c(a b)` as it does now. Note


##pedantic spec notes:
When converting in-memory representations to maps, if the same key occurs more than once, the latest occurance should overwrite the earlier ones.