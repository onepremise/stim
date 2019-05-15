extern crate image;
extern crate clap;

use clap::{Arg, App, AppSettings};
use image::{ImageBuffer, GenericImageView, Rgba};
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

use std::process;
use std::str::FromStr;
//use std::cell::RefCell;

struct StegProcessor {
    file_name: String,
    h_buffer: Option<ImageBuffer<Rgba<u8>, Vec<u8>>>,
    lsb_bit_len: usize,
    lsb_byte: u8,
    find_content: bool,
    find_hidden_img: bool,
    r_channel_file: Option<File>,
    g_channel_file: Option<File>,
    b_channel_file: Option<File>
}

impl StegProcessor {

//    pub fn new(image: String) -> StegProcessor {
//        StegProcessor { image: image }
//    }

    pub fn set_path(&mut self, x: String) {
        self.file_name = x;
    }

    pub fn set_find_content(&mut self, c: bool) {
        self.find_content = c;
    }

    pub fn set_find_hidden_img(&mut self, h: bool) {
        self.find_hidden_img = h;
    }

    pub fn is_finding_content(&self) -> bool { self.find_content }
    pub fn is_finding_image(&self) -> bool { self.find_hidden_img }

    pub fn default() -> StegProcessor {
        StegProcessor {
            file_name: "".to_string(),
            h_buffer: None,
            lsb_bit_len:0,
            lsb_byte: 0,
            find_content: false,
            find_hidden_img: false,
            r_channel_file: None,
            g_channel_file: None,
            b_channel_file: None
        }

    }

    pub fn process_image(&mut self, swap_endian: bool, bit_len_str: String) {
        self.lsb_bit_len = usize::from_str(bit_len_str.as_str()).unwrap();

        self.validate_bit_len();

        self.lsb_byte = self.build_lsb_mask(self.lsb_bit_len);

        let string_red = self.file_name.clone() + &".red.lsb".to_string() + &bit_len_str + &".scan".to_string();
        let string_grn = self.file_name.clone() + &".grn.lsb".to_string() + &bit_len_str + &".scan".to_string();
        let string_blue = self.file_name.clone() + &".blu.lsb".to_string() + &bit_len_str + &".scan".to_string();

        let string_red_ch = self.file_name.clone() + &".red.jpg".to_string();
        let string_grn_ch = self.file_name.clone() + &".grn.jpg".to_string();
        let string_blue_ch = self.file_name.clone() + &".blu.jpg".to_string();
        let string_alpha_ch = self.file_name.clone() + &".alp.jpg".to_string();

        let path = Path::new(self.file_name.as_str());

        let r_path = Path::new(string_red.as_str());
        let g_path = Path::new(string_grn.as_str());
        let b_path = Path::new(string_blue.as_str());

        let source_img = image::open(&path).unwrap();

        if self.find_content {
            self.r_channel_file = Some(File::create(&r_path).unwrap());
            self.g_channel_file = Some(File::create(&g_path).unwrap());
            self.b_channel_file = Some(File::create(&b_path).unwrap());
        }

        let (width, height) = source_img.dimensions();

        println!("processing image: {}", self.file_name);
        println!("dimensions {:?}", (width, height));
        println!("{:?}", source_img.color());

        let mut red_target = ImageBuffer::new(width, height);
        let mut grn_target = ImageBuffer::new(width, height);
        let mut blue_target = ImageBuffer::new(width, height);
        let mut alpha_target = ImageBuffer::new(width, height);

        self.h_buffer = Some(ImageBuffer::new(width, height));

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

            if self.find_content {
                self.process_content(&mut red_lsb_slot, &mut grn_lsb_slot,
                                     &mut blu_lsb_slot, red_val, green_val, blue_val,
                                     swap_endian);
            }

            if self.find_hidden_img {
                self.merge_hidden_image(x, y, red_val,green_val,
                                        blue_val,alpha_val);
            }
        }

        if self.find_content {
            red_target.save(string_red_ch.as_str()).unwrap();
            grn_target.save(string_grn_ch.as_str()).unwrap();
            blue_target.save(string_blue_ch.as_str()).unwrap();
            alpha_target.save(string_alpha_ch.as_str()).unwrap();
        }

        if self.find_hidden_img {
            if let Some(ref mut b) = self.h_buffer {
                let h_img=self.file_name.clone() + &".recovered.lsb".to_string() + &bit_len_str + &".png".to_string();
                b.save(h_img).unwrap();
            }
        }

