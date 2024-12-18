[![Join the chat at https://gitter.im/makoConstruct/termpose](https://badges.gitter.im/Join%20Chat.svg)](https://gitter.im/makoConstruct/termpose?utm_source=badge&utm_medium=badge&utm_campaign=pr-badge&utm_content=badge)

## Termpose - a sensitive markup language

Termpose is an extremely flexible markup language with an elegant whitespace syntax.

Termpose data, once parsed, is very simple to work with. It doesn't have maps and numbers. It only has lists and strings. APIs are provided for type-checking-translating into types like maps and numbers, but the format itself does not impose. The only characters termpose applies special meaning to are `":()\ `. The rest are yours to use as you please.



Here's an example of the kind of data I store in termpose. This is very similar to part of the save format of a game I'm working on:
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

In an s-expression language (in this case, [woodslist](https://github.com/makoConstruct/termpose/blob/master/woodslist.md)), that would would break down to the following structure:

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

Warning, Rust is currently the only language that supports termpose per specification, the others do something different with items in a line with indental, because until writing the spec I hadn't thought hard about what made the most sense there.

| language | spec compliant | woodslist | termpose | status | further info |
| ---------|----------------|-----------|----------|--------|------------- |
| C++ | No |  | Yes | Very nice. | [Basic API](https://github.com/makoConstruct/termpose/blob/master/basic%20C%2B%2B%20api.md), [Intro to parser combinators](https://github.com/makoConstruct/termpose/blob/master/cppintro.md), [termpose.ccp](https://github.com/makoConstruct/termpose/blob/master/termpose.cpp) |
| Haxe | No |  | Yes | Pretty good. | [Haxe Source](https://github.com/makoConstruct/termpose/blob/master/Termpose.hx) |
| Scala | No |  | Yes | Very nice | [This brief intro](https://github.com/makoConstruct/termpose/wiki/Introducing-Termpose!-(To-Scala)) and the [Source](https://github.com/makoConstruct/termpose/blob/master/src/main/scala/Termpose.scala) (start at the bottom) |
| Javascript | No |  | Yes | There is a basic API that converts termpose to arrays of strings. | [Haxe Source](https://github.com/makoConstruct/termpose/blob/master/Termpose.hx) |
| C# | No |  | Yes | Give me an API design and I'll do it | [Haxe Source](https://github.com/makoConstruct/termpose/blob/master/Termpose.hx) |
| Java | No |  | Yes | Talk to me if you'd like to make a nice API, it'll be easy | [Haxe Source](https://github.com/makoConstruct/termpose/blob/master/Termpose.hx) |
| Python | No |  | Yes | Talk to me if you'd like to make a nice API, it'll be easy | [Haxe Source](https://github.com/makoConstruct/termpose/blob/master/Termpose.hx) |
| C | No |  | Yes | low-level char* → Term* API is implemented and without leaks | [C Header](https://github.com/makoConstruct/termpose/blob/master/termpose.h) |
| Rust | Yes | Yes | Yes | Good support for termpose and [woodslist](https://github.com/makoConstruct/termpose/blob/master/woodslist.md). Has autoderive. | [Rust Intro](https://github.com/makoConstruct/termpose/blob/master/rust/README.md) |
| Dart | Yes | Yes |  | | |


## Format in detail

Here, I'm going to describe how data and structures can be written in termpose.

In s-expression based languages, the first term in a list is often considered special. For instance,

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

In a termpose context, a structure like this could be expressed in just the same way, minus the commas. We don't need or want any commas around here.

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

I should say, colons aren't a special syntax, they break down to lists as well:

```
(CGRect (x 0) (y 0) (width 320) (height 500))
```

You might say they pair things. You might also say that a colon invokes the first thing as a function of the second thing. Either interpretation is valid.

```
print:"maybe"
```

If you wanted to invoke a series of chained one-parameter invocations, you could.

```
print:to_lowercase:reverse:"EBYAM"
```

Would translate to

```
print(to_lowercase(reverse(EBYAM)))
```

You might notice the quote marks around "EBYAM" were omitted away in the translation. Quotes are for including whitespace in atoms. If you aren't using any whitespace within the atom string, they aren't needed. Here are some more typical examples of the quote syntax:

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

If we were really making a termpose-based programming language, we might want to add something extra to quoted atoms to signal that we intend them to be read as string literals. Termpose tries to make that easy by having quoted strings be *invoked* by anything they are directly adjacent to. For instance, if we used a `'`,

```
print '"
  a string
  another string
print '"and then another"
```

would break down to

```
((print ("'" "a string\nanother string")) (print ("'" "and then another")))
```

Which would make it very easy to detect literals by looking for terms that start with `'`.

I hope I've been able to convince you that termpose is flexible enough for whatever data you want to express. To understand how easy it is to work with termpose, take a look at some the API intros listed above.