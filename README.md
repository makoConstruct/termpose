##termpose
A dead simple, pretty, machine-readable format for trees of strings.

###it's basically s-expressions, but not
Termpose is extremely regular, but instead of basing all of its data on lists(a rather unperformant data structure that is not appropriate for most applications), termpose speaks of terms and their associated sequences, (EG `array(1 2 3 4)`, 'array' is the term, the numbers are the sequence), under the expectation that the host language will interpret sequences in terms of their associated term. This is a reasonable expectation, considering that even Lisp gives special meaning to the first term of its lists in every single except for when you're literally specifying a list data structure. Termpose is for contexts where the list ds is not given such undue primacy.

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
to
  1
  2
  3
  4
  5
whaia(gloria(54 90 142))
goings
  comings("7 9")
  aways(90)
""")

>[{term:"abacus", c:[{term:"opens"}]}, {term:"to", c:[1,2,3,4,5]}, {term:"whaia", c:[{term:"gloria", c:[{term:"54"}, {term:"90"}, {term:"142"}]}]}, {term:"goings", c:[{term:"comings", c:[{term:"7"}]}, {term:"aways", c:[{term:"90"}]}]}]

>Seq(Term("abacus", Seq(Term("opens"))), Term("to", Seq(1,2,3,4,5)), Term("whaia", Seq(Term("gloria", Seq(Term("54"), Term("90"), Term("142"))))), Term("goings", Seq(Term("comings", Seq(Term("7"))), Term("aways", Seq(Term("90"))))))
```