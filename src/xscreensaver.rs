use crate::e4verse::E4Verse;
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

const SCROLL_STEP: i32 = 3;
const FONTSIZE_FACTOR: f64 = 1000.0_f64;
const FPS: u64 = 20;

#[link(name = "X11")]
#[link(name = "Xft")]
extern "C" {}

pub struct ScreensaverSetup {
    dpy: *mut Display,
    root_window_id: Window,
    height: i32,
    width: i32,
    x: i32,
    y: i32,
    line_length: i32,
    font_size: i32,
    bible_path: String,
    duration: u64,
}

impl ScreensaverSetup {
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
        let dpy = unsafe { XOpenDisplay(libc::getenv(display_string.as_ptr())) };

        match xscreensaver_id {
            Some(root_window_id) => {
                // Use xscreensaver window
                let mut attrs = MaybeUninit::<XWindowAttributes>::uninit();
                unsafe {
                    XGetWindowAttributes(dpy, root_window_id, attrs.as_mut_ptr());
                }
                let attrs2 = unsafe { attrs.assume_init() };
                let g = unsafe { XCreateGC(dpy, root_window_id, 0, std::ptr::null_mut()) };
                unsafe {
                    XSetForeground(dpy, g, XWhitePixelOfScreen(XDefaultScreenOfDisplay(dpy)));
                }
                let g2 = unsafe { XCreateGC(dpy, root_window_id, 0, std::ptr::null_mut()) };
                unsafe {
                    XSetForeground(dpy, g2, XBlackPixelOfScreen(XDefaultScreenOfDisplay(dpy)));
                }

                // Calculate the font size in percentual of the window size
                let calculated_font_size: i32 =
                    ((attrs2.width as f64) * font_size as f64 / FONTSIZE_FACTOR).round() as i32;

                Ok(ScreensaverSetup {
                    dpy,
                    root_window_id,
                    height: attrs2.height,
                    width: attrs2.width,
                    x: -1,
                    y: -1,
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
                let screen = unsafe { XDefaultScreenOfDisplay(dpy) };
                let win = unsafe {
                    XCreateSimpleWindow(
                        dpy,
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

                let g = unsafe { XCreateGC(dpy, win, 0, std::ptr::null_mut()) };
                unsafe {
                    XSetForeground(dpy, g, XWhitePixelOfScreen(XDefaultScreenOfDisplay(dpy)));
                }
                let g2 = unsafe { XCreateGC(dpy, win, 0, std::ptr::null_mut()) };
                unsafe {
                    XSetForeground(dpy, g2, XBlackPixelOfScreen(XDefaultScreenOfDisplay(dpy)));
                }
                unsafe {
                    XMapWindow(dpy, win);
                }

                // Calculate the font size in percentual of the window size
                let calculated_font_size: i32 =
                    ((width as f64) * font_size as f64 / FONTSIZE_FACTOR).round() as i32;

                Ok(ScreensaverSetup {
                    dpy,
                    root_window_id: win,
                    height: height as i32,
                    width: width as i32,
                    x: -1,
                    y: -1,
                    line_length: line_length,
                    font_size: calculated_font_size,
                    bible_path: bible_path,
                    duration: speed,
                })
            }
        }
    }

    pub fn clear(&mut self, w: i32, h: i32) {
        unsafe {
            XClearArea(
                self.dpy,
                self.root_window_id,
                self.x,
                self.y,
                w as u32,
                h as u32,
                0_i32,
            );
        }
    }

    pub fn draw_e4verse(&mut self) {
        let step = 5;

        let mut rng = rand::thread_rng();
        // Get a verse
        let mut e4verse = E4Verse::new(
            self.width,
            self.height,
            self.line_length,
            self.bible_path.clone(),
        );
        let original_verse = e4verse.verse.clone();
        let mut attrs = MaybeUninit::<XWindowAttributes>::uninit();
        unsafe {
            XGetWindowAttributes(self.dpy, self.root_window_id, attrs.as_mut_ptr());
        }
        let attrs2 = unsafe { attrs.assume_init() };
        let win_ref = attrs2.visual;
        let screen_ptr = unsafe { XDefaultScreenOfDisplay(self.dpy) };
        let colormap = unsafe { (*screen_ptr).cmap };

        let draw = unsafe { XftDrawCreate(self.dpy, self.root_window_id, win_ref, colormap) };

        let white = XftColor {
            pixel: 0xFFFFFF, // Pixel value for white
            color: XRenderColor {
                red: 65535,
                green: 65535,
                blue: 65535,
                alpha: 65535,
            },
        };

        let screen_count = unsafe { XScreenCount(self.dpy) };

        if screen_count == 0 {
            // No screens for display...
            panic!("No screens found for current dpy.");
        }

        let screen_num = 0;
        let font_name = CString::new(format!("Sans-{}", self.font_size)).unwrap();
        let xftfont = unsafe { XftFontOpenName(self.dpy, screen_num, font_name.as_ptr()) };
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
                    self.dpy,
                    xftfont,
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
        self.x = rng.gen_range(0..text_width);
        self.y = rng.gen_range(0..verse_height);
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
                        xftfont,
                        self.x,
                        self.y + text_height * i,
                        line.as_ptr(),
                        line.len() as i32,
                    )
                };
            }
            // Flush everything
            unsafe { XFlush(self.dpy) };
            std::thread::sleep(frame_interval);
            self.clear(
                text_width,
                verse_height,
            );
            match e4verse.direction {
                crate::e4verse::Direction::TopLeft => {
                    self.x -= SCROLL_STEP;
                    self.y -= SCROLL_STEP;
                    if self.x < 0 && self.y < 0 {
                        e4verse.direction = rand::random();
                        self.x = 0;
                        self.y = 0;
                    } else {
                        if self.x < 0 {
                            self.x = 0;
                            e4verse.direction = crate::e4verse::Direction::TopRight;
                        } else if self.y < 0 {
                            self.y = 0;
                            e4verse.direction = crate::e4verse::Direction::BottomLeft;
                        }
                    } 
                },
                crate::e4verse::Direction::TopRight => {
                    self.x += SCROLL_STEP;
                    self.y -= SCROLL_STEP;
                    if (self.x + text_width) > self.width && self.y < 0 {
                        self.x = self.width - text_width;
                        self.y = 0;
                        e4verse.direction = rand::random();
                    } else {
                        if (self.x + text_width) > self.width {
                            self.x = self.width - text_width;
                            e4verse.direction = crate::e4verse::Direction::TopLeft;
                        } else if self.y < 0 {
                            self.y = 0;
                            e4verse.direction = crate::e4verse::Direction::BottomRight;
                        }
                    }                     
                },
                crate::e4verse::Direction::BottomRight => {
                    self.x += SCROLL_STEP;
                    self.y += SCROLL_STEP;
                    if (self.x + text_width) > self.width && (self.y + verse_height) > self.height {
                        self.x = self.width - text_height;
                        self.y = self.height - verse_height;
                        e4verse.direction = rand::random();
                    } else {
                        if (self.x + text_width) > self.width {
                            self.x = self.width - text_width;
                            e4verse.direction = crate::e4verse::Direction::BottomLeft;
                        } else if (self.y + verse_height) > self.height {
                            self.y = self.height - verse_height;
                            e4verse.direction = crate::e4verse::Direction::TopRight;
                        }
                    } 
                },
                crate::e4verse::Direction::BottomLeft => {
                    self.x -= SCROLL_STEP;
                    self.y += SCROLL_STEP;
                    if self.x < 0 && (self.y + verse_height) > self.height {
                        self.x = 0;
                        self.y = self.height - verse_height;
                        e4verse.direction = rand::random();
                    } else {
                        if self.x < 0 {
                            self.x = 0;
                            e4verse.direction = crate::e4verse::Direction::BottomRight;
                        } else if (self.y + verse_height) > self.height {
                            self.y = self.height - verse_height;
                            e4verse.direction = crate::e4verse::Direction::TopLeft;
                        }
                    }                     
                },
            }
        }
    }
}
