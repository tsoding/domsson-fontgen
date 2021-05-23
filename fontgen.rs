use std::os::raw::{c_char, c_uchar};
use std::ffi::CString;
use std::fs::{File};
use std::io::{BufWriter, Write};

type Pixel = u8;
const COMPS: i32 = 1;

const IMAGE_WIDTH: usize = 128;
const IMAGE_HEIGHT: usize = 64;
const IMAGE_COLS: usize = 18;
const IMAGE_ROWS: usize = 7;
const IMAGE_CHAR_WIDTH: usize = IMAGE_WIDTH / IMAGE_COLS;
const IMAGE_CHAR_HEIGHT: usize = IMAGE_HEIGHT / IMAGE_ROWS;

#[link(name = "stb_image")]
extern "C" {
    fn stbi_load(filename: *const c_char,
                 x: *mut i32, y: *mut i32,
                 comp: *mut i32,
                 req_comp: i32) -> *mut c_uchar;
}

fn compress_monochrome_pixels_into_bits(pixels: &[u8]) -> Vec<u8> {
    let chunk_size: usize = 8;
    assert!(pixels.len() % chunk_size == 0);
    let chunk_count: usize = pixels.len() / chunk_size;

    let mut chunks = Vec::<u8>::new();

    for chunk_index in 0..chunk_count {
        let mut chunk = 0;
        for bit_index in 0..chunk_size {
            let pixel = pixels[chunk_index * chunk_size + bit_index];
            chunk = if pixel == 0x00 {
                chunk << 1
            } else {
                (chunk << 1) | 1
            };
        }
        chunks.push(chunk);
    }

    chunks
}

fn compress_bytes_with_custom_rle(bytes: &[u8]) -> Vec<u8> {
    let mut result = Vec::<_>::new();

    let n = bytes.len();
    let mut i = 0;

    while i < n {
        result.push(bytes[i]);

        if bytes[i] == 0x00 {
            i += 1;
            let mut count: u8 = 1;
            while i < n && bytes[i] == 0x00 && count < 255 {
                i += 1;
                count += 1;
            }
            result.push(count);
        } else {
            i += 1;
        }
    }

    result
}

fn pretty_print_bytes_as_c_array(bytes: &[u8], row_size: usize, name: &str) {
    let row_count: usize = (bytes.len() + row_size - 1) / row_size;
    println!("const unsigned char {}[{}] = {{", name, bytes.len());
    for row in 0..row_count {
        print!("    ");
        for col in 0..row_size {
            let index = row * row_size + col;
            if index < bytes.len() {
                print!("{:#04x}, ", bytes[row * row_size + col]);
            }
        }
        println!("");
    }
    println!("}};");
}

fn pretty_print_bytes_as_rust_array(bytes: &[u8], row_size: usize, name: &str) {
    let row_count: usize = (bytes.len() + row_size - 1) / row_size;
    println!("const {}: [u8; {}] = [", name, bytes.len());
    for row in 0..row_count {
        print!("    ");
        for col in 0..row_size {
            let index = row * row_size + col;
            if index < bytes.len() {
                print!("{:#04x}, ", bytes[row * row_size + col]);
            }
        }
        println!("");
    }
    println!("];");
}

fn save_pixels_as_ppm(pixels: &mut[u8], file_path: &str) {
    assert!(pixels.len() == IMAGE_WIDTH * IMAGE_HEIGHT);
    let ppm_file = File::create(file_path)
        .expect(&format!("Could not create file `{}`", file_path));
    let mut ppm_writer = BufWriter::new(&ppm_file);
    writeln!(&mut ppm_writer, "P6").unwrap();
    writeln!(&mut ppm_writer, "{} {}", IMAGE_WIDTH, IMAGE_HEIGHT).unwrap();
    writeln!(&mut ppm_writer, "255").unwrap();
    for pixel in pixels {
        ppm_writer.write(&[*pixel, *pixel, *pixel]).unwrap();
    }
}

fn generate_solid_character(pixels: &mut [u8]) {
    assert!(pixels.len() == IMAGE_WIDTH * IMAGE_HEIGHT);
    let index = 126 - 32 + 1;
    let x0 = (index % IMAGE_COLS) * IMAGE_CHAR_WIDTH;
    let y0 = (index / IMAGE_COLS) * IMAGE_CHAR_HEIGHT;
    for dy in 0..IMAGE_CHAR_HEIGHT {
        for dx in 0..IMAGE_CHAR_WIDTH {
            let x = x0 + dx;
            let y = y0 + dy;
            assert!((0..IMAGE_WIDTH).contains(&x));
            assert!((0..IMAGE_HEIGHT).contains(&y));
            pixels[y * IMAGE_WIDTH + x] = 0xFF;
        }
    }
}

