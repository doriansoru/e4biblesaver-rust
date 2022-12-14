use crate::e4::E4;
use rand::Rng;
use std::ffi::CString;
use std::mem::MaybeUninit;
use x11::{
    xft,
    xlib::{
        Display, Window, XBlackPixelOfScreen, XCreateGC, XCreateSimpleWindow,
        XDefaultScreenOfDisplay, XFillRectangle, XFlush, XGetWindowAttributes, XMapWindow,
        XOpenDisplay, XRootWindowOfScreen, XSetForeground, XWhitePixelOfScreen, XWindowAttributes,
        GC,
    },
};

const SCROLL_STEP: i32 = 2;

#[link(name = "X11")]
extern "C" {}

pub struct ScreensaverSetup {
    dpy: *mut Display,
    root_window_id: Window,
    height: i32,
    width: i32,
    graphics_context: GC,
    black_graphics_context: GC,
    x: i32,
    y: i32,
    line_length: i32,
    font_size: i32,
    bible_path: String,
    speed: u64,
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
                    ((attrs2.width as f64) * font_size as f64 / 100.0_f64).round() as i32;

                Ok(ScreensaverSetup {
                    dpy,
                    root_window_id,
                    height: attrs2.height,
                    width: attrs2.width,
                    graphics_context: g,
                    black_graphics_context: g2,
                    x: -1,
                    y: -1,
                    line_length: line_length,
                    font_size: calculated_font_size,
                    bible_path: bible_path,
                    speed: speed,
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
                    ((width as f64) * font_size as f64 / 100.0).round() as i32;

                Ok(ScreensaverSetup {
                    dpy,
                    root_window_id: win,
                    height: height as i32,
                    width: width as i32,
                    graphics_context: g,
                    black_graphics_context: g2,
                    x: -1,
                    y: -1,
                    line_length: line_length,
                    font_size: calculated_font_size,
                    bible_path: bible_path,
                    speed: speed,
                })
            }
        }
    }

    pub fn clear(&mut self) {
        unsafe {
            x11::xlib::XClearWindow(self.dpy, self.root_window_id);
        }
    }

    pub fn draw_e4(&mut self) {
        let step = 10;

        let mut rng = rand::thread_rng();
        // Get a verse
        let e4 = E4::new(
            self.width,
            self.height,
            self.line_length,
            self.bible_path.clone(),
        );
        let original_verse = e4.verse.clone();
        let mut attrs = unsafe { std::mem::uninitialized() };
        unsafe {
            XGetWindowAttributes(self.dpy, self.root_window_id, &mut attrs);
        }
        let win_ref = attrs.visual;
        let screen_ptr = unsafe { XDefaultScreenOfDisplay(self.dpy) };
        let colormap = unsafe { (*screen_ptr).cmap };

        let draw = unsafe { xft::XftDrawCreate(self.dpy, self.root_window_id, win_ref, colormap) };

        let white = x11::xft::XftColor {
            pixel: 0xFFFFFF, // Pixel value for white
            color: x11::xrender::XRenderColor {
                red: 65535,
                green: 65535,
                blue: 65535,
                alpha: 65535,
            },
        };

        let screen_count = unsafe { x11::xlib::XScreenCount(self.dpy) };

        if screen_count == 0 {
            // No screens for display...
            panic!("No screens found for current dpy.");
        }

        let screen_num = 0;
        let font_name = CString::new(format!("Sans-{}", self.font_size)).unwrap();
        let xftfont =
            unsafe { x11::xft::XftFontOpenName(self.dpy, screen_num, font_name.as_ptr()) };
        let mut extents = x11::xrender::XGlyphInfo {
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
                x11::xft::XftTextExtents16(
                    self.dpy,
                    xftfont,
                    line.trim().as_ptr() as *const _,
                    line.trim().len() as i32,
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

        let mut to_width = self.width - text_width;

        if to_width < 0 || to_width > self.width {
            // Set max x to 1/3 of the window
            to_width = ((self.width / 3) as f64).round() as i32;
        }

        let mut to_height = self.height - (text_height * original_verse.lines().count() as i32);

        if to_height < 0 || to_height > self.height {
            // Set max y to 1/3 of the window
            to_height = ((self.height / 3) as f64).round() as i32;
        }

        let frame_interval = std::time::Duration::from_millis(self.speed);
        self.x = rng.gen_range(0..to_width);
        self.y = rng.gen_range(0..to_height);
        while self.x > (text_width * -1) {
            // Write text to screen
            let mut i = 0;
            for line in original_verse.lines() {
                i += 1;
                unsafe {
                    x11::xft::XftDrawStringUtf8(
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
            self.x -= SCROLL_STEP;
            std::thread::sleep(frame_interval);
            self.clear();
        }
    }
}
