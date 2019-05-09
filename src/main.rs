extern crate image;
extern crate clap;

use clap::{Arg, App, AppSettings};
use image::{ImageBuffer, GenericImageView};
use std::fs::File;
use std::io::Read;
use std::io::Write;
//use std::io::BufReader;
use std::path::Path;
//use std::process::exit;
//use image::GenericImageView

//use std::ffi::OsString;
//use std::ffi::OsStr;
//use std::convert::AsRef;

fn main() {
    let app = App::new("stim")
        .setting(AppSettings::ColorAuto)
        .setting(AppSettings::ColoredHelp)
        .setting(AppSettings::DeriveDisplayOrder)
        .setting(AppSettings::UnifiedHelpMessage)
        .version("0.1")
        .about("Steganographic processor for finding hidden content in files")
    .arg(Arg::with_name("reverse")
        .short("r")
        .long("reverse")
        .value_name("FILE")
        .help("reverse content of file, from tail to head")
        .takes_value(true))
    .arg(Arg::with_name("lsb_tech")
        .short("l")
        .long("lsb")
        .value_name("FILE")
        .help("Extract content using least significant bit technique")
        .takes_value(true))
    .arg(Arg::with_name("switch_endian")
        .short("sw")
        .long("switch_endian")
        .value_name("bool")
        .help("Swap the current byte representation LSB to MSB")
        .takes_value(false));

    let matches = app.get_matches();

//    let lsb_tech = matches.value_of("lsb_tech");

    if let Some(source) = matches.value_of("reverse") {
        reverse_file(source.to_string(), matches.is_present("switch_endian"));
    }

    if let Some(source) = matches.value_of("lsb_tech") {
        process_image(source.to_string(),matches.is_present("switch_endian"));
    }
}

fn process_image(image: String, swap_endian: bool) {
    let string_red = image.clone() + &".red.txt".to_string();
    let string_grn = image.clone() + &".grn.txt".to_string();
    let string_blue = image.clone() + &".blu.txt".to_string();
//    let string_alpha = image.clone() + &".alp.txt".to_string();

    let string_red_ch = image.clone() + &".red.jpg".to_string();
    let string_grn_ch = image.clone() + &".grn.jpg".to_string();
    let string_blue_ch = image.clone() + &".blu.jpg".to_string();
    let string_alpha_ch = image.clone() + &".alp.jpg".to_string();

    let path = Path::new(image.as_str());

    let r_path = Path::new(string_red.as_str());
    let g_path = Path::new(string_grn.as_str());
    let b_path = Path::new(string_blue.as_str());

    let source_img = image::open(&path).unwrap();

    let mut r_channel_file = File::create(&r_path).unwrap();
    let mut g_channel_file = File::create(&g_path).unwrap();
    let mut b_channel_file = File::create(&b_path).unwrap();
//    let result = jpeg::JPEGDecoder::new(f);
//    let decoder = result.unwrap();

    let (width, height) = source_img.dimensions();

    println!("processing image: {}", image);
    println!("dimensions {:?}", (width, height));
    println!("{:?}", source_img.color());

    let mut red_target = ImageBuffer::new(width, height);
    let mut grn_target = ImageBuffer::new(width, height);
    let mut blue_target = ImageBuffer::new(width, height);
    let mut alpha_target = ImageBuffer::new(width, height);

    let lsb_byte: u8 = 0b0000_0001;
    let lsb2_byte: u8 = 0b0000_0011;

    let mut red_lsb_slot : Vec<u8> = Vec::new();
    let mut grn_lsb_slot : Vec<u8> = Vec::new();
    let mut blu_lsb_slot : Vec<u8> = Vec::new();

    for (x, y, pixel) in alpha_target.enumerate_pixels_mut() {
        let src_pixel = source_img.get_pixel(x, y);

        let red_val = src_pixel.data[0];
        let green_val = src_pixel.data[1];
        let blue_val = src_pixel.data[2];
        let alpha_val = src_pixel.data[3];

        let red_only = image::Rgba([red_val, 0, 0, alpha_val]);
        let grn_only = image::Rgba([0, green_val, 0, alpha_val]);
        let blu_only = image::Rgba([0, 0, blue_val, alpha_val]);
        let alpha_only = image::Rgba([0, 0, 0, alpha_val]);

        *pixel=alpha_only;

        red_target.put_pixel(x, y, red_only);
        grn_target.put_pixel(x, y, grn_only);
        blue_target.put_pixel(x, y, blu_only);

        red_lsb_slot.push(red_val & lsb_byte);
        grn_lsb_slot.push(green_val & lsb_byte);
        blu_lsb_slot.push(blue_val & lsb_byte);

        process_slot(&mut r_channel_file, &mut red_lsb_slot, swap_endian);
        process_slot(&mut g_channel_file, &mut grn_lsb_slot, swap_endian);
        process_slot(&mut b_channel_file, &mut blu_lsb_slot, swap_endian);
    }

    red_target.save(string_red_ch.as_str()).unwrap();
    grn_target.save(string_grn_ch.as_str()).unwrap();
    blue_target.save(string_blue_ch.as_str()).unwrap();
    alpha_target.save(string_alpha_ch.as_str()).unwrap();

    println!("complete.");
}

