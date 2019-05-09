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