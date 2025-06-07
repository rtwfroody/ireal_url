// Reference: https://www.irealpro.com/ireal-pro-custom-chord-chart-protocol

use nom::branch::alt;
use nom::bytes::complete::tag;
use nom::bytes::complete::take;
use nom::bytes::complete::take_until;
use nom::character::complete::digit1;
use nom::combinator::all_consuming;
use nom::combinator::map;
use nom::multi::many0;
use nom::sequence::tuple;
use nom::IResult;

use crate::types::AlteredNotes;
use crate::types::Chord;
use crate::types::Flavor;
use crate::types::Note;
use crate::types::Number;

#[derive(Debug, Clone, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub enum Width {
    Wide,
    Narrow,
    Unknown,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub enum Token {
    AlternateChord(Chord), // These show up above the regular music.
    Bar,
    Blank,
    Chord(Chord, Width),
    Coda,
    Comment(String), // There is some stuff inside the comment that we could probably parse.
    DoubleBarEnd,
    DoubleBarStart,
    EndingMeasure,
    FinalBar,
    NumberedEnding(String),
    PauseSlash,
    RepeatEnd,
    RepeatMeasure,
    BarAndRepeat,
    RepeatTwoMeasures,
    RepeatStart,
    SectionMarker(String),
    Segno,
    TimeSignature(u32, u32),
    VerticalSpace,
    Fermata,

    Squeeze,
    Unsqueeze,

    // I think this is like in BiaB. Comma is to put two chords in the first
    // half of a measure, and space is just to have one chord in each half.
    Comma,
    Space,
}

fn section_marker<'a>() -> impl FnMut(&'a str) -> IResult<&'a str, Token> {
    map(tuple((tag("*"), take(1usize))), |x: (&str, &str)| {
        Token::SectionMarker(x.1.to_string())
    })
}

fn numbered_ending<'a>() -> impl FnMut(&'a str) -> IResult<&'a str, Token> {
    map(tuple((tag("N"), take(1usize))), |x: (&str, &str)| {
        Token::NumberedEnding(x.1.to_string())
    })
}

fn comment<'a>() -> impl FnMut(&'a str) -> IResult<&'a str, Token> {
    map(
        tuple((tag("<"), take_until(">"), tag(">"))),
        |x: (&str, &str, &str)| Token::Comment(x.1.to_string()),
    )
}

fn alternate<'a>() -> impl FnMut(&'a str) -> IResult<&'a str, Token> {
    map(
        tuple((tag("("), chord(), tag(")"))),
        |x: (&str, Chord, &str)| Token::AlternateChord(x.1),
    )
}

fn note<'a>() -> impl FnMut(&'a str) -> IResult<&'a str, Note> {
    // Put longer strings first.
    alt((
        map(tag("Ab"), |_| Note::AFlat),
        map(tag("A#"), |_| Note::ASharp),
        map(tag("A"), |_| Note::A),
        map(tag("Bb"), |_| Note::BFlat),
        map(tag("B"), |_| Note::B),
        map(tag("Cb"), |_| Note::CFlat),
        map(tag("C#"), |_| Note::CSharp),
        map(tag("C"), |_| Note::C),
        map(tag("Db"), |_| Note::DFlat),
        map(tag("D#"), |_| Note::DSharp),
        map(tag("D"), |_| Note::D),
        map(tag("Eb"), |_| Note::EFlat),
        map(tag("E"), |_| Note::E),
        map(tag("F#"), |_| Note::FSharp),
        map(tag("F"), |_| Note::F),
        map(tag("Gb"), |_| Note::GFlat),
        map(tag("G#"), |_| Note::GSharp),
        map(tag("G"), |_| Note::G),
        map(tag("W"), |_| Note::W),
    ))
}

fn number<'a>() -> impl FnMut(&'a str) -> IResult<&'a str, Number> {
    alt((
        map(tag("2"), |_| Number::Two),
        map(tag("3"), |_| Number::Three),
        map(tag("5"), |_| Number::Five),
        map(tag("6"), |_| Number::Six),
        map(tag("7"), |_| Number::Seven),
        map(tag("9"), |_| Number::Nine),
        map(tag("11"), |_| Number::Eleven),
        map(tag("13"), |_| Number::Thirteen),
    ))
}

fn number_option<'a>() -> impl FnMut(&'a str) -> IResult<&'a str, Option<Number>> {
    alt((map(number(), Some), map(tag(""), |_| None)))
}