fn process_slot(channel_file: &mut File, lsb_slot: &mut Vec<u8>, swap_endian: bool) {
    if lsb_slot.len() == 8 {
        let lsb_result= transform_bytes(lsb_slot, swap_endian);
//        println!("result {:#b}", lsb_result);
        channel_file.write_all(&[lsb_result]).expect("Unable to write data");
//        exit(0);
    }
}

fn transform_bytes(lsb_slot: &mut Vec<u8>, swap_endian: bool) -> u8 {
    let mut result = 0b0000_0000;

    for x in 1..9 {
        let byte=lsb_slot.get(x-1).unwrap();
//        println!("byte {}, {:#b}", x, *byte);
        if swap_endian {
            result |= *byte << x-1;
        } else {
            result |= *byte << 8 - x;
        }
    }

//    println!("result before {:#b}", result);

//    println!("result after {:#b}", result.rotate_right(7));

    lsb_slot.clear();

    result
}

fn reverse_bits(byte: u8, big_endian: bool) -> u8 {
    let table: [u8; 256] = [
        0x00, 0x80, 0x40, 0xC0, 0x20, 0xA0, 0x60, 0xE0, 0x10, 0x90, 0x50, 0xD0, 0x30, 0xB0, 0x70, 0xF0,
        0x08, 0x88, 0x48, 0xC8, 0x28, 0xA8, 0x68, 0xE8, 0x18, 0x98, 0x58, 0xD8, 0x38, 0xB8, 0x78, 0xF8,
        0x04, 0x84, 0x44, 0xC4, 0x24, 0xA4, 0x64, 0xE4, 0x14, 0x94, 0x54, 0xD4, 0x34, 0xB4, 0x74, 0xF4,
        0x0C, 0x8C, 0x4C, 0xCC, 0x2C, 0xAC, 0x6C, 0xEC, 0x1C, 0x9C, 0x5C, 0xDC, 0x3C, 0xBC, 0x7C, 0xFC,
        0x02, 0x82, 0x42, 0xC2, 0x22, 0xA2, 0x62, 0xE2, 0x12, 0x92, 0x52, 0xD2, 0x32, 0xB2, 0x72, 0xF2,
        0x0A, 0x8A, 0x4A, 0xCA, 0x2A, 0xAA, 0x6A, 0xEA, 0x1A, 0x9A, 0x5A, 0xDA, 0x3A, 0xBA, 0x7A, 0xFA,
        0x06, 0x86, 0x46, 0xC6, 0x26, 0xA6, 0x66, 0xE6, 0x16, 0x96, 0x56, 0xD6, 0x36, 0xB6, 0x76, 0xF6,
        0x0E, 0x8E, 0x4E, 0xCE, 0x2E, 0xAE, 0x6E, 0xEE, 0x1E, 0x9E, 0x5E, 0xDE, 0x3E, 0xBE, 0x7E, 0xFE,
        0x01, 0x81, 0x41, 0xC1, 0x21, 0xA1, 0x61, 0xE1, 0x11, 0x91, 0x51, 0xD1, 0x31, 0xB1, 0x71, 0xF1,
        0x09, 0x89, 0x49, 0xC9, 0x29, 0xA9, 0x69, 0xE9, 0x19, 0x99, 0x59, 0xD9, 0x39, 0xB9, 0x79, 0xF9,
        0x05, 0x85, 0x45, 0xC5, 0x25, 0xA5, 0x65, 0xE5, 0x15, 0x95, 0x55, 0xD5, 0x35, 0xB5, 0x75, 0xF5,
        0x0D, 0x8D, 0x4D, 0xCD, 0x2D, 0xAD, 0x6D, 0xED, 0x1D, 0x9D, 0x5D, 0xDD, 0x3D, 0xBD, 0x7D, 0xFD,
        0x03, 0x83, 0x43, 0xC3, 0x23, 0xA3, 0x63, 0xE3, 0x13, 0x93, 0x53, 0xD3, 0x33, 0xB3, 0x73, 0xF3,
        0x0B, 0x8B, 0x4B, 0xCB, 0x2B, 0xAB, 0x6B, 0xEB, 0x1B, 0x9B, 0x5B, 0xDB, 0x3B, 0xBB, 0x7B, 0xFB,
        0x07, 0x87, 0x47, 0xC7, 0x27, 0xA7, 0x67, 0xE7, 0x17, 0x97, 0x57, 0xD7, 0x37, 0xB7, 0x77, 0xF7,
        0x0F, 0x8F, 0x4F, 0xCF, 0x2F, 0xAF, 0x6F, 0xEF, 0x1F, 0x9F, 0x5F, 0xDF, 0x3F, 0xBF, 0x7F, 0xFF
    ];

    let step1 = (byte & 0xff);
    let step2 = table[step1 as usize];

//    println!("step1 0b{:08b}", step1);
//    println!("step2 0b{:08b}", step2);

    step2
}

