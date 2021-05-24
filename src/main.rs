use std::{env, fs::File, io::{Error, Read, Seek, SeekFrom}, process};

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
	let mut gct_file: File;
	match gct {
		Ok(f) => gct_file = f,
		Err(error) => {
			let error_string = error.to_string();
			println!("GCT Error: {}\n", error_string);
			usage();
			process::exit(exitcode::NOINPUT);
		}
	}

	let seek_result = gct_file.seek(SeekFrom::Start(0x10));
	match seek_result {
		Ok(_) => {},
		Err(error) => {
			let error_string = error.to_string();
			println!("Seek Error: {}\n", error_string);
			usage();
			process::exit(exitcode::IOERR);
		},
	}
	
	let width: i32;
	match read_short(&gct_file) {
		Ok(w) => width = w,
		Err(error) => {
			let error_string = error.to_string();
			println!("{}\n", error_string);
			usage();
			process::exit(exitcode::IOERR);
		},
	}

	let height: i32;
	match read_short(&gct_file) {
		Ok(h) => height = h,
		Err(error) => {
			let error_string = error.to_string();
			println!("{}\n", error_string);
			usage();
			process::exit(exitcode::IOERR);
		},
	}

	println!("{}", width);
	println!("{}", height);
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

fn read_short(mut file: &File) -> Result<i32, Error> {
	let mut buffer = [0; 2]; // 2 byte buffer
	file.read(&mut buffer)?;

	let s = i16::from_be_bytes(buffer);
	let width = i32::from(s);
	return Ok(width);
}