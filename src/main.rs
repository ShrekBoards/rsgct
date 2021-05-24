use std::{env, fs::File, process};

fn main() {
	let args: Vec<_> = env::args().collect();

	if args.len() < 3 {
		println!("Not enough arguments\n");
		usage();
		process::exit(exitcode::USAGE);
	}
	
	let mode_request = &args[1];
	match mode_request.as_str() {
		"-e" => extract(args),
		"-i" => inject(args),
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

fn extract(args: Vec<String>) {
	let file_path = &args[2];

	let gct = File::open(file_path);
	let gct_file: File;
	match gct {
		Ok(f) => gct_file = f,
		Err(error) => {
			let error_string = error.to_string();
			println!("{}\n", error_string);
			usage();
			process::exit(exitcode::NOINPUT);
		}
	}
}

fn inject(args: Vec<String>) {
	if args.len() < 5 {
		println!("Not enough arguments\n");
		usage();
		process::exit(exitcode::USAGE);
	}

	let png_path = &args[2];

	let png = File::open(png_path);
	let png_file: File;
	match png {
		Ok(f) => png_file = f,
		Err(error) => {
			let error_string = error.to_string();
			println!("PNG Error: {}\n", error_string);
			usage();
			process::exit(exitcode::NOINPUT);
		}
	}

	if args[3] != "-t" {
		println!("No -t argument provided\n");
		usage();
		process::exit(exitcode::USAGE);
	}

	let gct_path = &args[4];

	let gct = File::open(gct_path);
	let gct_file: File;
	match gct {
		Ok(f) => gct_file = f,
		Err(error) => {
			let error_string = error.to_string();
			println!("GCT Error: {}\n", error_string);
			usage();
			process::exit(exitcode::NOINPUT);
		}
	}
}