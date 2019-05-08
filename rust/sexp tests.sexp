(tests
	("small"
		(a b)
		(a b)
	)
	("benign newline"
		(a b
		c)
		(a b c)
	)
	("big"
		(a b
			"c
def

  " xy (99 9 (
  odo
  	od
  )
		)
	)
		(a b "c\ndef\n\n  " xy (99 9 (odo od)))
	)
)