use crate::bibleverse::BibleVerse;
use crate::bibleverse::Direction;
use rand::Rng;
use std::ffi::CString;
use std::mem::MaybeUninit;
use x11::{
    xft::{XftColor, XftDrawCreate, XftDrawStringUtf8, XftFontOpenName, XftTextExtentsUtf8},
    xlib::{
        Display, Window, XBlackPixelOfScreen, XClearArea, XCreateGC, XCreateSimpleWindow,
        XDefaultScreenOfDisplay, XFlush, XGetWindowAttributes, XMapWindow, XOpenDisplay,
        XRootWindowOfScreen, XScreenCount, XSetForeground, XWhitePixelOfScreen, XWindowAttributes,
    },
    xrender::{XGlyphInfo, XRenderColor},
};

// Move the verse of n pixels at each update
const FLUTTUATION_SIZE: i32 = 3;

// The real font size in pixels will be screen_width / FONTSIZE_FACTOR / give setting font width
const FONTSIZE_FACTOR: f64 = 10.0_f64;

// Update the verse each n milliseconds
const FPS: u64 = 50;

#[link(name = "X11")]
#[link(name = "Xft")]
extern "C" {}

pub struct ScreensaverSetup {
    display: *mut Display,
    window_id: Window,
    height: i32,
    width: i32,
    verse_x: i32,
    verse_y: i32,
    line_length: i32, // In characters
    font_size: i32, // In pixels
    bible_path: String,
    duration: u64,
}

impl ScreensaverSetup {
    fn calculate_font_size(w: f64, font_size: f64) -> i32 {
        (w / font_size / FONTSIZE_FACTOR).round() as i32
    }

    pub fn new(
        line_length: i32,
        font_size: i32,
        bible_path: String,
        speed: u64,
    ) -> Result<Self, ()> {
        let xscreensaver_id_str = std::env::var("XSCREENSAVER_WINDOW")
            .ok()
            .unwrap_or(String::new())
            .split_whitespace()
            .next()
            .unwrap_or("")
            .trim_start_matches("0x")
            .to_string();

        let xscreensaver_id = Window::from_str_radix(&xscreensaver_id_str, 16).ok();
        let display_string = CString::new("DISPLAY").unwrap();
        let display = unsafe { XOpenDisplay(libc::getenv(display_string.as_ptr())) };

        match xscreensaver_id {
            Some(root_window_id) => {
                // Use xscreensaver window
                let mut attrs = MaybeUninit::<XWindowAttributes>::uninit();
                unsafe {
                    XGetWindowAttributes(display, root_window_id, attrs.as_mut_ptr());
                }
                let attrs2 = unsafe { attrs.assume_init() };
                let g = unsafe { XCreateGC(display, root_window_id, 0, std::ptr::null_mut()) };
                unsafe {
                    XSetForeground(display, g, XWhitePixelOfScreen(XDefaultScreenOfDisplay(display)));
                }
                let g2 = unsafe { XCreateGC(display, root_window_id, 0, std::ptr::null_mut()) };
                unsafe {
                    XSetForeground(display, g2, XBlackPixelOfScreen(XDefaultScreenOfDisplay(display)));
                }

                // Calculate the font size in percentual of the window size
                let calculated_font_size: i32 =
                    Self::calculate_font_size(attrs2.width as f64, font_size as f64);

                Ok(ScreensaverSetup {
                    display,
                    window_id: root_window_id,
                    height: attrs2.height,
                    width: attrs2.width,
                    verse_x: -1,
                    verse_y: -1,
                    line_length: line_length,
                    font_size: calculated_font_size,
                    bible_path: bible_path,
                    duration: speed,
                })
            }
            None => {
                // Create a normal window, no xscreensaver started
                let height = 800;
                let width = 1200;
                let screen = unsafe { XDefaultScreenOfDisplay(display) };
                let win = unsafe {
                    XCreateSimpleWindow(
                        display,
                        XRootWindowOfScreen(screen),
                        0,
                        0,
                        width,
                        height,
                        10,
                        XBlackPixelOfScreen(screen),
                        XBlackPixelOfScreen(screen),
                    )
                };

                let g = unsafe { XCreateGC(display, win, 0, std::ptr::null_mut()) };
                unsafe {
                    XSetForeground(display, g, XWhitePixelOfScreen(XDefaultScreenOfDisplay(display)));
                }
                let g2 = unsafe { XCreateGC(display, win, 0, std::ptr::null_mut()) };
                unsafe {
                    XSetForeground(display, g2, XBlackPixelOfScreen(XDefaultScreenOfDisplay(display)));
                }
                unsafe {
                    XMapWindow(display, win);
                }

                // Calculate the font size in percentual of the window size
                let calculated_font_size: i32 =
                    Self::calculate_font_size(width as f64, font_size as f64);
                Ok(ScreensaverSetup {
                    display,
                    window_id: win,
                    height: height as i32,
                    width: width as i32,
                    verse_x: -1,
                    verse_y: -1,
                    line_length: line_length,
                    font_size: calculated_font_size,
                    bible_path: bible_path,
                    duration: speed,
                })
            }
        }
    }

