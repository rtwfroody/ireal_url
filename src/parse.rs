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
        write!(f, "{}/{}", self.top, self.bottom)
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
    pub raw: String,
    pub written_bars: Vec<WrittenBar>,
}

impl fmt::Display for Music {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        for (i, bar) in self.written_bars.iter().enumerate() {
            if i % 4 == 0 && i > 0 {
                writeln!(f, "|")?;
            }
            write!(f, "{}", bar)?;
        }
        Ok(())
    }
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
        Some((tok, _)) if tok == &expected => Ok((&input[1..], tok.clone())),
        _ => Err(nom::Err::Error(nom::error::Error::new(
            input,
            nom::error::ErrorKind::Tag,
        ))),
    }
}

fn non_consuming_token<'a>(expected: Token) -> impl Fn(&'a [Token]) -> IResult<&'a [Token], Token> {
    move |input: &'a [Token]| match input.first() {
        Some(tok) if tok == &expected => Ok((input, tok.clone())),
        _ => Err(nom::Err::Error(nom::error::Error::new(
            input,
            nom::error::ErrorKind::Tag,
        ))),
    }
}

enum BarPrefixElement {
    RepeatStart,
    SectionMarker(String),
    NumberedEnding(u32),
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

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Marker {
    SectionMarker(String),
    NumberedEnding(u32),
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
            Token::Chord(c) => {
                output.push(Token::Chord(c.clone()));
            }
            _ => {
                output.push(token.clone());
            }
        }
    }
    output
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum WrittenElement {
    SectionMarker(String),
    TimeSignature(TimeSignature),
    Chord(Chord, Width),
    NumberedEnding(u32),
    RepeatMeasure,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct WrittenBar {
    repeat_start: bool,
    repeat_end: bool,
    double_start: bool,
    double_end: bool,
    elements: Vec<WrittenElement>,
}

impl WrittenBar {
    pub fn new() -> Self {
        WrittenBar {
            repeat_start: false,
            repeat_end: false,
            double_start: false,
            double_end: false,
            elements: vec![],
        }
    }

    pub fn repeat() -> Self {
        WrittenBar {
            repeat_start: false,
            repeat_end: false,
            double_start: false,
            double_end: false,
            elements: vec![WrittenElement::RepeatMeasure],
        }
    }
}

impl fmt::Display for WrittenBar {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "|")?;
        if self.double_start {
            write!(f, "|")?;
        }
        if self.repeat_start {
            write!(f, ":")?;
        }
        write!(f, " ")?;
        let mut count = 0;
        for (i, element) in self.elements.iter().enumerate() {
            if i > 0 {
                write!(f, " ")?;
            }
            match element {
                WrittenElement::SectionMarker(s) => write!(f, "[{}]", s)?,
                WrittenElement::TimeSignature(ts) => write!(f, "{}", ts)?,
                WrittenElement::Chord(c, width) => {
                    let s = c.to_string();
                    if *width == Width::Narrow {
                        write!(f, "{:>6}", s)?;
                        count += 1;
                    } else {
                        write!(f, "{:>6}      ", s)?;
                        count += 2;
                    }
                }
                WrittenElement::NumberedEnding(n) => write!(f, "N{}", n)?,
                WrittenElement::RepeatMeasure => write!(f, "            %           ")?,
            }
        }
        // TODO: Take time signature into account
        while (count < 4) {
            write!(f, "      ")?;
            count += 1;
        }
        if self.repeat_end {
            write!(f, ":")?;
        }
        if self.double_end {
            write!(f, "|")?;
        }
        // Assume there's a | for the next line
        Ok(())
    }
}

pub fn parse_music(text: &str) -> Result<Music, String> {
    // Remove blanks before parsing
    println!("Text: {}", text);
    let tokens = tokenize::tokenize(text)?;
    println!("Tokens: {:?}", tokens);

    let mut written_bars = vec![];
    let mut written_bar = WrittenBar::new();
    let mut width = Width::Wide;
    for token in tokens.iter() {
        match token {
            Token::RepeatStart => {
                written_bar.repeat_start = true;
            },
            Token::RepeatEnd => {
                written_bar.repeat_end = true;
            },
            Token::SectionMarker(s) => {
                written_bar.elements.push(WrittenElement::SectionMarker(s.clone()));
            },
            Token::TimeSignature(top, bottom) => {
                written_bar.elements.push(WrittenElement::TimeSignature(TimeSignature {
                    top: *top,
                    bottom: *bottom,
                }));
            },
            Token::Chord(c) => {
                written_bar.elements.push(WrittenElement::Chord(c.clone(), width.clone()));
            },
            Token::Comma | Token::Space | Token::Blank => {
                // Ignore these tokens
            },
            Token::Bar => {
                written_bars.push(written_bar);
                written_bar = WrittenBar::new();
            },
            Token::Squeeze => {
                width = Width::Narrow;
            },
            Token::Unsqueeze => {
                width = Width::Wide;
            },
            Token::NumberedEnding(n) => {
                written_bar.elements.push(WrittenElement::NumberedEnding(*n));
            },
            Token::DoubleBarStart => {
                written_bar.double_start = true;
            },
            Token::DoubleBarEnd => {
                written_bar.double_end = true;
            },
            Token::BarAndRepeat => {
                written_bars.push(written_bar);
                written_bar = WrittenBar::repeat();
            },
            Token::FinalBar => {
                written_bars.push(written_bar);
                break; // Final bar, stop processing
            },
            _ => panic!("Unexpected token: {:?}", token),
        }
    }

    Ok(Music {
        written_bars,
        raw: text.to_string(),
    })
}
