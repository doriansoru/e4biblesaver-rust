use easy_reader::EasyReader;
use gtk::prelude::*;
use gtk::{EventBox, Fixed, Label, Window, WindowType};
use rand::Rng;
use std::{fs::File, io::Error};

struct Verse {
    text: String,
    markup: String,
    hdir: i32,
    vdir: i32,
}

impl Verse {
    const FACTOR: f32 = 0.16; // Verse text width is (FACTOR * screen width)%
    const INTERVAL: f64 = 0.001;
    const STEP: i32 = 2;
    const DURATION: f64 = 15.0;
    const BIBLE_SEPARATOR: &'static str = "|";

    fn new() -> Result<Self, Error> {
        let max_verse_line_len = 40;
        let bible = File::open(include!("bible.h"))?;
        let mut reader = EasyReader::new(bible)?;
        //Select the verse
        let verse = reader.random_line()?.unwrap();
        let fields: Vec<&str> = verse.split(Verse::BIBLE_SEPARATOR).collect();
        //fields[0] = book name; fields[1] = chapter number; fields[2] = verse number; fields[3] = verse text
        let mut formatted_verse: String = format!(
            "[{} {}:{}] {}",
            &(fields[0]).trim(),
            &(fields[1]).trim(),
            &(fields[2]).trim(),
            &(fields[3]).trim()
        );
        //Format the verse to max max_verse_line_len characters
        //by adding \n
        let cloned_verse = formatted_verse.clone();
        let mut i: usize = 0;
        formatted_verse = String::from("");
        for word in cloned_verse.split_whitespace() {
            let count: usize = word.chars().count();
            if (i + count) > max_verse_line_len {
                formatted_verse.push('\n');
                i = 0;
            } else {
                i += count;
            }
            formatted_verse.push_str(word);
            formatted_verse.push(' ');
        }
        let screen = gdk::Screen::default().unwrap();
        let w = screen.width();
        let width = (w as f32 * Verse::FACTOR).ceil() as i32;
        let verse_markup = format!("<span size=\"{}%\">{}</span>", width, formatted_verse);
        let mut rng = rand::thread_rng();
        let hdir: i32 = match rng.gen_range(0..2) {
            0 => -1,
            _ => 1, // 1 included
        };
        let vdir: i32 = match rng.gen_range(0..2) {
            0 => -1,
            _ => 1, // 1 included
        };

        let verse = Self {
            text: formatted_verse,
            markup: verse_markup,
            hdir,
            vdir,
        };
        Ok(verse)
    }
}

fn main() {
    assert!(!gtk::init().is_err(), "Can't init GTK");

    let screen = gdk::Screen::default().unwrap();
    let w = screen.width();
    let h = screen.height();
    // We create the main window.
    let window = Window::new(WindowType::Toplevel);
    window.set_title("E4BibleSaver");
    window.set_default_size(w, h);

    let label = Label::new(None);
    let container = Fixed::new();
    let mut verse: Verse = Verse::new().unwrap();
    label.set_markup(&verse.markup);
    container.put(&label, 0, 0);
    let eventbox = EventBox::new();

    // Necessary to catch mouse movements and keyboard press
    eventbox.set_property("can-focus", true);
    eventbox.grab_focus();
    eventbox.set_events(eventbox.events() | gdk::EventMask::POINTER_MOTION_MASK | gdk::EventMask::KEY_PRESS_MASK);
    eventbox.connect("motion-notify-event", false, |_| {
        gtk::main_quit();
        let result = glib::value::Value::from_type(glib::types::Type::BOOL);
        Some(result)
    });

    eventbox.connect("button-press-event", false, |_| {
        gtk::main_quit();
        let result = glib::value::Value::from_type(glib::types::Type::BOOL);
        Some(result)
    });

    eventbox.connect("key-press-event", false, |_| {
        gtk::main_quit();
        let result = glib::value::Value::from_type(glib::types::Type::BOOL);
        Some(result)
    });

    eventbox.add(&container);
    window.add(&eventbox);
    let mut duration: f64 = 0.0;
    //0 seconds, 0.INTERVAL_seconds nanoseconds
    let interval = std::time::Duration::new(0, (Verse::INTERVAL * 1_000_000_000_f64) as u32);
    // we are using a closure to capture the label (else we could also use a normal function)
    let update_verse = move || {
        let x = label.allocation().x();
        let y = label.allocation().y();
        //Get the label context width and height in pixels
        let layout = label.create_pango_layout(Some(&verse.text));
        layout.set_markup(&verse.markup);
        let (lw, lh) = layout.pixel_size();
        if verse.hdir == -1 {
            if x <= 0 {
                verse.hdir = 1;
            }
            if verse.vdir == -1 {
                if y <= 0 {
                    verse.vdir = 1;
                }
            } else if verse.vdir == 1 {
                if y >= h - lh - Verse::STEP {
                    verse.vdir = -1;
                }
            }
        } else {
            if x >= w - lw  - Verse::STEP {
                verse.hdir = -1;
            }
            if verse.vdir == -1 {
                if y <= 0 {
                    verse.vdir = 1;
                }
            } else if verse.vdir == 1 {
                if y >= h - lh - Verse::STEP {
                    verse.vdir = -1;
                }
            }
        }

        let new_x = x + (verse.hdir * Verse::STEP);
        let new_y = y + (verse.vdir * Verse::STEP);

        container.move_(&label, new_x, new_y);
        duration += Verse::INTERVAL;
        if duration >= Verse::DURATION {
            verse = Verse::new().unwrap();
            label.set_markup(&verse.markup);
            duration = 0.0;
        }
        // we could return glib::Continue(false) to stop after this tgtkick
        glib::Continue(true)
    };

    // executes the closure once every interval second
    glib::timeout_add_local(interval, update_verse);
    window.fullscreen();

    window.connect_delete_event(|_, _| {
        gtk::main_quit();
        Inhibit(false)
    });
    window.show_all();
    gtk::main();
}