        println!("complete.");
    }

    fn validate_bit_len(&mut self) {
        match self.lsb_bit_len {
            1 | 2 | 3| 4 => println!("Using lsb length: {}", self.lsb_bit_len),
            _ => {
                println!("Invalid bit length: {}, currently only support 1,2,4", self.lsb_bit_len);
                process::exit(0);
            },
        }
    }

    fn build_lsb_mask(&mut self, num : usize) -> u8 {
        let mut result = 0b0000_0001;
        for _x in 1..num {
            result <<=1;
            result |= 0b0000_0001;
        }

        println!("using bit mask: 0b{:08b}", result);

        result
    }

    pub fn merge_hidden_image(&mut self, x: u32, y: u32, _red_val: u8,
                              _green_val: u8, _blue_val: u8, _alpha_val: u8) {
//        println!("byte before {}, 0b{:08b}", 8/self.lsb_bit_len, (self.lsb_byte & _red_val));
        let new_red=(self.lsb_byte & _red_val) << (8-self.lsb_bit_len);
        let new_grn=(self.lsb_byte & _green_val) << (8-self.lsb_bit_len);
        let new_blu=(self.lsb_byte & _blue_val) << (8-self.lsb_bit_len);
//        println!("byte after {}, 0b{:08b}", 8/self.lsb_bit_len, new_red);
        let new_pixel = image::Rgba([new_red, new_grn, new_blu, _alpha_val]);

        if let Some(ref mut b) = self.h_buffer {
            b.put_pixel(x, y, new_pixel);
        }
    }

    fn process_content(&mut self, red_lsb_slot : &mut Vec<u8>, grn_lsb_slot : &mut Vec<u8>,
                       blu_lsb_slot : &mut Vec<u8>, red_val: u8, green_val: u8, blue_val: u8,
                       swap_endian: bool) {
        red_lsb_slot.push(red_val & self.lsb_byte);
        grn_lsb_slot.push(green_val & self.lsb_byte);
        blu_lsb_slot.push(blue_val & self.lsb_byte);

        self.process_slot("red", red_lsb_slot, swap_endian);
        self.process_slot("green", grn_lsb_slot, swap_endian);
        self.process_slot("blue", blu_lsb_slot, swap_endian);
    }

    fn process_slot(&mut self, channel: &str, lsb_slot: &mut Vec<u8>, swap_endian: bool) {
        // 8/3 results in floor return, 2
        if lsb_slot.len() == (8/self.lsb_bit_len) {
            let lsb_result= self.transform_bytes(lsb_slot, swap_endian);
//        println!("result {:#b}", lsb_result);
            match channel.as_ref() {
                "red" => {
                    if let Some(ref mut _file) = self.r_channel_file {
                        _file.write_all(&[lsb_result]).expect("Unable to write data");
                    }
                },
                "green" => {
                    if let Some(ref mut _file) = self.g_channel_file {
                        _file.write_all(&[lsb_result]).expect("Unable to write data");
                    }
                },
                "blue" => {
                    if let Some(ref mut _file) = self.b_channel_file {
                        _file.write_all(&[lsb_result]).expect("Unable to write data");
                    }
                },
                _ => {
                    println!("Channel type required!");
                    process::exit(0);
                }
            }
        }
    }

    fn transform_bytes(&mut self, lsb_slot: &mut Vec<u8>, swap_endian: bool) -> u8 {
        let mut result = 0b0000_0000;

        for x in 1..(8/self.lsb_bit_len)+1 {
            let byte=lsb_slot.get(x-1).unwrap();
//            println!("byte before {}, 0b{:08b}", x, *byte);
            result |= *byte << 8 - (x*self.lsb_bit_len);
//            println!("byte after {}, 0b{:08b}", x, result);
        }

        lsb_slot.clear();

        if swap_endian {
            self.reverse_bits(result)
        } else {
            result
        }

//        process::exit(0);
    }

    fn reverse_bits(&mut self, byte: u8) -> u8 {
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

        let step1 = byte & 0xff;
        let step2 = table[step1 as usize];

//    println!("step1 0b{:08b}", step1);
//    println!("step2 0b{:08b}", step2);

        step2
    }

    fn reverse_file(&mut self, file_loc: String, swap_endian: bool) {
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
                let reversed= self.reverse_bits(*byte);
                r_file.write_all(&[reversed]).expect("Unable to write data");
                bytes_count += 1;
            }

            println!("swapped each bit for buffer size: {}", bytes_count);
        } else {
            r_file.write_all(&data).expect("Unable to write data");
        }

        println!("complete.");
    }
}

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

    .arg(Arg::with_name("lsb_content")
        .short("c")
        .long("content")
        .value_name("FILE")
        .help("Extract content using least significant bit technique")
        .takes_value(true))
    .arg(Arg::with_name("lsb_image")
            .short("i")
            .long("lsb_image")
            .value_name("FILE")
            .help("Extract a hidden image using least significant bit technique: \nhttps://towardsdatascience.com/steganography-hiding-an-image-inside-another-77ca66b2acb1")
            .takes_value(true))
    .arg(Arg::with_name("lsb_bit_len")
            .short("l")
            .long("lsb_bit_len")
            .value_name("FILE")
            .help("Number of lsb bits your would like to use. 1, 2, 4, bits, default is 1.")
            .default_value("1")
            .takes_value(true))
    .arg(Arg::with_name("switch_endian")
        .short("sw")
        .long("switch_endian")
        .value_name("bool")
        .help("Swap the current byte representation LSB to MSB")
        .takes_value(false));

    let matches = app.get_matches();

    let mut steg_processor = StegProcessor::default();

    let bit_len = matches.value_of("lsb_bit_len").unwrap();

    if let Some(source) = matches.value_of("reverse") {
        steg_processor.set_path(source.to_string());
        steg_processor.reverse_file(source.to_string(), matches.is_present("switch_endian"));
    }

    if let Some(source) = matches.value_of("lsb_content") {
        steg_processor.set_path(source.to_string());
        steg_processor.set_find_content(true);
    }

    if let Some(source) = matches.value_of("lsb_image") {
        steg_processor.set_path(source.to_string());
        steg_processor.set_find_hidden_img(true);
    }

    if steg_processor.is_finding_content() || steg_processor.is_finding_image() {
        steg_processor.process_image(matches.is_present("switch_endian"),
                                     bit_len.to_string());
    }
}

//#[cfg(test)]
//mod tests {
//
//    #[test]
//    fn test_add() {
//        assert_eq!(add(1, 2), 3);
//    }
//}