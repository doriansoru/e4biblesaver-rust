mod e4verse;
use e4verse::E4Verse;
use std::path::PathBuf;

mod xscreensaver;
use xscreensaver::ScreensaverSetup;

const DEFAULT_SPEED: u64 = 20;
const DEFAULT_LINE_LENGTH: i32 = 40;
const DEFAULT_FONT_SIZE: i32 = 30;

use std::ffi::CString;
use std::os::raw::c_char;

extern "C" {
    fn setlocale(category: i32, locale: *const c_char) -> *mut c_char;
}

fn main() {
    unsafe {
        let locale = CString::new("UTF-8").unwrap();
        setlocale(1, locale.as_ptr());
    }
    // Get arguments
    let args: Vec<String> = std::env::args().collect();
    let program_name = std::env::args()
        .next()
        .as_ref()
        .map(std::path::Path::new)
        .and_then(std::path::Path::file_name)
        .and_then(std::ffi::OsStr::to_str)
        .map(String::from)
        .unwrap();
    let option_index_speed = args.clone().into_iter().position(|x| x == "-speed");
    let option_index_line_length = args.clone().into_iter().position(|x| x == "-line-length");
    let option_index_font_size = args.clone().into_iter().position(|x| x == "-font-size");
    let option_index_bible = args.clone().into_iter().position(|x| x == "-bible");

    let speed: Option<u64>;
    let line_length: Option<i32>;
    let font_size: Option<i32>;
    let bible_path: Option<String>;

    // Set default arguments if they are empty
    if option_index_speed.is_some() {
        let index = option_index_speed.unwrap();
        speed = Some(args[index + 1].parse().unwrap());
    } else {
        speed = Some(DEFAULT_SPEED);
    }

    if option_index_line_length.is_some() {
        let index = option_index_line_length.unwrap();
        line_length = Some(args[index + 1].parse().unwrap());
    } else {
        line_length = Some(DEFAULT_LINE_LENGTH);
    }

    if option_index_font_size.is_some() {
        let index = option_index_font_size.unwrap();
        font_size = Some(args[index + 1].parse().unwrap());
    } else {
        font_size = Some(DEFAULT_FONT_SIZE);
    }

    if option_index_bible.is_some() {
        let index = option_index_bible.unwrap();
        bible_path = Some(args[index + 1].parse().unwrap());
    } else {
        let mut config_bible_path = PathBuf::new();

        config_bible_path.push("/opt");
        config_bible_path.push(&program_name);
        config_bible_path.push("bible.txt");
        bible_path = Some(String::from(config_bible_path.as_path().to_str().unwrap()));
    }

    // Ok, start
    if let Ok(mut s) = ScreensaverSetup::new(
        line_length.unwrap(),
        font_size.unwrap(),
        bible_path.unwrap(),
        speed.unwrap(),
    ) {
        loop {
            s.draw_e4verse();
        }
    } else {
        let e4verse = E4Verse::new(80, 30, line_length.unwrap(), String::from(""));
        println!("{}", e4verse);
    }
}
