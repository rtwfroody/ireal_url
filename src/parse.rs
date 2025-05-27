use std::{fmt, vec};

use nom::{
    combinator::{all_consuming, map, opt},
    multi::many0,
    sequence::tuple,
    IResult,
};

use crate::{
    tokenize::{self, Token},
    types::Chord,
};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Bar {
    pub repeat_start: bool,
    pub counts: Box<[Vec<BarElement>]>,
}

impl Bar {
    pub fn new(count_count: u32) -> Self {
        let counts = vec![vec![]; count_count as usize];
        Bar {
            repeat_start: false,
            counts: counts.into_boxed_slice(),
        }
    }

    pub fn from_counts(counts: &[Vec<BarElement>]) -> Self {
        let counts: Vec<_> = counts.to_vec();
        let counts = counts.into_boxed_slice();
        Bar {
            repeat_start: false,
            counts,
        }
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

fn token<'a>(expected: Token) -> impl Fn(&'a [Token]) -> IResult<&'a [Token], Token> {
    move |input: &'a [Token]| {
        let (tok, remainder) = input.split_first().unwrap();
        if tok != &expected {
            Err(nom::Err::Error(nom::error::Error::new(
                input,
                nom::error::ErrorKind::Tag,
            )))
        } else {
            Ok((remainder, tok.clone()))
        }
    }
}

fn chord(input: &[Token]) -> IResult<&[Token], BarElement> {
    match input.first() {
        Some(Token::Chord(c)) => Ok((&input[1..], BarElement::Chord(c.clone()))),
        _ => Err(nom::Err::Error(nom::error::Error::new(
            input,
            nom::error::ErrorKind::Tag,
        ))),
    }
}

fn bar(input: &[Token]) -> IResult<&[Token], Bar> {
    // A simple bar is one or more chords.
    map(
        tuple((
            opt(token(Token::RepeatStart)),
            many0(chord),
            opt(token(Token::Bar)),
        )),
        |(repeat_start, chords, _)| {
            let repeat_start = repeat_start.is_some();
            Bar {
                repeat_start,
                counts: chords
                    .into_iter()
                    .map(|c| vec![c])
                    .collect::<Vec<_>>()
                    .into_boxed_slice(),
            }
        },
    )(input)
}

pub fn parse_music(text: &str) -> Result<Music, String> {
    let tokens = tokenize::tokenize(text)?;
    let bars = all_consuming(many0(bar))(&tokens).unwrap().1;
    Ok(Music {
        repeat_start: None,
        bars,
        raw: text.to_string(),
    })
}
