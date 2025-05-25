mod tokenize;
use tokenize::SimpleBar;
pub use tokenize::{
    Bar,
    BarElement,
    Chord,
    Music,
    Note,
    Number,
    Flavor
};

const MUSIC_PREFIX : &str = "1r34LbKcu7";

/*
 * Format reference: https://github.com/pianosnake/ireal-reader
 */

fn unscramble(mut text : &str) -> String {
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

fn obfusc50(text : &str) -> String {
    /* Directly translated from
     * https://github.com/pianosnake/ireal-reader/blob/ce643f069732ab93b1dcbd621b6c0edfe9ab8a8b/unscramble.js#L21 */

    // The first 5 characters are switched with the last 5.
    let mut chars : Vec<char> = text.chars().collect();
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

fn decode_music(text : &str) -> Result<Music, String> {
    if !text.starts_with(MUSIC_PREFIX) {
        return Err(format!("Music doesn't start with {}", MUSIC_PREFIX));
    }
    let unscrambled = unscramble(&text[MUSIC_PREFIX.len()..]);
    tokenize::tokenize_music(unscrambled.as_str())
}

fn hex_digit_value(ch : char) -> Result<u32, String> {
    match ch {
        '0' => Ok(0), '1' => Ok(1), '2' => Ok(2), '3' => Ok(3),
        '4' => Ok(4), '5' => Ok(5), '6' => Ok(6), '7' => Ok(7),
        '8' => Ok(8), '9' => Ok(9), 'a' => Ok(10), 'b' => Ok(11),
        'c' => Ok(12), 'd' => Ok(13), 'e' => Ok(14), 'f' => Ok(15),
        'A' => Ok(10), 'B' => Ok(11), 'C' => Ok(12), 'D' => Ok(13),
        'E' => Ok(14), 'F' => Ok(15),
        _ => Err(format!("Unknown value: {}", ch))
    }
}

fn unescape_percent(text : &str) -> Result<String, String>
{
    enum UnescapeState {
        Plain,
        Percent,
        One
    }

    let mut state = UnescapeState::Plain;
    let mut result = String::new();
    let mut num = 0;
    for c in text.chars() {
        match state {
            UnescapeState::Plain => match c {
                '%' => state = UnescapeState::Percent,
                _ => result.push(c)
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
    pub title : String,
    pub songs : Vec<Song>
}

#[derive(Debug, PartialEq, Clone, Eq)]
pub struct Song {
    pub title : String,
    pub composer : String,
    pub style : String,
    pub key : String,
    pub transpose : String,
    pub music : Music,
    pub comp_style : String,
    pub bpm : u32,
    pub repeats : String
}

impl Song {
    fn from_text(text : &str) -> Self {
        let parts : Vec<&str> = text.split("=").collect();
        println!("title: {}", parts[0]);
        Song {
            title: parts[0].to_string(),
            composer: parts[1].to_string(),
            style: parts[3].to_string(),
            key: parts[4].to_string(),
            transpose: parts[5].to_string(),
            music: decode_music(parts[6]).unwrap(),
            comp_style: parts[7].to_string(),
            bpm: parts[8].parse().unwrap(),
            repeats: parts[9].to_string()
        }
    }

    // Just turn this into a sequence of Chords
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
}

/* See https://loophole-letters.vercel.app/ireal-changes */
pub fn parse_url(mut text : &str) -> Result<Collection, String> {
    text = text.trim();
    if !text.starts_with("irealb://") {
        return Err("Expected URL to start with 'irealb://'".to_string())
    }

    let unescaped = unescape_percent(&text[9..])?;

    let mut parts : Vec<&str> = unescaped.split("===").collect();
    let collection_title =
            if parts.len() > 1 {
                parts.pop().unwrap()
            } else {
                "No Title"
            };
    let songs = parts.into_iter()
            .map(Song::from_text)
            .collect();
    Ok(Collection{
        title: collection_title.to_string(),
        songs})
}

#[cfg(test)]
mod tests {
    use nom::branch::Alt;

    use crate::tokenize::AlteredNotes;

    use super::*;

    #[test]
    fn work() {
        let text = "irealb://Work=Monk%20Thelonious==Medium%20Swing=Db==1r34LbK\
                cu7KQyX74Db7X7bEZL7E%207FZL%20lKcQyX7bGZL%20lcKQyXyQ%7CD4TA%2A%7B7F%7CQy%5\
                B%2ABD7L%20lcKQyX5b7C%7CQXy5b7GZL5b7G%20susZCh7X%7D%20%20lcFZL%20l7%20A7L7\
                bGZL%20lcKQyX7bCD%2A%5B%5DQyX5%239b7bAZXyQKcE%7CQyX7%20E7LZEb7XyQ%7CD7XyQK\
                cl%20Q%20ZY%7CQGXyQZ%20==0=0===";
        let result = parse_url(text).unwrap();

        #[allow(non_snake_case)]
        let Ab7b9s5 = BarElement::Chord(
            Chord::Some {
                root: Note::AFlat,
                flavor: Flavor::Dominant(Some(Number::Seven)),
                altered_notes: vec![AlteredNotes::Flat(Number::Nine), AlteredNotes::Sharp(Number::Five)],
                bass_note: None
            }
        );

        #[allow(non_snake_case)]
        let A7 = BarElement::Chord(
            Chord::Some {
                root: Note::A,
                flavor: Flavor::Dominant(Some(Number::Seven)),
                altered_notes: vec![],
                bass_note: None
            }
        );

        #[allow(non_snake_case)]
        let C7b5 = BarElement::Chord(
            Chord::Some {
                root: Note::C,
                flavor: Flavor::Dominant(Some(Number::Seven)),
                altered_notes: vec![AlteredNotes::Flat(Number::Five)],
                bass_note: None
            }
        );

        #[allow(non_snake_case)]
        let Ch7= BarElement::Chord(
            Chord::Some {
                root: Note::C,
                flavor: Flavor::HalfDiminished(Some(Number::Seven)),
                altered_notes: vec![],
                bass_note: None
            }
        );

        #[allow(non_snake_case)]
        let Db7 = BarElement::Chord(
            Chord::Some {
                root: Note::DFlat,
                flavor: Flavor::Dominant(Some(Number::Seven)),
                altered_notes: vec![],
                bass_note: None
            }
        );

        #[allow(non_snake_case)]
        let D7 = BarElement::Chord(
            Chord::Some {
                root: Note::D,
                flavor: Flavor::Dominant(Some(Number::Seven)),
                altered_notes: vec![],
                bass_note: None
            }
        );

        #[allow(non_snake_case)]
        let D7sus = BarElement::Chord(
            Chord::Some {
                root: Note::D,
                flavor: Flavor::Dominant(Some(Number::Seven)),
                altered_notes: vec![AlteredNotes::Sus],
                bass_note: None
            }
        );

        #[allow(non_snake_case)]
        let Eb7 = BarElement::Chord(
            Chord::Some {
                root: Note::EFlat,
                flavor: Flavor::Dominant(Some(Number::Seven)),
                altered_notes: vec![],
                bass_note: None
            }
        );

        #[allow(non_snake_case)]
        let E7 = BarElement::Chord(
            Chord::Some {
                root: Note::E,
                flavor: Flavor::Dominant(Some(Number::Seven)),
                altered_notes: vec![],
                bass_note: None
            }
        );

        #[allow(non_snake_case)]
        let F7 = BarElement::Chord(
            Chord::Some {
                root: Note::F,
                flavor: Flavor::Dominant(Some(Number::Seven)),
                altered_notes: vec![],
                bass_note: None
            }
        );

        #[allow(non_snake_case)]
        let Gb7 = BarElement::Chord(
            Chord::Some {
                root: Note::GFlat,
                flavor: Flavor::Dominant(Some(Number::Seven)),
                altered_notes: vec![],
                bass_note: None
            }
        );

        #[allow(non_snake_case)]
        let G = BarElement::Chord(
            Chord::Some {
                root: Note::G,
                flavor: Flavor::Dominant(None),
                altered_notes: vec![],
                bass_note: None
            }
        );

        #[allow(non_snake_case)]
        let G7b5 = BarElement::Chord(
            Chord::Some {
                root: Note::G,
                flavor: Flavor::Dominant(Some(Number::Seven)),
                altered_notes: vec![AlteredNotes::Flat(Number::Five)],
                bass_note: None
            }
        );

        assert_eq!(result, Collection {
            title: String::new(),
            songs: vec![
                Song {
                    title: "Work".to_string(),
                    composer: "Monk Thelonious".to_string(),
                    style: "Medium Swing".to_string(),
                    key: "Db".to_string(),
                    transpose: "".to_string(),
                    music: Music {
                        repeat_start: None,
                        raw: "{*AT44Db7XyQKcl LZGb7XyQKcl LZF7 E7LZEb7XyQ|D7XyQKcl  }\
                            [*BD7sus G7b5LZG7b5XyQ|C7b5XyQKcl LZCh7XyQ|F7XyQ|E7 A7LZAb7b9#5XyQ]\
                            [*CDb7XyQKcl LZGb7XyQKcl LZF7 E7LZEb7XyQ|D7XyQKcl Q ZY|QGXyQZ ".to_string(),
                        bars: vec![
                            Bar { elements: vec![Db7.clone()] },
                            Bar { elements: vec![Db7.clone()] },
                            Bar { elements: vec![Gb7.clone()] },
                            Bar { elements: vec![Gb7.clone()] },
                            Bar { elements: vec![F7.clone(), E7.clone()] },
                            Bar { elements: vec![Eb7.clone()] },
                            Bar { elements: vec![D7.clone()] },
                            Bar { elements: vec![D7.clone()] },
                            Bar { elements: vec![D7sus.clone(), G7b5.clone()] },
                            Bar { elements: vec![G7b5.clone()] },
                            Bar { elements: vec![C7b5.clone()] },
                            Bar { elements: vec![C7b5.clone()] },
                            Bar { elements: vec![Ch7.clone()] },
                            Bar { elements: vec![F7.clone()] },
                            Bar { elements: vec![E7.clone(), A7.clone()] },
                            Bar { elements: vec![Ab7b9s5.clone()] },
                            Bar { elements: vec![Db7.clone()] },
                            Bar { elements: vec![Db7.clone()] },
                            Bar { elements: vec![Gb7.clone()] },
                            Bar { elements: vec![Gb7.clone()] },
                            Bar { elements: vec![F7.clone(), E7.clone()] },
                            Bar { elements: vec![Eb7.clone()] },
                            Bar { elements: vec![D7.clone()] },
                            Bar { elements: vec![D7.clone()] },
                            Bar { elements: vec![] },
                            Bar { elements: vec![G.clone()] },
                    ]
            }, comp_style: "".to_string(), bpm: 0, repeats: "0".to_string() }]
        });
    }
}
