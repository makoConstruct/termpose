##Termpose - a sensitive markup language

[![Join the chat at https://gitter.im/makoConstruct/termpose](https://badges.gitter.im/Join%20Chat.svg)](https://gitter.im/makoConstruct/termpose?utm_source=badge&utm_medium=badge&utm_campaign=pr-badge&utm_content=badge)
Termpose is an extremely flexible markup language with an elegant syntax.

Termpose pays attention to your indentation. Expressions like
```
a
    b
    c
        d
    e
```
will be equivalent to `a(b c(d) e)`, which in turn equates to, and will serialize as the termpose expression `(a b (c d) e)`.

Termpose tries to keep out of your namespace, attributing its own meanings to the characters `":()`, but leaving ```\/?-+=[]*&^%$#@!`~;'.,<>``` for your domain-specific languages to define as they please.

Though most of termpose's applications are in representing data(say, config files, make files), like S-Expressions, termpose is robust enough and minimal enough that you could easily express program code with it. To illustrate, here's what a termpose programming language might look like:
```python
var i 0
while <(i 5)
  print 'Hello
  if ==(0 %(i 3))
    then
      println "' worldly people
    else
      println "' anxious dog
  +=(i 1)

>Hello anxious dog
>Hello worldly people
>Hello worldly people
>Hello anxious dog
>Hello worldly people


import std.Add
import std.Copy
import std.Clone
import std.TermposeSerialize

struct Vector x:Float y:Float

impl Add for Vector
  fun + self:Vector other:Vector ->:Vector
    Vector
      + self.x other.x
      + self.y other.y
impl Copy Clone TermposeSerialize for Vector

fun * m:Number v:Vector -> Vector(*(m v.x) *(m v.y))

println toSexp(*(5 Vector(1 3)

>(Vector 5 15)
```

We were always able to use S-Expressions instead of XML, Json Toml, Yaml or whatever else, but there was something about that syntax that made the prospect unpalatable.

Termpose takes that something out of the picture.

##Platform support?

| language | status | the closest thing we have to documentation |
| ---------|--------|------ |
| Scala | Fly | [Source](https://github.com/makoConstruct/termpose/blob/master/src/main/scala/Termpose.scala) (start at the bottom) |
| Javascript | Imminent | [Haxe Source](https://github.com/makoConstruct/termpose/blob/master/termpose.hx) |
| C# | Imminent | [Haxe Source](https://github.com/makoConstruct/termpose/blob/master/termpose.hx) |
| Java | Works | [Haxe Source](https://github.com/makoConstruct/termpose/blob/master/termpose.hx) |
| Python | Works | [Haxe Source](https://github.com/makoConstruct/termpose/blob/master/termpose.hx) |
| C++ | Maybe Imminent, uncertain about the generated code | [Haxe Source](https://github.com/makoConstruct/termpose/blob/master/termpose.hx) |

###key
| Fly | No docs, but the API is quite comprehensive |
| Imminent | Already has support from the basic Haxe API and mako will elevate it to Fly the moment someone gives them a well designed, idiomatic interface to hook up. |
| Works | Haxe support, but the basic Haxe-support API for the parser and Term Tree data structure is minimal and crude. However, anyone can connect the basic API up to something more idiomatic very easily. |


##Tell us about the implementation?
Termpose is implemented in the style of a streaming state-machine. This in combination with the obstinately design-led nature of the application has lead to very ugly code, which required a lot of documentation of the mutable state. HOWEVER, this style gets us pretty well optimal time and memory efficiency. It also makes it quite straightforward to build event-driven parsers(IE, parsers that can find specific structures at specific places in a giant masses of termpose without parsing the entire thing into memory first) if the need ever arises.


##Termpose-formatted XML dialect
Termpose → XML translation is partially implemented, see the function translateTermposeToSingleLineXML.

Here's how one would write the first half of Mako's homepage in termpose's XML dialect:
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

##Termpose is better at expressing Json than Json

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

```javascript
highlight_line true
ignored_packages [(
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
`'` may be used an escape for instances like, say, representing the string `"true"` with `'true`(parsed termpose has no record of whether there were double quotes, so `"true"` would be indistinguishable from `true`).


###Spec by example

Here I will try to document the details of termpose's behavior by listing examples of termpose and their structure.

```python
A B
#(A B)
A B C
#(A B C)
A
  B
  C
#equivalent to previous
A(B C)
#equivalent
A (B C)
#(A (B C))
A B C
  D E
#(A B C (D E))
A(B C)
  D E
#((A B C) (D E))
A(B C) D
  E
  F
#((A B C) D E F)
A B C D
  E
  F
#A contains everything
A B C D(
  E
  F
#(A B C (D E F)). If a paren is left open, it contain any indented lines
A B(C
  D
#(A (B C D))
A (B F (C
  D
  E
#(A (B F (C D E))). It is the final open paren that contains the indented content
A(B(C(D(E
N(M(E
#((A (B (C (D E)))) (N (M E))). If the next line is not indented, parens left open will not contain anything extra.
A "a string"
#(A "a string"). (all symbols are just strings, but this one contains a space, so it needs to be represented with quotes.)
A "a string
#equivalent. Closing quote isn't needed if the line ends.
A"a string
#equivalent. Quotes are not valid characters for symbol names. If you wanted a symbol with a quote in it, here's how to do that:
"A\"a" string
#("A\"a" string)
A "
  a string that
  is multiline
#(A "a string that↩is multiline")
A B:C D:E
#(A (B C) (D E))
A:B:C:D:E
#(A (B (C (D E))))
A :B :C :D :E
#(A (B) (C) (D) (E)). I don't know when anyone would use this but it's not like " :" has any other reasonable meanings.
A B:
  C
  D
#(A (B C D)). Again, kinda unnecessary when you can s/:/(/, but there's no other reasonable way to interpret this.
```