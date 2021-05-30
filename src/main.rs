use std::{
    env,
    fs::File,
    io::{Error, Read, Seek, SeekFrom},
    path::Path,
    process, u32,
};

use image::{Rgb, Rgba, RgbaImage};

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
    let fn_no_ext = path.file_stem();
    let fn_string: &str;
    let fs_string: &str;
    match filename {
        Some(fname) => match fname.to_str() {
            Some(str) => fn_string = str,
            None => fn_string = "[filename error]",
        },
        None => fn_string = "[filename error]",
    }

    match fn_no_ext {
        Some(fname) => match fname.to_str() {
            Some(str) => fs_string = str,
            None => fs_string = "[filename error]",
        },
        None => fs_string = "[filename error]",
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

    let width: u32;
    match read_short(&gct_file) {
        Ok(w) => width = w,
        Err(error) => {
            let error_string = error.to_string();
            println!("Unable to read width: {}\n", error_string);
            usage();
            process::exit(exitcode::IOERR);
        }
    }

    let height: u32;
    match read_short(&gct_file) {
        Ok(h) => height = h,
        Err(error) => {
            let error_string = error.to_string();
            println!("Unable to read height: {}\n", error_string);
            usage();
            process::exit(exitcode::IOERR);
        }
    }

    // TODO: support -w & -h
    //println!("{}", width);
    //println!("{}", height);

    if width == 0 || height == 0 {
        println!("width/height from GCT incorrect ({}/{})", width, height);
        usage();
        process::exit(exitcode::NOINPUT);
    }

    // TODO: support -s
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

    let png_buffer = create_png(&gct_file, width, height);

    let png_path = format!("{}.png", fs_string);
    let save_result = png_buffer.save(&png_path);
    match save_result {
        Ok(_) => {}
        Err(error) => {
            let error_string = error.to_string();
            println!("Unable to save {}: {}\n", png_path, error_string);
            usage();
            process::exit(exitcode::IOERR);
        }
    }

    println!("Extraction completed successfully to {}", png_path);
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

fn read_short(mut file: &File) -> Result<u32, Error> {
    let mut buffer = [0; 2]; // 2 byte buffer
    file.read_exact(&mut buffer)?;

    let s = u16::from_be_bytes(buffer);
    let i = u32::from(s);
    Ok(i)
}

fn read_int(mut file: &File) -> Result<u32, Error> {
    let mut buffer = [0; 4]; // 4 byte buffer
    file.read_exact(&mut buffer)?;

    let i = u32::from_be_bytes(buffer);
    Ok(i)
}

fn create_png(gct: &File, width: u32, height: u32) -> RgbaImage {
    let mut png_buffer = RgbaImage::new(width, height);

    let mut x: u32 = 0;
    let mut y: u32 = 0;

    let mut dx: u32 = 0;
    let mut dy: u32 = 0;

    let mut dy_hack = 0;

    while y < height - 4 {
        if dx >= 8 {
            if dy_hack == x {
                dy = 4;
            } else {
                dy = 0;
            }
            dx = 0;
        }

        rw_block(&gct, x + dx, y + dy, &mut png_buffer);

        dy_hack = x;
        if dx == 4 && dy == 4 {
            x += 8;
        }

        dx += 4;

        if x > width - 4 {
            x = 0;
            y += 8;
        }
    }

    png_buffer
}

fn rw_block(gct: &File, x: u32, y: u32, png_buffer: &mut RgbaImage) {
    let colours = get_next_colours(&gct);

    let block = get_block(&gct);

    write_block(block, colours, x, y, png_buffer);
}

fn write_block(
    block: [u32; 16],
    colours: [Rgba<u8>; 4],
    x: u32,
    y: u32,
    png_buffer: &mut RgbaImage,
) {
    for dy in 0..4 {
        for dx in 0..4 {
            let block_index = (dy * 4 + dx) as usize;
            let colour_index = block[block_index] as usize;
            png_buffer.put_pixel(x + dx, y + dy, colours[colour_index]);
        }
    }
}

fn get_next_colours(gct: &File) -> [Rgba<u8>; 4] {
    let lc0: u32;
    match read_short(&gct) {
        Ok(s) => lc0 = s,
        Err(error) => {
            let error_string = error.to_string();
            println!("Unable to read colour 0 from GCT: {}\n", error_string);
            usage();
            process::exit(exitcode::IOERR);
        }
    }

    let lc1: u32;
    match read_short(&gct) {
        Ok(s) => lc1 = s,
        Err(error) => {
            let error_string = error.to_string();
            println!("Unable to read colour 1 from GCT: {}\n", error_string);
            usage();
            process::exit(exitcode::IOERR);
        }
    }

    let c0 = rgb565_to_rgba_colour(lc0);
    let c1 = rgb565_to_rgba_colour(lc1);
    let c2: Rgba<u8>;
    let c3: Rgba<u8>;

    if lc0 >= lc1 {
        let lc2 = mix_colours(lc0, lc1, 2, 1, 3); // 2/3 & 1/3
        c2 = rgb565_to_rgba_colour(lc2);

        let lc3 = mix_colours(lc0, lc1, 1, 2, 3); // 1/3 & 2/3
        c3 = rgb565_to_rgba_colour(lc3);
    } else {
        let lc2 = mix_colours(lc0, lc1, 1, 1, 2); // 1/2 each
        c2 = rgb565_to_rgba_colour(lc2);

        // if lc0 is smaller than lc1 in DXT1, c3 should be transparent
        c3 = Rgba([0, 0, 0, 0]);
    }

    [c0, c1, c2, c3]
}

fn get_block(gct: &File) -> [u32; 16] {
    let indexes: u32;
    match read_int(&gct) {
        Ok(s) => indexes = s,
        Err(error) => {
            let error_string = error.to_string();
            println!(
                "Unable to read colour indexes for block in GCT: {}\n",
                error_string
            );
            usage();
            process::exit(exitcode::IOERR);
        }
    }

    let mut block: [u32; 16] = [0; 16];
    for (i, b) in block.iter_mut().enumerate() {
        *b = (indexes >> (30 - (2 * i))) & 0b11; // 2-bit index for each of 16 pixels in 4x4 group
    };

    block
}

fn mix_colours(c0: u32, c1: u32, mul1: u32, mul2: u32, div: u32) -> u32 {
    let r0 = (c0 >> 11) & 31;
    let g0 = (c0 >> 5) & 63;
    let b0 = c0 & 31;

    let r1 = (c1 >> 11) & 31;
    let g1 = (c1 >> 5) & 63;
    let b1 = c1 & 31;

    let r = (r0 * mul1 + r1 * mul2) / div;
    let g = (g0 * mul1 + g1 * mul2) / div;
    let b = (b0 * mul1 + b1 * mul2) / div;

    (r << 11) | (g << 5) | b
}

// expand rgb565 to rgb888
fn rgb565_to_rgb888(c: u32) -> u32 {
    // duplicate lookup tables to avoid casting u8 -> u32
    // 5-bit to 8-bit lookup table
    const CC58: [u32; 32] = [
        0x00, 0x08, 0x10, 0x19, 0x21, 0x29, 0x31, 0x3a, 0x42, 0x4a, 0x52, 0x5a, 0x63, 0x6b, 0x73,
        0x7b, 0x84, 0x8c, 0x94, 0x9c, 0xa5, 0xad, 0xb5, 0xbd, 0xc5, 0xce, 0xd6, 0xde, 0xe6, 0xef,
        0xf7, 0xff,
    ];
    // 6-bit to 8-bit lookup table
    const CC68: [u32; 64] = [
        0x00, 0x04, 0x08, 0x0c, 0x10, 0x14, 0x18, 0x1c, 0x20, 0x24, 0x28, 0x2d, 0x31, 0x35, 0x39,
        0x3d, 0x41, 0x45, 0x49, 0x4d, 0x51, 0x55, 0x59, 0x5d, 0x61, 0x65, 0x69, 0x6d, 0x71, 0x75,
        0x79, 0x7d, 0x82, 0x86, 0x8a, 0x8e, 0x92, 0x96, 0x9a, 0x9e, 0xa2, 0xa6, 0xaa, 0xae, 0xb2,
        0xb6, 0xba, 0xbe, 0xc2, 0xc6, 0xca, 0xce, 0xd2, 0xd7, 0xdb, 0xdf, 0xe3, 0xe7, 0xeb, 0xef,
        0xf3, 0xf7, 0xfb, 0xff,
    ];

    let r5 = (c >> 11) as usize;
    let g6 = (c >> 5 & 0x3F) as usize;
    let b5 = (c & 0x1F) as usize;

    0xFF << 24 | CC58[r5] << 16 | CC68[g6] << 8 | CC58[b5]
}

// 5-bit to 8-bit lookup table
const CC58: [u8; 32] = [
    0x00, 0x08, 0x10, 0x19, 0x21, 0x29, 0x31, 0x3a, 0x42, 0x4a, 0x52, 0x5a, 0x63, 0x6b, 0x73, 0x7b,
    0x84, 0x8c, 0x94, 0x9c, 0xa5, 0xad, 0xb5, 0xbd, 0xc5, 0xce, 0xd6, 0xde, 0xe6, 0xef, 0xf7, 0xff,
];
// 6-bit to 8-bit lookup table
const CC68: [u8; 64] = [
    0x00, 0x04, 0x08, 0x0c, 0x10, 0x14, 0x18, 0x1c, 0x20, 0x24, 0x28, 0x2d, 0x31, 0x35, 0x39, 0x3d,
    0x41, 0x45, 0x49, 0x4d, 0x51, 0x55, 0x59, 0x5d, 0x61, 0x65, 0x69, 0x6d, 0x71, 0x75, 0x79, 0x7d,
    0x82, 0x86, 0x8a, 0x8e, 0x92, 0x96, 0x9a, 0x9e, 0xa2, 0xa6, 0xaa, 0xae, 0xb2, 0xb6, 0xba, 0xbe,
    0xc2, 0xc6, 0xca, 0xce, 0xd2, 0xd7, 0xdb, 0xdf, 0xe3, 0xe7, 0xeb, 0xef, 0xf3, 0xf7, 0xfb, 0xff,
];

// expand rgb565 to rgb888 and put it in Rgb<u8>
fn rgb565_to_rgb_colour(c: u32) -> Rgb<u8> {
    let r5 = (c >> 11) as usize;
    let g6 = (c >> 5 & 0x3F) as usize;
    let b5 = (c & 0x1F) as usize;

    let r8 = CC58[r5];
    let g8 = CC68[g6];
    let b8 = CC58[b5];

    Rgb([r8, g8, b8])
}

// expand rgb565 to rgb888 and put it in Rgba<u8>
fn rgb565_to_rgba_colour(c: u32) -> Rgba<u8> {
    let r5 = (c >> 11) as usize;
    let g6 = (c >> 5 & 0x3F) as usize;
    let b5 = (c & 0x1F) as usize;

    let r8 = CC58[r5];
    let g8 = CC68[g6];
    let b8 = CC58[b5];

    Rgba([r8, g8, b8, 255])
}
