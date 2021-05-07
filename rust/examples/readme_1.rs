extern crate wood;
use wood::{parse_woodslist, dewoodify};

fn main(){
  let r:Vec<usize> = dewoodify(&parse_woodslist("0 1 2").unwrap()).unwrap();
  
  assert_eq!(2, r[2]); //easy as zero one two
}