
extern crate termpose;
extern crate termpose_derive;

use termpose::Termable;
use termpose_derive::Termable;


#[derive(Termable)]
struct Thing {
	a: String,
	b: String,
}

#[derive(Termable)]
enum Uh{
	Varian(String),
	Bobo{thing:String},
	Noune,
}


fn prin<T:Termable>(v:&T){ println!("it's {}", v.termify().to_string()); }

fn main() {
	
	println!("{}", 0usize.to_string());
	
	let u = Uh::Varian("n".into());
	// if let Uh::Varian(nn,) = u {
		
	// }
	
	prin(&Thing{a: "yes".into(), b: "huuu".into()});
	
	prin(&u);
	prin(&Uh::Noune);
	prin(&Uh::Bobo{thing: "hamnure".into()});
	
}
