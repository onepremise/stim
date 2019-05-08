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
use std::env;

use std::io::BufReader;

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
        process_image(source.to_string());
    }
}

fn process_image(image: String) {
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

        process_slot(&mut r_channel_file, &mut red_lsb_slot);
        process_slot(&mut g_channel_file, &mut grn_lsb_slot);
        process_slot(&mut b_channel_file, &mut blu_lsb_slot);
    }

    red_target.save(string_red_ch.as_str()).unwrap();
    grn_target.save(string_grn_ch.as_str()).unwrap();
    blue_target.save(string_blue_ch.as_str()).unwrap();
    alpha_target.save(string_alpha_ch.as_str()).unwrap();

    println!("complete.");
}

fn process_slot(channel_file: &mut File, lsb_slot: &mut Vec<u8>) {
    if lsb_slot.len() == 8 {
        let lsb_result= transform_bytes(lsb_slot, false);
//        println!("result {:#b}", lsb_result);
        channel_file.write_all(&[lsb_result]).expect("Unable to write data");
//        exit(0);
    }
}

fn transform_bytes(lsb_slot: &mut Vec<u8>, big_endian: bool) -> u8 {
    let mut result = 0b0000_0000;

    for x in 1..9 {
        let byte=lsb_slot.get(x-1).unwrap();
//        println!("byte {}, {:#b}", x, *byte);
        if big_endian {
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
    let mut result = 0b0000_0000;



    result
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
            println!("result befor {:#b}", byte);
            let reversed= reverse_bits(*byte, swap_endian);
            println!("result after {:#b}", reversed);

            bytes_count += 1;
        }

        println!("swapped each bit for buffer size: {}", bytes_count);
    } else {
        r_file.write_all(&data).expect("Unable to write data");
    }

    println!("complete.");
}