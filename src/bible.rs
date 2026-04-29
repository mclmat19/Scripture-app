use serde_json::Value;
use std::fs;

#[derive(Debug, Clone)]
pub struct Verse {
    pub number : u32,
    pub text: String,
}

#[derive(Debug, Clone)]
pub struct Chapter { 
    pub number: u32,
    pub verses: Vec<Verse>,
}

#[derive(Debug, Clone)]
pub struct Book {
    pub name: String,
    pub testament: Testament,
    pub chapters: Vec<Chapter>,
}

#[derive(Debug, Clone, PartialEq)]
pub enum Testament {
    Old,
    New,
}

impl Book {
    pub fn chapter_count(&self) -> usize {
        self.chapters.len()
    }
}

const OLD_TESTAMENT: &[&str] = &[
    "Genesis", "Exodus", "Leviticus", "Numbers", "Deuteronomy",
    "Joshua", "Judges", "Ruth", "1 Samuel", "2 Samuel",
    "1 Kings", "2 Kings", "1 Chronicles", "2 Chronicles",
    "Ezra", "Nehemiah", "Esther", "Job", "Psalms", "Proverbs",
    "Ecclesiastes", "Song of Solomon", "Isaiah", "Jeremiah",
    "Lamentations", "Ezekiel", "Daniel", "Hosea", "Joel", "Amos",
    "Obadiah", "Jonah", "Micah", "Nahum", "Habakkuk", "Zephaniah",
    "Haggai", "Zechariah", "Malachi",
];


const NEW_TESTAMENT: &[&str] = &[
    "Matthew", "Mark", "Luke", "John", "Acts",
    "Romans", "1 Corinthians", "2 Corinthians", "Galatians", "Ephesians",
    "Philippians", "Colossians", "1 Thessalonians", "2 Thessalonians",
    "1 Timothy", "2 Timothy", "Titus", "Philemon", "Hebrews",
    "James", "1 Peter", "2 Peter", "1 John", "2 John", "3 John",
    "Jude", "Revelation",
];

pub fn load_bible() -> Vec<Book> {
    let json_str = fs::read_to_string("ESV_bible.json").unwrap_or_else(|_| {
        fs::read_to_string("../src/ESV/ESV_bible.json").unwrap_or_else(|_| {
            eprintln!("Couldn't  find File");
            eprintln!("Put it in the TOML folder and try again.\n");
            std::process::exit(1);
        })
    });

    let data: Value = serde_json::from_str(&json_str).unwrap_or_else(|e| {
        eprintln!("\n Failed to parse: {}", e);
        std::process::exit(1);
    });
    
    let mut books: Vec<Book> = Vec::new();

    for name in OLD_TESTAMENT.iter().chain(NEW_TESTAMENT.iter()) {
        let testament = if OLD_TESTAMENT.contains(name) {
            Testament::Old
        } else {
            Testament::New
        };

        let book_data = match data.get(*name) {
            Some(v) => v,
            None => continue,            
        };
        
        let book_obj = match book_data.as_object() {
            Some(o) => o,
            None => continue,
        };

        let mut chapter_nums: Vec<u32> = book_obj
            .keys()
            .filter_map(|k| k.parse::<u32>().ok())
            .collect();
        chapter_nums.sort_unstable();

        let mut chapters: Vec<Chapter> = Vec::new();

        for ch_num in chapter_nums {
            let ch_data = match book_obj.get(&ch_num.to_string()) {
                Some(v) => v,
                None => continue,
            };
            let ch_obj = match ch_data.as_object() {
                Some(o) => o,
                None => continue,
            };

            let mut verse_nums: Vec<u32> = ch_obj
                .keys()
                .filter_map(|k| k.parse::<u32>().ok())
                .collect();
            verse_nums.sort_unstable();

            let mut verses: Vec<Verse> = Vec::new();

            for v_num in verse_nums {
                if let Some(text_val) = ch_obj.get(&v_num.to_string()) {
                    if let Some(text_str) = text_val.as_str() {
                        verses.push(Verse {
                            number: v_num,
                            text: text_str.to_string(),
                        });
                    }
                }
            }

            if !verses.is_empty() {
                chapters.push(Chapter {
                    number: ch_num,
                    verses,
                });
            }
        }

        if !chapters.is_empty() {
            books.push(Book {
                name: name.to_string(),
                testament,
                chapters,
            });
            
        }
    }
    if books.is_empty() {
        eprintln!("No books loaded sorry");
        std::process::exit(1);
    }

    books
}
