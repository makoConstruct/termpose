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