fn usage() {
    eprintln!("Usage: ./fontgen [OPTIONS] <bitmap-font-spritesheet.png>");
    eprintln!("OPTIONS:");
    eprintln!("    -solid           Generate an additional character with solid color");
    eprintln!("    -debug           Save the modified image as `debug.ppm` for debugging purposes");
    eprintln!("    -raw             Do not apply any compression to the font");
    eprintln!("    -f <rust|c|bin>  Output format of the compressed font");
}

enum Format {
    Rust,
    C,
    Bin
}

impl Format {
    fn from_name(name: &str) -> Option<Self> {
        match name {
            "rust" => Some(Self::Rust),
            "bin" => Some(Self::Bin),
            "c" => Some(Self::C),
            _ => None
        }
    }
}

fn main() {

    let (file_path, format, solid, debug, raw) = {
        let mut file_path: Option<String> = None;
        let mut format = Format::Rust;
        let mut solid = false;
        let mut debug = false;
        let mut raw = false;

        let mut args = std::env::args();
        args.next(); // skip program;

        while let Some(flag) = args.next() {
            match flag.as_str() {
                "-f" => {
                    if let Some(format_name) = args.next() {
                        if let Some(custom_format) = Format::from_name(&format_name) {
                            format = custom_format;
                        } else {
                            usage();
                            eprintln!("ERROR: `{}` is not a correct output format",
                                      format_name);
                            std::process::exit(1);
                        }
                    } else {
                        usage();
                        eprintln!("ERROR: No argument is provided for flag `{}`", flag);
                        std::process::exit(1);
                    }
                }

                "-solid" => {
                    solid = true;
                }

                "-debug" => {
                    debug = true;
                }

                "-raw" => {
                    raw = true;
                }

                _ => if file_path.is_some() {
                    usage();
                    eprintln!("ERROR: Only one input file is supported right now");
                    std::process::exit(1);
                } else {
                    file_path = Some(flag);
                }
            }
        }

        if let Some(file_path) = file_path {
            (file_path, format, solid, debug, raw)
        } else {
            usage();
            eprintln!("ERROR: No input file path was provided");
            std::process::exit(1);
        }
    };

    let pixels: &mut [u8] = unsafe {
        let (mut w, mut h) = (0, 0);
        let file_path_cstr = CString::new(file_path.clone())
            .expect("Could not construct CString out of the provided file path");
        let pixels =
            stbi_load(file_path_cstr.into_raw(), &mut w, &mut h, std::ptr::null_mut(), COMPS) as *mut Pixel;

        if pixels == std::ptr::null_mut() {
            panic!("Could not read file {}", file_path);
        }

        if w != IMAGE_WIDTH as i32 || h != IMAGE_HEIGHT as i32 {
            panic!("Expected image of size {}x{} but got {}x{}",
                   IMAGE_WIDTH, IMAGE_HEIGHT,
                   w, h);
        }

        std::slice::from_raw_parts_mut(pixels, IMAGE_WIDTH * IMAGE_HEIGHT)
    };

    if solid {
        generate_solid_character(pixels);
    }

    if debug {
        save_pixels_as_ppm(pixels, "debug.ppm");
    }

    let compressed_bytes = if raw {
        pixels.to_vec()
    } else {
        compress_bytes_with_custom_rle(
            &compress_monochrome_pixels_into_bits(pixels))
    };

    match format {
        Format::Rust => {
            println!("// Copy-paste this into your code");
            println!("// Generated by https://github.com/tsoding/domsson-fontgen from `{}`", file_path);
            pretty_print_bytes_as_rust_array(&compressed_bytes, 16, "COMPRESSED_FONT");
        }
        Format::Bin => {
            std::io::stdout().write(&compressed_bytes).expect("Could not output compressed bytes to stdout");
        }
        Format::C => {
            println!("// Copy-paste this into your code");
            println!("// Generated by https://github.com/tsoding/domsson-fontgen from `{}`", file_path);
            pretty_print_bytes_as_c_array(&compressed_bytes, 16, "COMPRESSED_FONT");
        }
    }
}