fn reverse_file(file_loc: String, swap_endian: bool) {
    println!("processing: {}", file_loc);

    let string_reverse = file_loc.clone() + &".reversed.bin".to_string();
    let r_path = Path::new(string_reverse.as_str());

    let path = Path::new(file_loc.as_str());

    let mut file = File::open(&path).unwrap();
    let mut r_file = File::create(&r_path).unwrap();

    println!("reading {}, size: {}", file_loc, file.metadata().unwrap().len());

    let mut data: Vec<u8> = Vec::with_capacity(file.metadata().unwrap().len() as usize);

    file.read_to_end(&mut data).unwrap();

    println!("reversing {}, buffer size: {}", file_loc, data.len());

    data.reverse();

    if swap_endian {
        let mut data_iter = data.iter();
        let mut bytes_count: i32 = 0;

        while let Some(byte) = data_iter.next() {
            let reversed= reverse_bits(*byte, swap_endian);
            r_file.write_all(&[reversed]).expect("Unable to write data");
            bytes_count += 1;
        }

        println!("swapped each bit for buffer size: {}", bytes_count);
    } else {
        r_file.write_all(&data).expect("Unable to write data");
    }

    println!("complete.");
}

fn test_write(file_loc: String) {
    let string_reverse = file_loc.clone() + &".test.jpg".to_string();
    let r_path = Path::new(string_reverse.as_str());

    let path = Path::new(file_loc.as_str());

    let mut file = File::open(&path).unwrap();
    let mut r_file = File::create(&r_path).unwrap();

    let mut data: Vec<u8> = Vec::with_capacity(file.metadata().unwrap().len() as usize);

    file.read_to_end(&mut data).unwrap();

    let mut data_iter = data.iter();
    let mut bytes_count: i32 = 0;

    while let Some(byte) = data_iter.next() {
        r_file.write_all(&[*byte]).expect("Unable to write data");
        bytes_count += 1;
    }
    println!("bytes written: {}", bytes_count);
}