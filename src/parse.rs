use std::fmt;

use crate::{
    tokenize::{self, Token},
    types::Chord,
};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Bar {
    pub counts: Box<[Vec<BarElement>]>,
}

impl Bar {
    pub fn new(count_count: u32) -> Self {
        let counts = vec![vec![]; count_count as usize];
        Bar {
            counts: counts.into_boxed_slice(),
        }
    }

    pub fn from_counts(counts: &[Vec<BarElement>]) -> Self {
        let counts: Vec<_> = counts.to_vec();
        let counts = counts.into_boxed_slice();
        Bar { counts }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TimeSignature {
    pub top: u32,
    pub bottom: u32,
}

impl fmt::Display for TimeSignature {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "T{}{}", self.top, self.bottom)
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum BarElement {
    SectionMarker(String),
    TimeSignature(TimeSignature),
    // TODO: Chord(Chord, Vec<Chord>), // Chord and any alternate chords
    Chord(Chord),
    AlternateChord(Chord),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Music {
    pub repeat_start: Option<usize>,
    pub raw: String,
    pub bars: Vec<Bar>,
}

impl fmt::Display for BarElement {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            BarElement::SectionMarker(s) => s.fmt(f),
            BarElement::TimeSignature(ts) => ts.fmt(f),
            BarElement::Chord(c) => c.fmt(f),
            BarElement::AlternateChord(c) => {
                write!(f, "({})", c)
            }
        }
    }
}

pub fn parse_music(text: &str) -> Result<Music, String> {
    let mut bars = vec![];
    let time_signature = TimeSignature { top: 4, bottom: 4 }; // Default time signature
    let mut increment = 2;
    let mut bar = None;
    let mut count = 0;
    for token in tokenize::tokenize(text)? {
        println!("  Token: {:?}", token);
        match token {
            Token::Bar
            | Token::FinalBar
            | Token::DoubleBarEnd
            | Token::RepeatEnd
            | Token::DoubleBarStart => {
                if bar.is_some() {
                    bars.push(bar.unwrap());
                    bar = None;
                    count = 0;
                }
            }
            Token::Chord(c) => {
                if bar.is_none() {
                    bar = Some(Bar::new(time_signature.top));
                    count = 0;
                }
                bar.as_mut().unwrap().counts[count].push(BarElement::Chord(c));
                count += increment;
            }
            Token::AlternateChord(c) => {
                // The alternate chord applies to the previous chord we added.
                if bar.is_none() {
                    bar = Some(Bar::new(time_signature.top));
                    count = 0;
                }
                bar.as_mut().unwrap().counts[count].push(BarElement::AlternateChord(c));
            }
            Token::RepeatMeasure => {
                if bars.is_empty() {
                    return Err("Repeat measure at beginning of song".to_string());
                }
                let last_bar = bars.last().unwrap();
                bar = Some(last_bar.clone());
                count = 0;
            }
            Token::RepeatTwoMeasures => {
                if bars.len() < 2 {
                    return Err("Repeat 2 measures at beginning of song".to_string());
                }
                let a = bars[bars.len() - 2].clone();
                let b = bars[bars.len() - 1].clone();
                bars.push(a);
                bar = Some(b.clone());
                count = 0;
            }
            Token::BarAndRepeat => {
                if let Some(unwrapped_bar) = bar {
                    bars.push(unwrapped_bar);
                }
                let last_bar = bars.last().unwrap();
                bar = Some(last_bar.clone());
                count = 0;
            }
            Token::Squeeze => {
                increment = 1;
            }
            Token::Unsqueeze => {
                increment = 2;
            }
            ignore => println!("    Ignoring token: {:?}", ignore),
        }
    }
    Ok(Music {
        repeat_start: None,
        bars,
        raw: text.to_string(),
    })
}
