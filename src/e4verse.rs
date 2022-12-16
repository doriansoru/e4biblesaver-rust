use rand::{
    distributions::{Distribution, Standard},
    Rng,
};
use std::io::Error;

mod e4reader {
    use std::{
        fs::File,
        io::{self, prelude::*},
    };

    pub struct BufReader {
        reader: io::BufReader<File>,
    }

    impl BufReader {
        pub fn open(path: impl AsRef<std::path::Path>) -> io::Result<Self> {
            let file = File::open(path)?;
            let reader = io::BufReader::new(file);

            Ok(Self { reader })
        }

        pub fn read_line<'buf>(
            &mut self,
            buffer: &'buf mut String,
        ) -> Option<io::Result<&'buf mut String>> {
            buffer.clear();

            self.reader
                .read_line(buffer)
                .map(|u| if u == 0 { None } else { Some(buffer) })
                .transpose()
        }
    }
}

#[derive(Debug)]
pub struct E4Verse {
    pub height: i32,
    pub width: i32,
    pub line_length: i32,
    pub verse: String,
    pub x: i32,
    pub y: i32,
    pub direction: Direction,
}

#[derive(Debug)]
pub enum Direction {
    TopLeft,
    TopRight,
    BottomRight,
    BottomLeft,
}

impl Distribution<Direction> for Standard {
    fn sample<R: Rng + ?Sized>(&self, rng: &mut R) -> Direction {
        // match rng.gen_range(0, 3) { // rand 0.5, 0.6, 0.7
        match rng.gen_range(0..4) {
            // rand 0.8
            0 => Direction::TopLeft,
            1 => Direction::TopRight,
            2 => Direction::BottomRight,
            _ => Direction::BottomLeft,
        }
    }
}

impl E4Verse {
    pub fn new(width: i32, height: i32, line_length: i32, bible_path: String) -> Self {
        let v = Self::new_verse(line_length, bible_path).unwrap();
        let e4verse = Self {
            height: height,
            width: width,
            line_length: line_length,
            verse: v,
            x: 0,
            y: 0,
            direction: rand::random(),
        };
        e4verse
    }

    // Gets a random line from a file and returns as a string
    fn get_random_line(bible_path: &String) -> String {
        // Create a file buffer reader
        let mut reader = e4reader::BufReader::open(bible_path).unwrap();

        // Initializes the reservoir with a random line from the file
        let mut reservoir: Vec<String> = Vec::new();
        let mut buffer = String::new();
        if let Some(line) = reader.read_line(&mut buffer) {
            reservoir.push(line.unwrap().to_string());
        }

        // Iterates each line of the file file and replaces one line of the reservoir with probability 1/n
        let mut n = 1;
        while let Some(line) = reader.read_line(&mut buffer) {
            n += 1;
            let random_index = rand::random::<usize>() % n;
            if random_index < reservoir.len() {
                reservoir[random_index] = line.unwrap().to_string();
            }
        }

        // Returns one random line from the reservoir
        let random_index = rand::random::<usize>() % reservoir.len();
        reservoir[random_index].clone()
    }

    fn new_verse(line_length: i32, bible_path: String) -> Result<String, Error> {
        const BIBLE_SEPARATOR: &'static str = "|";

        //Select a random verse
        let verse = Self::get_random_line(&bible_path);

        let fields: Vec<&str> = verse.split(BIBLE_SEPARATOR).collect();
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
        let mut i: i32 = 0;
        formatted_verse = String::from("");
        for word in cloned_verse.split_whitespace() {
            let count: i32 = word.chars().count().try_into().unwrap();
            if (i + count) > line_length {
                formatted_verse.push('\n');
                i = 0;
            } else {
                i += count;
            }
            formatted_verse.push_str(word);
            formatted_verse.push(' ');
        }

        Ok(formatted_verse)
    }
}

impl std::fmt::Display for E4Verse {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "{}", &self.verse)
    }
}