fn flavor<'a>() -> impl FnMut(&'a str) -> IResult<&'a str, Flavor> {
    // Put longer strings first.
    alt((
        map(tag("69"), |_| Flavor::SixthNinth),
        map(tag("-69"), |_| Flavor::MinorSixthNinth),
        map(tuple((tag("-^"), number_option())), |x| {
            Flavor::MinorMajor(x.1)
        }),
        map(tuple((tag("-"), number_option())), |x| Flavor::Minor(x.1)),
        map(tuple((tag("^"), number_option())), |x| Flavor::Major(x.1)),
        map(tuple((tag("h"), number_option())), |x| {
            Flavor::HalfDiminished(x.1)
        }),
        map(tuple((tag("o^"), number_option())), |x| {
            Flavor::DiminishedMajor(x.1)
        }),
        map(tuple((tag("o"), number_option())), |x| {
            Flavor::Diminished(x.1)
        }),
        map(tuple((tag("+"), number_option())), |x| {
            Flavor::Augmented(x.1)
        }),
        map(number_option(), Flavor::Dominant),
    ))
}

fn altered_notes<'a>() -> impl FnMut(&'a str) -> IResult<&'a str, Vec<AlteredNotes>> {
    many0(alt((
        map(tuple((tag("b"), number())), |x| AlteredNotes::Flat(x.1)),
        map(tuple((tag("#"), number())), |x| AlteredNotes::Sharp(x.1)),
        map(tuple((tag("add"), number())), |x| AlteredNotes::Add(x.1)),
        map(tag("sus"), |_| AlteredNotes::Sus),
        map(tag("alt"), |_| AlteredNotes::Alt),
    )))
}

fn over<'a>() -> impl FnMut(&'a str) -> IResult<&'a str, Option<Note>> {
    alt((
        map(tuple((tag("/"), note())), |x| Some(x.1)),
        map(tag(""), |_| None),
    ))
}

fn chord<'a>() -> impl FnMut(&'a str) -> IResult<&'a str, Chord> {
    alt((
        map(tag("n"), |_| Chord::NC),
        map(tuple((note(), flavor(), altered_notes(), over())), |x| {
            Chord::Some {
                root: x.0,
                flavor: x.1,
                altered_notes: x.2,
                bass_note: x.3,
            }
        }),
    ))
}

fn chord_token<'a>() -> impl FnMut(&'a str) -> IResult<&'a str, Token> {
    alt((
        map(tag("n"), |_| Token::Chord(Chord::NC, Width::Unknown)),
        map(tuple((note(), flavor(), altered_notes(), over())), |x| {
            Token::Chord(
                Chord::Some {
                    root: x.0,
                    flavor: x.1,
                    altered_notes: x.2,
                    bass_note: x.3,
                },
                Width::Unknown,
            )
        }),
    ))
}

fn time_signature<'a>() -> impl FnMut(&'a str) -> IResult<&'a str, Token> {
    /* The only signatures in jazz1400 are: T24, T34, T44, T54, T64. */
    /* Assume top number can be multiple digits, and the bottom number is a
     * single digit. */
    map(tuple((tag("T"), digit1)), |x| {
        let digits: &str = x.1;
        let (top, bottom) = digits.split_at(digits.len() - 1);
        let top_num = top.parse().unwrap();
        let bottom_num = bottom.parse().unwrap();
        Token::TimeSignature(top_num, bottom_num)
    })
}

fn control<'a>() -> impl FnMut(&'a str) -> IResult<&'a str, Token> {
    alt((
        map(tag("{"), |_| Token::RepeatStart),
        map(tag("}|"), |_| Token::RepeatEnd),
        map(tag("}"), |_| Token::RepeatEnd),
        map(tag(","), |_| Token::Comma),
        map(tag("XyQ"), |_| Token::Blank),
        map(tag("r|"), |_| Token::RepeatTwoMeasures),
        map(tag("x"), |_| Token::RepeatMeasure),
        map(tag("s"), |_| Token::Squeeze),
        map(tag("Q"), |_| Token::Coda),
        map(tag("S"), |_| Token::Segno),
        map(tag("Y"), |_| Token::VerticalSpace),
        map(tag("p"), |_| Token::PauseSlash),
        map(tag("U"), |_| Token::EndingMeasure),
        map(tag("l"), |_| Token::Unsqueeze),
        map(tag("f"), |_| Token::Fermata),
        map(tag(" "), |_| Token::Space),
    ))
}

fn bar_line<'a>() -> impl FnMut(&'a str) -> IResult<&'a str, Token> {
    alt((
        map(tag("||"), |_| Token::Bar),
        map(tag("|"), |_| Token::Bar),
        map(tag("["), |_| Token::DoubleBarStart),
        map(tag("]"), |_| Token::DoubleBarEnd),
        map(tag("Z"), |_| Token::FinalBar),
        map(tag("Kcl"), |_| Token::BarAndRepeat),
        map(tag("LZ|"), |_| Token::Bar),
        map(tag("LZ"), |_| Token::Bar),
    ))
}

fn tokens(input: &str) -> IResult<&str, Vec<Token>> {
    many0(alt((
        chord_token(),
        bar_line(),
        control(),
        comment(),
        alternate(),
        section_marker(),
        numbered_ending(),
        time_signature(),
    )))(input)
}

pub fn tokenize(input: &str) -> Result<Vec<Token>, String> {
    Ok(all_consuming(tokens)(input).unwrap().1)
}
