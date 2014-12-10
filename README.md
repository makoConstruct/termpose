##termpose
A dead simple, pretty, machine-readable format for trees of strings.

###it's basically s-expressions, but not
Termpose is extremely regular, but instead of basing all of its data on lists, termpose speaks of terms and their associated sequences, (EG `array(1 2 3 4)`, 'array' is the term, the numbers are the sequence), under the expectation that the host language will interpret sequences in terms of their associated term. This is a reasonable expectation, considering that even Lisp gives special meaning to the first term of its lists in the majority of cases.

Another differentiating feature is that termpose is aware of indentation. Expressions like
```
a
	b
	c
		d
	e
```
will be equivalent to `a(b c(d) e)`. This allows you to lay out your termpose with a minimum of friction and aesthetic noise.

syntax by example:
```
A B
> B pertains to A
A B C
> B and C pertain to A
A
  B
  C
> equivalent
A(B C)
> equivalent
A B:C D:E
> B and D pertain to A, but C and E pertain to their colon buddy, B and D respectively
A "a string"
> "a string" pertains to "A" ("A" == A, all symbols are just strings)
A "a string
> equivalent. Closing quote isn't needed if the line ends. What else could such a statement be intended to mean?
A """
  a string
  that is multiline
> "a string↩that is multiline" pertains to A
```

potential translation scheme:
```
term = parse("""
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

javascript:
>{{abacus:"opens"}, horse:1, to:{1:1,2:1,3:1,4:1,5:1}, whaia:{gloria:{54:1,90:1,142:1}}}
```