use std::fs::File;
use std::io::{BufRead, BufReader};
use std::path::PathBuf;

pub fn get_word_at_position(line_num: u32, char_num: u32, file_path: PathBuf) -> Option<String> {
    let file = File::open(file_path).ok()?;
    let reader = BufReader::new(file);
    // Iterate over each line in the file
    for (i, line) in reader.lines().enumerate() {
        let Ok(line) = line else {
            continue;
        };
        // If this is the target line
        if i == line_num as usize {
            // If the target character is within the bounds of this line
            if char_num < line.len() as u32 {
                // Find the word that contains the target character
                return find_word(line, char_num);
            }
        }
    }
    None
}

fn find_word(line: String, position: u32) -> Option<String> {
    let Some(char_at_position) = line.get(position as usize .. position as usize + 1) else {
        return None;
    };
    if !char_at_position.chars().next().unwrap().is_alphabetic() {
        return None;
    };
    let line_after_position = &line[position as usize..].to_string();
    let line_before_position = &line[..position as usize]
        .to_string()
        .chars()
        .rev()
        .collect::<String>();
    let left_part_of_word =
        get_partial_word(line_before_position.to_string()).unwrap_or("".to_string());
    let right_part_of_word =
        get_partial_word(line_after_position.to_string()).unwrap_or("".to_string());
    let word = format!(
        "{}{}",
        left_part_of_word.chars().rev().collect::<String>(),
        right_part_of_word
    );
    Some(word)
}

fn get_partial_word(line: String) -> Option<String> {
    let mut chars = line.chars().peekable();
    let mut word = String::new();
    while let Some(c) = chars.next() {
        match c {
            'a'..='z' | 'A'..='Z' | '_' => {
                word.push(c);
                while let Some(&c) = chars.peek() {
                    match c {
                        'a'..='z' | 'A'..='Z' | '_' => {
                            word.push(c);
                            chars.next();
                        }
                        _ => break,
                    }
                }
                return Some(word);
            }
            ' ' | '\t' | '\r' | '\n' | '(' | ')' | ',' | ';' | '.' | '=' | '!' | '<' | '>'
            | '+' | '-' | ':' => {
                return Some(word);
            }
            _ => continue,
        }
    }
    None
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn should_be_able_to_find_word_in_a_line() {
        let line = "SELECT * FROM dbo.tbl_person;".to_string();
        let char_num = 10;
        let word = find_word(line.clone(), char_num);
        assert_eq!(word, Some("FROM".to_string()));

        let line = "SELECT * FROM dbo.tbl_person;".to_string();
        let char_num = 0;
        let word = find_word(line.clone(), char_num);
        assert_eq!(word, Some("SELECT".to_string()));

        let char_num = line.len() as u32 - 2;
        let word = find_word(line.clone(), char_num);
        assert_eq!(word, Some("tbl_person".to_string()));

        let char_num = line.len() as u32 - 1;
        let word = find_word(line.clone(), char_num);
        assert_eq!(word, None);

        let char_num = 6;
        let word = find_word(line.clone(), char_num);
        assert_eq!(word, None);

        let char_num = 17;
        let word = find_word(line.clone(), char_num);
        assert_eq!(word, None);
    }
    #[test]
    fn should_get_the_table_name_if_cursor_is_at_beggining_of_the_word() {
        let line = "SELECT PersonID FROM Persons;".to_string();
        let char_num = 21;
        let word = find_word(line.clone(), char_num);
        assert_eq!(word, Some("Persons".to_string()));
    }
}
