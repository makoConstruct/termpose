[![Join the chat at https://gitter.im/makoConstruct/termpose](https://badges.gitter.im/Join%20Chat.svg)](https://gitter.im/makoConstruct/termpose?utm_source=badge&utm_medium=badge&utm_campaign=pr-badge&utm_content=badge)

## Termpose - a sensitive markup language

Termpose is an extremely flexible markup language with an elegant whitespace syntax.

Termpose data, once parsed, is very simple to work with. It doesn't have maps and numbers. It only has lists and strings. APIs are provided for type-checking-translating into types like maps and numbers, but the format itself does not impose. The only characters termpose applies special meaning to are `":() `. The rest are yours to use as you please.



Here's an example of the kind data I store in termpose. This is very similar to the save format of a game I'm working on:
```
mon
  name leafward
  affinity creation
  description "
    Plants healing bombs.
    Standard attack.
    Watch out, it's fragile!
  stride 2
  stamina 3
  recovery 2
  health 50
  abilities
    move
    strike drain:2 damage:standard:2
    bomb drain:3 effect:heal:standard:2 grenadeTimer:2 grenadeHealth:20 range:2 radius:2
```

In an s-expression language, that would would break down to the following structure:

```
(mon
  (name leafward)
  (affinity creation)
  (description
"Plants healing bombs.
Standard attack.
Watch out, it's fragile!")
  (stride 2)
  (stamina 3)
  (recovery 2)
  (health 50)
  (abilities
    move
    (strike (drain 2) (damage (standard 2)))
    (bomb (drain 3) (effect (heal (standard 2))) (grenadeTimer 2) (grenadeHealth 20) (range 2) (radius 2))
  )
)
```

We were always able to use a nice, minimal, flexible S-Expressions language instead of XML, Json, Toml, Yaml or whatever else, but there was something about those old syntaxes that made the prospect unpalatable. Termpose takes that something out of the picture.

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


## Format in detail

Here, I'm going to describe how data and structures can be written in termpose.

In s-expression based languages, the first term in a list is considered special. For instance,

```
(f a b c)
```

Here, this would generally be an application of the function `f`, being invoked on the variables `a`, `b` and `c`.

In termpose, this structure can also be expressed in a way that will be more familiar to most programmers and mathematicians, one character shorter to type, and perhaps more explicit

```
f(a b c)
```

Anything that can be expressed that way can also be written

```
f
  a
  b
  c
```

Often, in programming languages, and in data, we want an easy way to express pairs. For instance, in swift, you might define or create a rectangle with

```
CGRect(x: 0, y: 0, width: 320, height: 500)
```

In a termpose context, a structure like this could be expressed in just the same way, minus the colons. We don't need or want any colons around here.

```
CGRect(x: 0  y: 0  width: 320  height: 500)
```

Or more minimally

```
CGRect x:0 y:0 width:320 height:500
```

Or

```
CGRect
  x:0
  y:0
  width:320
  height:500
```

You might interrupt: "But didn't you say there were only lists and atoms in termpose"? Right. Colons are nothing special really, they break down like this

```
(CGRect (x 0) (y 0) (width 320) (height 500))
```

You might say they pair things. You might also say that a colon invokes the first thing as a function of the second thing. Either interpretation is valid.

```
print:"chill"
```

If you chose to chain these commas for a series of chained one-parameter invocations, you could.

```
print: to_lowercase: duplicate_back_letter: "CHIL"
```

Would translate to

```
print(to_lowercase(duplicate_back_letter(CHIL)))
```

You might notice the quote marks around "CHILL" went away in the translation. Quotes are for including whitespace in atoms. If you aren't using any whitespace within the atom string, they aren't needed. Here are some more typical examples of the quote syntax:

```
print "a string"
print "another string"
print "and then another"
```

But what if our hypothetical termpose-based programming language wanted to print something very long, with multiple lines?

```
print "
  a string
  another string
print "and then another"
```

We could simply open a string into an indent before we want it to start and unindent when we want it to end.

If you were really making a termpose-based programming language, you might want to add something extra to quoted atoms to signal that they are literals. Termpose tries to make that easy.

```
print '"
  a string
  another string
print '"and then another"
```

This would break down to

```
( (print (' "a string\nanother string")) (print (' "and then another")) )
```

Which would make it very easy to detect literals by looking for terms that start with `'`.

I hope I've been able to convince you that termpose is flexible enough for whatever data you want to express. To understand how easy it is to work with termpose, take a look at some of the APIs.