use std::{fmt, vec};

use nom::{
    branch::alt,
    combinator::{all_consuming, map},
    multi::many0,
    sequence::tuple,
    IResult,
};

use crate::{
    tokenize::{self, Token},
    types::Chord,
};

/* The chords that are playing in each bar. */
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Bar {
    pub repeat_start: bool,
    pub section_marker: Option<String>,
    pub counts: Vec<CountElement>,
}

impl Bar {
    pub fn new(count_count: usize) -> Self {
        let counts = vec![CountElement::None; count_count as usize];
        Bar {
            repeat_start: false,
            section_marker: None,
            counts: counts,
        }
    }

    pub fn from_counts(counts: Vec<CountElement>) -> Self {
        let counts: Vec<_> = counts.to_vec();
        Bar {
            repeat_start: false,
            section_marker: None,
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
pub enum CountElement {
    None,
    Chord(Chord, Vec<Chord>), // Chord and any alternate chords
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

enum BarPrefixElement {
    RepeatStart,
    SectionMarker(String),
    TimeSignature(u32, u32),
}

fn section_marker(input: &[Token]) -> IResult<&[Token], BarPrefixElement> {
    match input.first() {
        Some(Token::SectionMarker(s)) => {
            Ok((&input[1..], BarPrefixElement::SectionMarker(s.clone())))
        }
        _ => Err(nom::Err::Error(nom::error::Error::new(
            input,
            nom::error::ErrorKind::Tag,
        ))),
    }
}

fn time_signature(input: &[Token]) -> IResult<&[Token], BarPrefixElement> {
    match input.first() {
        Some(Token::TimeSignature(top, bottom)) => {
            Ok((&input[1..], BarPrefixElement::TimeSignature(*top, *bottom)))
        }
        _ => Err(nom::Err::Error(nom::error::Error::new(
            input,
            nom::error::ErrorKind::Tag,
        ))),
    }
}

fn chord(input: &[Token]) -> IResult<&[Token], Chord> {
    match input.first() {
        Some(Token::Chord(c)) => Ok((&input[1..], c.clone())),
        _ => Err(nom::Err::Error(nom::error::Error::new(
            input,
            nom::error::ErrorKind::Tag,
        ))),
    }
}

enum SimpleBarContent {
    RepeatMeasure,
    Counts(Vec<CountElement>),
}

/** A simple bar is basically what a bar looks like on the page. */
struct SimpleBar {
    repeat_start: bool,
    section_marker: Option<String>,
    time_signature: Option<TimeSignature>,
    content: SimpleBarContent,
}

fn simple_bar(input: &[Token]) -> IResult<&[Token], SimpleBar> {
    // A simple bar is one or more chords.
    let (remainder, (prefixes, simple_bar_content, end)) = tuple((
        many0(alt((
            map(token(Token::RepeatStart), |_| BarPrefixElement::RepeatStart),
            section_marker,
            time_signature,
        ))),
        alt((
            map(token(Token::RepeatMeasure), |_| {
                SimpleBarContent::RepeatMeasure
            }),
            map(many0(chord), |chords| {
                SimpleBarContent::Counts(
                    // TODO: Here we need to deal with putting counts in the right place.
                    chords
                        .into_iter()
                        .map(|c| CountElement::Chord(c, vec![]))
                        .collect(),
                )
            }),
        )),
        token(Token::Bar),
    ))(input)
    .unwrap();
    let mut simple_bar = SimpleBar {
        repeat_start: false,
        section_marker: None,
        time_signature: None,
        content: simple_bar_content,
    };
    for prefix in prefixes.iter() {
        match prefix {
            BarPrefixElement::RepeatStart => {
                simple_bar.repeat_start = true;
            }
            BarPrefixElement::SectionMarker(s) => {
                simple_bar.section_marker = Some(s.clone());
            }
            BarPrefixElement::TimeSignature(top, bottom) => {
                simple_bar.time_signature = Some(TimeSignature {
                    top: *top,
                    bottom: *bottom,
                });
            }
        }
    }
    Ok((remainder, simple_bar))
}

fn simplify(input: &[Token]) -> Vec<Token> {
    input
        .iter()
        // Remove all spacer tokens
        .filter(|t| !matches!(t, Token::Blank | Token::Space | Token::Comma))
        // Replace BarAndRepeat with Bar and RepeatMeasure
        .flat_map(|t| match t {
            Token::BarAndRepeat => vec![Token::Bar, Token::RepeatMeasure],
            _ => vec![t.clone()],
        })
        .collect()
}

pub fn parse_music(text: &str) -> Result<Music, String> {
    // Remove blanks before parsing
    println!("Text: {}", text);
    let tokens = simplify(tokenize::tokenize(text)?.as_slice());
    println!("Tokens: {:?}", tokens);
    let simple_bars = all_consuming(many0(simple_bar))(&tokens).unwrap().1;
    let mut bars = vec![];
    let mut previous_bar: Option<Bar> = None;
    let mut time_signature = TimeSignature { top: 4, bottom: 4 };
    for simple_bar in simple_bars {
        if let Some(ts) = simple_bar.time_signature {
            time_signature = ts;
        }

        let counts = match simple_bar.content {
            SimpleBarContent::RepeatMeasure => previous_bar.unwrap().counts.clone(),
            SimpleBarContent::Counts(chords) => chords,
        };

        let bar = Bar {
            repeat_start: simple_bar.repeat_start,
            section_marker: simple_bar.section_marker,
            counts,
        };

        bars.push(bar.clone());
        previous_bar = Some(bar);
    }
    Ok(Music {
        repeat_start: None,
        bars,
        raw: text.to_string(),
    })
}
