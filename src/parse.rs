use std::{fmt, vec};

use nom::{
    branch::alt,
    combinator::{all_consuming, map, opt},
    multi::many0,
    sequence::tuple,
    IResult,
};

use crate::{
    tokenize::{self, Token, Width},
    types::Chord,
};

/* The chords that are playing in each bar. */
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Bar {
    // There's a double bar at the start of the bar.
    double_start: bool,
    // There's a double bar at the end of the bar.
    double_end: bool,
    pub repeat_start: bool,
    pub repeat_end: bool,
    pub markers: Vec<Marker>,
    pub counts: Vec<CountElement>,
}

impl Bar {
    pub fn new(count_count: usize) -> Self {
        let counts = vec![CountElement::None; count_count];
        Bar {
            double_start: false,
            double_end: false,
            repeat_start: false,
            repeat_end: false,
            markers: vec![],
            counts: counts,
        }
    }

    pub fn from_counts(counts: Vec<CountElement>) -> Self {
        let counts: Vec<_> = counts.to_vec();
        Bar {
            double_start: false,
            double_end: false,
            repeat_start: false,
            repeat_end: false,
            markers: vec![],
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
    move |input: &'a [Token]| match input.split_first() {
        None => Err(nom::Err::Error(nom::error::Error::new(
            input,
            nom::error::ErrorKind::Tag,
        ))),
        Some((tok, _)) if tok == &expected => Ok((&input[1..], tok.clone())),
        _ => Err(nom::Err::Error(nom::error::Error::new(
            input,
            nom::error::ErrorKind::Tag,
        ))),
    }
}

enum BarPrefixElement {
    RepeatStart,
    SectionMarker(String),
    NumberedEnding(String),
    TimeSignature(u32, u32),
    DoubleBarStart,
}

fn marker(input: &[Token]) -> IResult<&[Token], BarPrefixElement> {
    match input.first() {
        Some(Token::SectionMarker(s)) => {
            Ok((&input[1..], BarPrefixElement::SectionMarker(s.clone())))
        }
        Some(Token::NumberedEnding(s)) => {
            Ok((&input[1..], BarPrefixElement::NumberedEnding(s.clone())))
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

fn chord(input: &[Token]) -> IResult<&[Token], (Chord, Width)> {
    match input.first() {
        Some(Token::Chord(c, w)) => Ok((&input[1..], (c.clone(), w.clone()))),
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

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Marker {
    SectionMarker(String),
    NumberedEnding(String),
}

/** A simple bar is basically what a bar looks like on the page. */
struct SimpleBar {
    // There's a double bar at the start of the bar.
    double_start: bool,
    // There's a double bar at the end of the bar.
    double_end: bool,
    repeat_start: bool,
    repeat_end: bool,
    markers: Vec<Marker>,
    time_signature: Option<TimeSignature>,
    content: SimpleBarContent,
}

fn simple_bar(input: &[Token]) -> IResult<&[Token], SimpleBar> {
    // A simple bar is one or more chords.
    let (remainder, (prefixes, simple_bar_content, repeat_end, end)) = tuple((
        many0(alt((
            map(token(Token::RepeatStart), |_| BarPrefixElement::RepeatStart),
            map(token(Token::DoubleBarStart), |_| {
                BarPrefixElement::DoubleBarStart
            }),
            marker,
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
                        .flat_map(|(c, w)| match w {
                            Width::Narrow => vec![CountElement::Chord(c, vec![])],
                            Width::Wide => {
                                vec![CountElement::Chord(c, vec![]), CountElement::None]
                            }
                            Width::Unknown => panic!(
                                "At this stage chord width should be know for chord: {:?}",
                                c
                            ),
                        })
                        .collect(),
                )
            }),
        )),
        opt(token(Token::RepeatEnd)),
        alt((
            token(Token::DoubleBarEnd),
            token(Token::Bar),
            token(Token::FinalBar),
        )),
    ))(input)?;
    let mut simple_bar = SimpleBar {
        double_start: false,
        double_end: end == Token::DoubleBarEnd,
        repeat_start: false,
        repeat_end: repeat_end.is_some(),
        markers: vec![],
        time_signature: None,
        content: simple_bar_content,
    };
    for prefix in prefixes.iter() {
        match prefix {
            BarPrefixElement::RepeatStart => {
                simple_bar.repeat_start = true;
            }
            BarPrefixElement::SectionMarker(s) => {
                simple_bar.markers.push(Marker::SectionMarker(s.clone()));
            }
            BarPrefixElement::NumberedEnding(s) => {
                simple_bar.markers.push(Marker::NumberedEnding(s.clone()));
            }
            BarPrefixElement::TimeSignature(top, bottom) => {
                simple_bar.time_signature = Some(TimeSignature {
                    top: *top,
                    bottom: *bottom,
                });
            }
            BarPrefixElement::DoubleBarStart => {
                simple_bar.double_start = true;
            }
        }
    }
    Ok((remainder, simple_bar))
}

fn simplify(input: &[Token]) -> Vec<Token> {
    let mut output = vec![];
    let mut width = Width::Wide;
    for token in input.iter() {
        match token {
            // Remove all spacer tokens
            Token::Blank | Token::Space | Token::Comma => continue,
            // Replace BarAndRepeat with Bar and RepeatMeasure
            Token::BarAndRepeat => {
                output.push(Token::Bar);
                output.push(Token::RepeatMeasure);
            }
            // Turn Chords into wide/narrow chords based on Squeeze and Unsqueeze tokens.
            Token::Squeeze => {
                width = Width::Narrow;
            }
            Token::Unsqueeze => {
                width = Width::Wide;
            }
            Token::Chord(c, _) => {
                output.push(Token::Chord(c.clone(), width.clone()));
            }
            _ => {
                output.push(token.clone());
            }
        }
    }
    output
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
            double_start: simple_bar.double_start,
            double_end: simple_bar.double_end,
            repeat_start: simple_bar.repeat_start,
            repeat_end: simple_bar.repeat_end,
            markers: simple_bar.markers.clone(),
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
