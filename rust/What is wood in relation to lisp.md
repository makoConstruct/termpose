## What is wood in relation to lisp?

- It basically is just syntaxes for expressing s-expressions, it is mostly the same as that component of lisp.

- Wood supports multiple syntaxes though.

- Wood also isn't list-based, it's based on vecs. Vecs are much faster to iterate over, to add to, are more memory-efficient, and support constant-time indexing, which includes fast access to the back elements.

    - Note, this may be especially suited to rust, as rust allows taking subslices of vecs, which means that you really don't miss List::tail methods, and you also have other subslices that lisp can't do, like taking a reference of just the first n elements (lisp can only take slices that include the last element).

- It was developed more in the context of writing configuration files, but there is a programming language built on wood: [`munk`](https://github.com/makoConstruct/munk).