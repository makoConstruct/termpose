
extern crate termpose;
extern crate termpose_derive;

use std::fmt::Debug;
use termpose::Termable;
use termpose::Untermable;
use termpose_derive::Termable;
use termpose_derive::Untermable;


#[derive(Termable, Untermable, PartialEq, Eq, Debug)]
struct StructThing {
	a: String,
	b: String,
}

#[derive(Termable, Untermable, PartialEq, Eq, Debug)]
struct TupleStructThing(String, String);

#[derive(Termable, Untermable, PartialEq, Eq, Debug)]
struct UnitStructThing;

#[derive(Termable, Untermable, PartialEq, Eq, Debug)]
enum EnumThing {
	TupleVariant(String),
	StructVariant{a:String},
	UnitVariant,
}



fn check<T:Termable + Untermable + PartialEq + Debug>(v:&T){
	let termifiedstr = v.termify().to_string();
	println!("it's {}", termifiedstr.as_str());
	assert_eq!(v, &T::untermify(&termpose::parse(termifiedstr.as_str()).unwrap()).unwrap());
}

fn main() {
	check(&StructThing{a: "aa".into(), b: "bb".into()});
	check(&TupleStructThing("aao".into(), "bbo".into()));
	check(&UnitStructThing);
	check(&EnumThing::StructVariant{a: "aa".into()});
	check(&EnumThing::TupleVariant("aa".into()));
	check(&EnumThing::UnitVariant);
}
