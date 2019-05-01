# nakedlist

Nakedlist is a relatively simple indent-sensitive format for expressing structured data.

example:

```
agent
  name "Mitch Henderson"
  role "hotdog seller"
  inventory
    "hotdog cart"
    tongs
    hat
      attribute striped
  description "this hotdog seller wants to sell you a hotdog"

agent
  name Johnson
  role "newspaper reader"
  inventory newspaper
  description "Johnson is sitting on a bench, reading a newspaper"

(agent (name Crouo) (role Pigeon) (inventory feathers straw) (description "Just a darn pigeon"))

(agent
  (name Wuo) (role Pigeon) (inventory dominance (gut (contents crumbs poop))) (description "A more prominent pigeon
Wuo is bobbing flamboyantly next to the females, and no one is stopping him.
Wuo isn't taking any shit from anyone.
Except for the seagulls,
who are big and mean.")
)

```

Nakedlist is a [Termpose](https://github.com/makoConstruct/termpose) language, meaning it parses into Terms and can be worked with using the same APIs as other termpose formats.

It is formally defined [here](https://github.com/makoConstruct/termpose/blob/master/nakedlist_spec.md).
