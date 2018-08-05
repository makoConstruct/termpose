[![Join the chat at https://gitter.im/makoConstruct/termpose](https://badges.gitter.im/Join%20Chat.svg)](https://gitter.im/makoConstruct/termpose?utm_source=badge&utm_medium=badge&utm_campaign=pr-badge&utm_content=badge)

## Termpose - a sensitive markup language

Termpose is an extremely flexible markup language with an elegant syntax.

Termpose data, once parsed, is very simple to work with. It doesn't have maps and numbers. It only has lists and strings. APIs are provided for type-checking-translating into types like maps and numbers, of course, but the way you use the data doesn't constrain the way you might represent it.

Termpose's advantage is its supreme writability. Termpose is minimal and flexible enough that if you wanted to make a programming language, but couldn't be bothered writing a parser, you could just use Termpose, and it wouldn't be all that bad. IMO, it would end up nicer than most lisps.

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

The only characters termpose applies special meaning to are `":() `. The rest are yours to use as you please

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

