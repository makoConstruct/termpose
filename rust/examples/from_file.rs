extern crate wood;
use wood::{parse_multiline_termpose};
use std::error::Error;

fn main()-> Result<(), Box<dyn Error>> {
    let w = parse_multiline_termpose(std::fs::read_to_string("longterm.term")?.as_str())?;
    //assert that the first term in the file has 14 children
    assert_eq!(w.head()?.tail().len(), 14);
    Ok(())
}