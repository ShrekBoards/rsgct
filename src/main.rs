extern crate exitcode;

use std::{env, process};

fn main() {
	let args: Vec<_> = env::args().collect();

	if args.len() <= 2 {
		println!("Usage:");
		println!("rsgct -e file.gct [-w width] [-h height] [-s start_byte_hex]");
		println!("rsgct -i file.png -t file.gct [-s start_byte_hex] [-wd (write dimensions)]");
		process::exit(exitcode::USAGE);
	}
	
}
