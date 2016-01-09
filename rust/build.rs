use std::process::Command;
use std::process::Output;
use std::env;
use std::path::Path;

fn compose_output(o:std::io::Result<std::process::Output>)-> Result<String, String> {
	let to_utf8 = |v:Vec<u8>| match String::from_utf8(v) { Ok(s)=> s, Err(_)=> "<was not valid unicode>".to_string() };
	match o {
		Ok(Output{status, stdout, stderr})=>{
			let succeeded = status.success();
			let were_error = stderr.len() > 0;
			match (succeeded, were_error) {
				(true, false)=> Ok(to_utf8(stdout)),
				(false, false)=> Err(format!("process failed with status code {}\n{}",
					status,
					to_utf8(stdout),
				)),
				(s, true) =>{
					let output = format!("process {} there was stderr output\nstdout:\n{}\nstderr:\n{}",
						if s { "completed successfully, but" } else { "failed, and" },
						to_utf8(stdout),
						to_utf8(stderr),
					);
					if s { Ok(output) } else { Err(output) }
				},
			}
		}
		Err(_)=> Err("command failure".to_string())
	}
}

fn print_and_panic_on_fail(o:Result<String, String>){
	match o {
		Ok(o)=> {
			println!("{}", o);
		},
		Err(o)=> {
			println!("{}", o);
			panic!("could not build");
		},
	}
}

fn main(){
	env::set_current_dir(&Path::new("src")).unwrap();
	print_and_panic_on_fail(compose_output(Command::new("make").output()));
	println!("cargo:rustc-link-lib=static=termpose");
	println!("cargo:rustc-link-search=src");
}