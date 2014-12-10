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


```
//rambling around:
{
  abacus: "opens",
  to: [
    1,
    2,
    3,
    4,
    5
  ],
  whaia: {gloria: [54, 90, 142]}
  goings:{
    comings:"7  9"
    aways:90
  }
}

term = parse("""
abacus(opens)
horse
to
  1
  2
  3
  4
  5
whaia(gloria(54 90 142))
""")

>[{term:"abacus", c:[{term:"opens"}]}, {term:"horse"}, {term:"to", c:[1,2,3,4,5]}, {term:"whaia", c:[{term:"gloria", c:[{term:"54"}, {term:"90"}, {term:"142"}]}]}]
//¿or maybe
>{{abacus:"opens"}, horse:1, to:{1:1,2:1,3:1,4:1,5:1}, whaia:{gloria:{54:1,90:1,142:1}}}

>Seq(Term("abacus", Seq(Term("opens"))), Term("to", Seq(1,2,3,4,5)), Term("whaia", Seq(Term("gloria", Seq(Term("54"), Term("90"), Term("142"))))))
```
