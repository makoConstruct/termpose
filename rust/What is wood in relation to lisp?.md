## What is wood in relation to lisp?

- It basically is just syntaxes for expressing s-expressions, it is mostly the same as that component of lisp.

- Wood supports multiple syntaxes though.

- Wood also isn't list-based, it's based on vecs. Vecs are much faster to iterate over, to add to, and are more memory-efficient than lists. They make accessing the back elements of a branch much faster.

    - Note, this may be especially suited to rust, as rust allows taking subslices of vecs, which means that you really don't miss List::tail methods.

- It was developed more in the context of writing configuration files, but there is a programming language built on wood: [`munk`](https://github.com/makoConstruct/munk).