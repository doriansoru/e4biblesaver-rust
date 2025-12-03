mod bibleverse;
use bibleverse::BibleVerse;
use std::path::PathBuf;

mod biblescreensaver;
use biblescreensaver::ScreensaverSetup;

const DEFAULT_DURATION: u64 = 30;
const DEFAULT_LINE_LENGTH: i32 = 40;
const DEFAULT_FONT_SIZE: i32 = 5;

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
    let option_index_duration = args.clone().into_iter().position(|x| x == "-duration");
    let option_index_line_length = args.clone().into_iter().position(|x| x == "-line-length");
    let option_index_font_size = args.clone().into_iter().position(|x| x == "-font-size");
    let option_index_bible = args.clone().into_iter().position(|x| x == "-bible");

    let duration: Option<u64>;
    let line_length: Option<i32>;
    let font_size: Option<i32>;
    let bible_path: Option<String>;

    // Set default arguments if they are empty
    if let Some(index) = option_index_duration {
        duration = Some(args[index + 1].parse().unwrap());
    } else {
        duration = Some(DEFAULT_DURATION);
    }

    if let Some(index) = option_index_line_length {
        line_length = Some(args[index + 1].parse().unwrap());
    } else {
        line_length = Some(DEFAULT_LINE_LENGTH);
    }

    if let Some(index) = option_index_font_size {
        font_size = Some(args[index + 1].parse().unwrap());
    } else {
        font_size = Some(DEFAULT_FONT_SIZE);
    }

    if let Some(index) = option_index_bible {
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
        duration.unwrap(),
    ) {
        loop {
            s.draw_e4verse();
        }
    } else {
        let e4verse = BibleVerse::new(line_length.unwrap(), String::from(""));
        println!("{}", e4verse);
    }
}
