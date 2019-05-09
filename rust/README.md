# Rust Wood

![crates.io](https://img.shields.io/crates/v/wood.svg)  ![crates.io](https://img.shields.io/crates/v/wood_derive.svg)

Wood is a very simple serialization datatype consisting of nested lists of strings

[Termpose](https://github.com/makoConstruct/termpose/), Nakedlist, and [Woodslist](https://github.com/makoConstruct/termpose/blob/master/woodslist.md) are text formats that parse into Wood.

The rust library currently has excellent support for termpose and woodslist.

### Examples

```rust
extern crate wood;
use wood::{parse_woodslist, dewoodify};

fn main(){
  let r:Vec<usize> = dewoodify(&parse_woodslist("0 1 2").unwrap()).unwrap();
  
  assert_eq!(2, r[2]); //easy as zero one two
}
```


Although Wood's autoderive isn't as fully featured as serde's (maybe we should make a serde crate for it), it does exist and it does work.

```rust
extern crate wood;
extern crate wood_derive;
use wood::{parse_termpose, pretty_termpose, Woodable, Dewoodable};
use wood_derive::{Woodable, Dewoodable};

#[derive(Woodable, Dewoodable, PartialEq, Debug)]
struct Dato {
  a:String,
  b:bool,
}

fn main(){
  let od = Dato{a:"chock".into(), b:true};
  let s = pretty_termpose(&od.woodify());
  
  assert_eq!("Dato a:chock b:true", &s);
  
  let d = Dato::dewoodify(&parse_termpose(&s).unwrap()).unwrap();
  
  assert_eq!(&od, &d);
}
```


There are also these things called wooder combinators. I haven't found a way to make them really useful in rust for various reasons, but they'll be (usually) zero-sized values that you can assemble to specify a translation between wood and data (you usually explain both directions at once. I wanna call them "bifunctions").

I've made a start on Dewooder/Wooder combinators, for when you want to customize the parsing process.

```rust
extern crate wood;
extern crate wood_derive;
use wood::{wooder, Wood, Dewooder};
use wood_derive::{Woodable, Dewoodable};

#[derive(Woodable, Dewoodable)]
struct Datu {
  name: String,
  numbers: Vec<u32>,
}

fn main(){
  let data:Wood = wood::parse_multiline_termpose("
  
list
  entry 1
  entry 2
  sublist
    Datu name:n numbers(0 1 2)
    Datu name:nnn numbers(0 1 2)
  entry 3
  
").unwrap();
  
  let sublist:&Wood = data.find("list").unwrap().find("sublist").unwrap();
  
  let _:Vec<Datu> = wooder::TaggedSequence("sublist", wooder::Central).dewoodify(sublist).unwrap();
}
```