use std::os::raw::{c_char, c_uchar};
use std::ffi::CString;

type Pixel = u8;
const COMPS: i32 = 1;

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

fn usage() {
    eprintln!("Usage: ./fontgen [OPTIONS] <bitmap-font-spritesheet.png>");
    eprintln!("OPTIONS:");
    eprintln!("    -f <rust|c|bin>         Output format of the compressed font");
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
    const IMAGE_WIDTH: usize = 128;
    const IMAGE_HEIGHT: usize = 64;

    let (file_path, format) = {
        let mut file_path: Option<String> = None;
        let mut format = Format::Rust;

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
                },

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
            (file_path, format)
        } else {
            usage();
            eprintln!("ERROR: No input file path was provided");
            std::process::exit(1);
        }
    };

    let pixels: &[u8] = unsafe {
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

        std::slice::from_raw_parts(pixels, IMAGE_WIDTH * IMAGE_HEIGHT)
    };

    let compressed_bytes = compress_bytes_with_custom_rle(
        &compress_monochrome_pixels_into_bits(pixels));

    match format {
        Format::Rust => {
            println!("// Copy-paste this into your code");
            println!("// Generated by https://github.com/tsoding/domsson-fontgen from `{}`", file_path);
            pretty_print_bytes_as_rust_array(&compressed_bytes, 16, "COMPRESSED_FONT");
        }
        Format::Bin => {
            use std::io::Write;
            std::io::stdout().write(&compressed_bytes).expect("Could not output compressed bytes to stdout");
        }
        Format::C => {
            println!("// Copy-paste this into your code");
            println!("// Generated by https://github.com/tsoding/domsson-fontgen from `{}`", file_path);
            pretty_print_bytes_as_c_array(&compressed_bytes, 16, "COMPRESSED_FONT");
        }
    }
}
