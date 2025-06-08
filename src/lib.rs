mod parse;
mod tokenize;
mod types;
use parse::Music;

const MUSIC_PREFIX: &str = "1r34LbKcu7";

/*
 * Format reference: https://github.com/pianosnake/ireal-reader
 */

fn unscramble(mut text: &str) -> String {
    /* Directly translated from
     * https://github.com/pianosnake/ireal-reader/blob/ce643f069732ab93b1dcbd621b6c0edfe9ab8a8b/unscramble.js#L5 */
    let mut result = String::new();

    while text.len() > 50 {
        let (part, remainder) = text.split_at(50);
        text = remainder;

        if text.len() < 2 {
            result.push_str(part);
        } else {
            result.push_str(&obfusc50(part));
        }
    }

    result.push_str(text);
    result
}

fn obfusc50(text: &str) -> String {
    /* Directly translated from
     * https://github.com/pianosnake/ireal-reader/blob/ce643f069732ab93b1dcbd621b6c0edfe9ab8a8b/unscramble.js#L21 */

    // The first 5 characters are switched with the last 5.
    let mut chars: Vec<char> = text.chars().collect();
    let last = chars.len() - 1;
    for i in 0..5 {
        chars.swap(i, last - i);
    }
    // Characters 10-24 are also switched.
    for i in 10..24 {
        chars.swap(i, last - i);
    }

    chars.into_iter().collect::<String>()
}

fn decode_music(text: &str) -> Result<Music, String> {
    if !text.starts_with(MUSIC_PREFIX) {
        return Err(format!("Music doesn't start with {}", MUSIC_PREFIX));
    }
    let unscrambled = unscramble(&text[MUSIC_PREFIX.len()..]);
    parse::parse_music(unscrambled.as_str())
}

fn hex_digit_value(ch: char) -> Result<u32, String> {
    match ch {
        '0' => Ok(0),
        '1' => Ok(1),
        '2' => Ok(2),
        '3' => Ok(3),
        '4' => Ok(4),
        '5' => Ok(5),
        '6' => Ok(6),
        '7' => Ok(7),
        '8' => Ok(8),
        '9' => Ok(9),
        'a' => Ok(10),
        'b' => Ok(11),
        'c' => Ok(12),
        'd' => Ok(13),
        'e' => Ok(14),
        'f' => Ok(15),
        'A' => Ok(10),
        'B' => Ok(11),
        'C' => Ok(12),
        'D' => Ok(13),
        'E' => Ok(14),
        'F' => Ok(15),
        _ => Err(format!("Unknown value: {}", ch)),
    }
}

fn unescape_percent(text: &str) -> Result<String, String> {
    enum UnescapeState {
        Plain,
        Percent,
        One,
    }

    let mut state = UnescapeState::Plain;
    let mut result = String::new();
    let mut num = 0;
    for c in text.chars() {
        match state {
            UnescapeState::Plain => match c {
                '%' => state = UnescapeState::Percent,
                _ => result.push(c),
            },
            UnescapeState::Percent => {
                num = 16 * hex_digit_value(c)?;
                state = UnescapeState::One
            }
            UnescapeState::One => {
                num += hex_digit_value(c)?;
                result.push(char::from_u32(num).unwrap());
                state = UnescapeState::Plain
            }
        }
    }
    Ok(result)
}

#[derive(Debug, PartialEq)]
pub struct Collection {
    pub title: String,
    pub songs: Vec<Song>,
}

#[derive(Debug, PartialEq, Clone, Eq)]
pub struct Song {
    pub title: String,
    pub composer: String,
    pub style: String,
    pub key: String,
    pub transpose: String,
    pub music: Music,
    pub comp_style: String,
    pub bpm: u32,
    pub repeats: String,
}

impl Song {
    fn from_text(text: &str) -> Self {
        let parts: Vec<&str> = text.split("=").collect();
        println!();
        println!("Title: {}", parts[0]);
        let song = Song {
            title: parts[0].to_string(),
            composer: parts[1].to_string(),
            style: parts[3].to_string(),
            key: parts[4].to_string(),
            transpose: parts[5].to_string(),
            music: decode_music(parts[6]).unwrap(),
            comp_style: parts[7].to_string(),
            bpm: parts[8].parse().unwrap(),
            repeats: parts[9].to_string(),
        };
        println!("Music:\n{}", song.music);
        song
    }

    // Just turn this into a sequence of Chords
    /*
    fn expand(&self) -> Vec<SimpleBar> {
        self.music.bars.iter()
            .map(|bar| {
                SimpleBar {
                    chords: bar.elements.iter()
                        .filter_map(|el| {
                            if let BarElement::Chord(chord) = el {
                                Some(chord.clone())
                            } else {
                                None
                            }
                        })
                        .collect(),
                }
            })
            .collect()
    }
    */
}

/* See https://loophole-letters.vercel.app/ireal-changes */
pub fn parse_url(mut text: &str) -> Result<Collection, String> {
    text = text.trim();
    if !text.starts_with("irealb://") {
        return Err("Expected URL to start with 'irealb://'".to_string());
    }

    let unescaped = unescape_percent(&text[9..])?;

    let mut parts: Vec<&str> = unescaped.split("===").collect();
    let collection_title = if parts.len() > 1 {
        parts.pop().unwrap()
    } else {
        "No Title"
    };
    let songs = parts.into_iter().map(Song::from_text).collect();
    Ok(Collection {
        title: collection_title.to_string(),
        songs,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use pretty_assertions::assert_eq;

    #[test]
    fn work() {
        let text = "irealb://Work=Monk%20Thelonious==Medium%20Swing=Db==1r34LbK\
                cu7KQyX74Db7X7bEZL7E%207FZL%20lKcQyX7bGZL%20lcKQyXyQ%7CD4TA%2A%7B7F%7CQy%5\
                B%2ABD7L%20lcKQyX5b7C%7CQXy5b7GZL5b7G%20susZCh7X%7D%20%20lcFZL%20l7%20A7L7\
                bGZL%20lcKQyX7bCD%2A%5B%5DQyX5%239b7bAZXyQKcE%7CQyX7%20E7LZEb7XyQ%7CD7XyQK\
                cl%20Q%20ZY%7CQGXyQZ%20==0=0===";
        let result = parse_url(text).unwrap();

        assert_eq!(result.title, String::new());
        let song = &result.songs[0];
        assert_eq!(song.title, "Work".to_string());
        assert_eq!(song.composer, "Monk Thelonious".to_string());
        assert_eq!(song.style, "Medium Swing".to_string());
        assert_eq!(song.key, "Db".to_string());
        assert_eq!(format!("{}", song.music),
"|: [A] 4/4    Db7                  |             %           |    Gb7                  |             %           |
|     F7           E7      |    Eb7                  |     D7                  |             %           :|
|| [B]  D7sus         G7b5      |   G7b5                  |   C7b5                  |             %           |
|    Ch7                  |     F7                  |     E7           A7      | Ab7b9#5                  ||
|| [C]    Db7                  |             %           |    Gb7                  |             %           |
|     F7           E7      |    Eb7                  |     D7                  |             %            𝄌|
| 𝄌      G                  |
");
    }

    #[test]
    fn all_jazz() {
        use std::fs;

        // Try to parse every jazz song
        let content = fs::read_to_string("src/tests/data/jazz1460.url").unwrap();
        parse_url(&content).unwrap();
    }
}