    pub fn clear(&mut self, w: i32, h: i32) {
        // Boundary, in pixels, added to each coordinate
        let boundary: u32 = (self.width as f64 / 40.0_f64).round() as u32;
        unsafe {
            XClearArea(
                self.display,
                self.window_id,
                self.verse_x,
                self.verse_y,
                w as u32 + boundary,
                h as u32 + boundary,
                0_i32,
            );
        }
    }

    pub fn draw_e4verse(&mut self) {
        let step = 5;

        let mut rng = rand::rng();
        // Get a verse
        let mut e4verse = BibleVerse::new(
            self.width,
            self.height,
            self.line_length,
            self.bible_path.clone(),
        );
        let original_verse = e4verse.verse.clone();

        let mut attrs = MaybeUninit::<XWindowAttributes>::uninit();
        unsafe {
            XGetWindowAttributes(self.display, self.window_id, attrs.as_mut_ptr());
        }
        let attrs2 = unsafe { attrs.assume_init() };
        let win_ref = attrs2.visual;
        let screen_ptr = unsafe { XDefaultScreenOfDisplay(self.display) };
        let colormap = unsafe { (*screen_ptr).cmap };

        let draw = unsafe { XftDrawCreate(self.display, self.window_id, win_ref, colormap) };

        let white = XftColor {
            pixel: 0xFFFFFF, // Pixel value for white
            color: XRenderColor {
                red: 65535,
                green: 65535,
                blue: 65535,
                alpha: 65535,
            },
        };

        let screen_count = unsafe { XScreenCount(self.display) };

        if screen_count == 0 {
            // No screens for display...
            panic!("No screens found for current dpy.");
        }

        let screen_num = 0;
        let font_name = CString::new(format!("Sans-{}", self.font_size)).unwrap();
        let xft_font = unsafe { XftFontOpenName(self.display, screen_num, font_name.as_ptr()) };
        let mut extents = XGlyphInfo {
            width: 0,
            height: 0,
            x: 0,
            y: 0,
            xOff: 0,
            yOff: 0,
        };

        let mut text_width: i32 = 0;
        let mut text_height: i32 = 0;

        for line in original_verse.lines() {
            unsafe {
                XftTextExtentsUtf8(
                    self.display,
                    xft_font,
                    line.as_ptr() as *const _,
                    line.len() as i32,
                    &mut extents,
                )
            };

            let width = extents.width as i32;
            if width > text_width {
                text_width = width as i32;
            }

            let height = extents.height as i32;
            if height > text_height {
                text_height = height as i32;
            }
        }
        text_height += step;

        let verse_height = (text_height + step) * original_verse.lines().count() as i32;

        let frame_interval = std::time::Duration::from_millis(FPS);

        self.verse_x = rng.random_range(0..text_width);
        self.verse_y = rng.random_range(0..verse_height);
        let now = std::time::SystemTime::now();

        //while self.x > (text_width * -1) {
        while now.elapsed().unwrap().as_secs() < self.duration {
            // Write text to screen
            let mut i = 0;
            for line in original_verse.lines() {
                i += 1;
                unsafe {
                    XftDrawStringUtf8(
                        draw,
                        &white,
                        xft_font,
                        self.verse_x,
                        self.verse_y + text_height * i,
                        line.as_ptr(),
                        line.len() as i32,
                    )
                };
            }
            // Flush everything
            unsafe { XFlush(self.display) };
            std::thread::sleep(frame_interval);
            self.clear(text_width + step, verse_height);
            match e4verse.direction {
                crate::bibleverse::Direction::NorthWest => {
                    self.verse_x -= FLUTTUATION_SIZE;
                    self.verse_y -= FLUTTUATION_SIZE;
                    let index = rand::rng().random_range(0..=Direction::max());
                    let direction = Direction::from(index);

                    if self.verse_x < 0 && self.verse_y < 0 {
                        e4verse.direction = direction;
                        self.verse_x = 0;
                        self.verse_y = 0;
                    } else {
                        if self.verse_x < 0 {
                            self.verse_x = 0;
                            e4verse.direction = crate::bibleverse::Direction::NorthEeast;
                        } else if self.verse_y < 0 {
                            self.verse_y = 0;
                            e4verse.direction = crate::bibleverse::Direction::SouthWest;
                        }
                    }
                }
                crate::bibleverse::Direction::NorthEeast => {
                    self.verse_x += FLUTTUATION_SIZE;
                    self.verse_y -= FLUTTUATION_SIZE;
                    let index = rand::rng().random_range(0..=Direction::max());
                    let direction = Direction::from(index);

                    if (self.verse_x + text_width) > self.width && self.verse_y < 0 {
                        self.verse_x = self.width - text_width;
                        self.verse_y = 0;
                        e4verse.direction = direction;
                    } else {
                        if (self.verse_x + text_width) > self.width {
                            self.verse_x = self.width - text_width;
                            e4verse.direction = crate::bibleverse::Direction::NorthWest;
                        } else if self.verse_y < 0 {
                            self.verse_y = 0;
                            e4verse.direction = crate::bibleverse::Direction::SouthEeast;
                        }
                    }
                }
                crate::bibleverse::Direction::SouthEeast => {
                    self.verse_x += FLUTTUATION_SIZE;
                    self.verse_y += FLUTTUATION_SIZE;
                    let index = rand::rng().random_range(0..=Direction::max());
                    let direction = Direction::from(index);

                    if (self.verse_x + text_width) > self.width && (self.verse_y + verse_height) > self.height {
                        self.verse_x = self.width - text_height;
                        self.verse_y = self.height - verse_height;
                        e4verse.direction = direction;
                    } else {
                        if (self.verse_x + text_width) > self.width {
                            self.verse_x = self.width - text_width;
                            e4verse.direction = crate::bibleverse::Direction::SouthWest;
                        } else if (self.verse_y + verse_height) > self.height {
                            self.verse_y = self.height - verse_height;
                            e4verse.direction = crate::bibleverse::Direction::NorthEeast;
                        }
                    }
                }
                crate::bibleverse::Direction::SouthWest => {
                    self.verse_x -= FLUTTUATION_SIZE;
                    self.verse_y += FLUTTUATION_SIZE;
                    let index = rand::rng().random_range(0..=Direction::max());
                    let direction = Direction::from(index);

                    if self.verse_x < 0 && (self.verse_y + verse_height) > self.height {
                        self.verse_x = 0;
                        self.verse_y = self.height - verse_height;
                        e4verse.direction = direction;
                    } else {
                        if self.verse_x < 0 {
                            self.verse_x = 0;
                            e4verse.direction = crate::bibleverse::Direction::SouthEeast;
                        } else if (self.verse_y + verse_height) > self.height {
                            self.verse_y = self.height - verse_height;
                            e4verse.direction = crate::bibleverse::Direction::NorthWest;
                        }
                    }
                }
            }
        }
    }
}
