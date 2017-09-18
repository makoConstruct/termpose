[![Join the chat at https://gitter.im/makoConstruct/termpose](https://badges.gitter.im/Join%20Chat.svg)](https://gitter.im/makoConstruct/termpose?utm_source=badge&utm_medium=badge&utm_campaign=pr-badge&utm_content=badge)

## Termpose - a sensitive markup language

Termpose is an extremely flexible markup language with an elegant syntax.

Termpose data, once parsed, is very simple to work with. It doesn't have maps and numbers. It only has lists and strings. APIs are provided for type-checking-translating into types like maps and numbers, of course, but the way you use the data doesn't constrain the way you might represent it.

Termpose's advantage is its supreme writability. Termpose is minimal and flexible enough that if you wanted to make a programming language, but couldn't be bothered writing a parser, you could just use Termpose, and it wouldn't be all that bad. IMO, it would end up nicer than most lisps.

Here's an example of the kind data I store in termpose. This is very similar to the save format of a game I'm working on:
```
world
  folk
    id 0
    position 0 10
    name blue
    tribe noctal
    carrying staff "poisoned apple" "veiled umbrella" shawl
  folk
    id 1
    position 0 -3
    name mako
    tribe noctal
    carrying string sticks "jar of sencha"
  creature
    id 2
    position -10 -2.6
    visual_range 8.7
    cognition
      on detect(1 player) do chase(1)
      on field(light) do chill_out
  item
    id 3
    paper
    position -12 3
    text"
      The agent sees that the cables of the rust-covered machine,
      fraying out of their worn sheath, are already plugged into 
      the wall. It finds a small lever on the side of the machine
      , flips it, and after a pang and a searing flash that fills
      the room, the red light on the machine starts to blink.
```

In an s-expression language, that would would break down to the following structure:
```
(world
  (folk
    (id 0)
    (position 0 10)
    (name blue)
    (tribe noctal)
    (carrying staff "poisoned apple" "veiled umbrella" shawl)
    (conversation (id 3)))
  (player
    (id 1)
    (position 0 -3)
    (name mako)
    (tribe noctal)
    (carrying string sticks "jar of sencha"))
  (creature
    (id 2)
    (position -10 -2.6)
    (visual_range 8.7)
    (cognition
      (on (detect 1 player) do (chase 1))
      (on (field light) do chill_out)))
  (item
    (id 3)
    paper
    (position -12 3)
    (text "The agent sees that the cables of the rust-covered machine,\nfraying out of their worn sheath, are already plugged into \nthe wall. It finds a small lever on the side of the machine\n, flips it, and after a pang and a searing flash that fills\nthe room, the red light on the machine starts to blink.")))
```

Termpose tries to keep out of your namespace, attributing its own meanings to the characters `":()`, but leaving ```\/?-+=[]*&^%$#@!`~;'.,<>``` for your domain-specific languages to define as they please.

We were always able to use a nice, minimal, flexible S-Expressions language instead of XML, Json, Toml, Yaml or whatever else, but there was something about those old syntaxes that made the prospect unpalatable.

Termpose takes that something out of the picture.

## Platform support?

| language | status | the closest thing we have to documentation |
| ---------|--------|------ |
| C++ | Very nice. The maintainer is using it actively right now. | [Basic API](https://github.com/makoConstruct/termpose/blob/master/basic%20C%2B%2B%20api.md), [Intro to parser combinators](https://github.com/makoConstruct/termpose/blob/master/cppintro.md), [termpose.ccp](https://github.com/makoConstruct/termpose/blob/master/termpose.cpp) |
| Haxe | Pretty good. | [Haxe Source](https://github.com/makoConstruct/termpose/blob/master/Termpose.hx) |
| Scala | Very nice | [This brief intro](https://github.com/makoConstruct/termpose/wiki/Introducing-Termpose!-(To-Scala)) and the [Source](https://github.com/makoConstruct/termpose/blob/master/src/main/scala/Termpose.scala) (start at the bottom) |
| Javascript | There is a basic API that converts termpose to arrays of strings. | [Haxe Source](https://github.com/makoConstruct/termpose/blob/master/Termpose.hx) |
| C# | Give me an API design and I'll do it | [Haxe Source](https://github.com/makoConstruct/termpose/blob/master/Termpose.hx) |
| Java | Talk to me if you'd like to make a nice API, it'll be easy | [Haxe Source](https://github.com/makoConstruct/termpose/blob/master/Termpose.hx) |
| Python | Talk to me if you'd like to make a nice API, it'll be easy | [Haxe Source](https://github.com/makoConstruct/termpose/blob/master/Termpose.hx) |
| C | low-level char* → Term* API is implemented and without leaks | [C Header](https://github.com/makoConstruct/termpose/blob/master/termpose.h) |
| Rust | Basic support + custom Codings. | [Rust Source](https://github.com/makoConstruct/termpose/blob/master/rust/src/lib.rs) |


## Tell us about the implementation?
Termpose is implemented in the style of a streaming state-machine. This in combination with the obstinately design-led nature of the application, which flat out refused to turn around when it started to understand why no one else does syntaxes like this, has lead to very ugly code, which required a lot of documentation of the mutable state. HOWEVER, this style gets us pretty well optimal time and memory efficiency. It also makes it quite straightforward to build event-driven parsers(IE, parsers that can find specific structures at specific places in a giant mass of termpose without parsing the entire thing into memory first) if the need ever arises.


### Spec by example

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