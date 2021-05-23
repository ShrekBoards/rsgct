use std::{env, process};

fn main() {
	let args: Vec<_> = env::args().collect();

	if args.len() <= 3 {
		usage();
		process::exit(exitcode::USAGE);
	}
	
	let mode_request = &args[1];
	match mode_request.as_str() {
		"-e" => extract(),
		"-i" => inject(),
		_ => {
			println!("Invalid operating mode.\n");
			usage();
			process::exit(exitcode::USAGE);
		},
	}
}

fn usage() {
	println!("Usage:");
	println!("rsgct -e file.gct [-w width] [-h height] [-s start_byte_hex]");
	println!("rsgct -i file.png -t file.gct [-s start_byte_hex] [-wd (write dimensions)]");
}

fn extract() {

}

fn inject() {

}