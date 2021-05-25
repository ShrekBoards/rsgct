use std::{
	env,
	fs::File,
	io::{Error, Read, Seek, SeekFrom},
	path::Path,
	process, u32,
};

use image::{
	dxt::{DXTVariant, DxtDecoder},
	png::PngEncoder,
	DynamicImage, ImageBuffer, ImageDecoder, Rgba, RgbaImage,
};

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
		}
	}
}

fn usage() {
	println!("Usage:");
	println!("rsgct -e file.gct [-w width] [-h height] [-s start_byte_hex]");
	println!("rsgct -i file.png -t file.gct [-s start_byte_hex] [-wd (write dimensions)]");
}

fn extract(args: Vec<String>) {
	let file_path_string = &args[2];
	let path = Path::new(file_path_string);
	let filename = path.file_name();
	let fn_string: &str;
	match filename {
		Some(fname) => match fname.to_str() {
			Some(str) => fn_string = str,
			None => fn_string = "[filename error]",
		},
		None => fn_string = "[filename error]",
	}

	let gct = File::open(path);
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
		Ok(_) => {}
		Err(error) => {
			let error_string = error.to_string();
			println!("Seek Error: {}\n", error_string);
			usage();
			process::exit(exitcode::IOERR);
		}
	}

	let width: i32;
	match read_short(&gct_file) {
		Ok(w) => width = w,
		Err(error) => {
			let error_string = error.to_string();
			println!("{}\n", error_string);
			usage();
			process::exit(exitcode::IOERR);
		}
	}

	let height: i32;
	match read_short(&gct_file) {
		Ok(h) => height = h,
		Err(error) => {
			let error_string = error.to_string();
			println!("{}\n", error_string);
			usage();
			process::exit(exitcode::IOERR);
		}
	}

	// TODO: support -w & -h
	//println!("{}", width);
	//println!("{}", height);

	if width == 0 || height == 0 {
		println!("Width/Height from GCT incorrect ({}/{})", width, height);
		usage();
		process::exit(exitcode::NOINPUT);
	}

	// TODO: support -s
	//let start = 0x40;
	let start = 0x40;

	let seek_result = gct_file.seek(SeekFrom::Start(start));
	match seek_result {
		Ok(_) => {}
		Err(error) => {
			let error_string = error.to_string();
			println!("Seek Error: {}\n", error_string);
			usage();
			process::exit(exitcode::IOERR);
		}
	}

	println!(
		"Extracting image from {} of size {}x{} from position {:#X}",
		fn_string, width, height, start
	);

	
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
