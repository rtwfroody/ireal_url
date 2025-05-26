// Reference: https://www.irealpro.com/ireal-pro-custom-chord-chart-protocol

use std::fmt;

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
pub struct Music {
    pub repeat_start: Option<usize>,
    pub raw: String,
    pub bars: Vec<Bar>,
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
    Chord(Chord),
    AlternateChord(Chord),
}

impl fmt::Display for BarElement {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            BarElement::SectionMarker(s) => s.fmt(f),
            BarElement::TimeSignature(ts) => ts.fmt(f),
            BarElement::Chord(c) => c.fmt(f),
            BarElement::AlternateChord(c) => format!("({})", c).fmt(f),
        }
    }
}

#[derive(Clone, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub enum Chord {
    NC,
    Some {
        root: Note,
        flavor: Flavor,
        altered_notes: Vec<AlteredNotes>,
        bass_note: Option<Note>,
    },
}

impl Chord {
    pub fn basic(root: Note, flavor: Flavor) -> Self {
        Chord::Some {
            root,
            flavor,
            altered_notes: vec![],
            bass_note: None,
        }
    }
}

impl fmt::Display for Chord {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Chord::NC => write!(f, "N.C."),
            Chord::Some {
                root,
                flavor,
                altered_notes,
                bass_note,
            } => {
                write!(
                    f,
                    "{}{}{}{}",
                    root,
                    flavor,
                    altered_notes
                        .iter()
                        .map(|x| x.to_string())
                        .collect::<String>(),
                    match &bass_note {
                        Some(note) => format!("/{}", note),
                        None => "".to_string(),
                    }
                )
            }
        }
    }
}

impl fmt::Debug for Chord {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        format!("{}", self).fmt(f)
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub enum Token {
    AlternateChord(Chord), // These show up above the regular music.
    Bar,
    Blank,
    Chord(Chord),
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

    // Seems to be used when there is one chord per note.
    Squeeze,

    // TODO: What is this for?
    SmallL,

    // I think this is like in BiaB. Comma is to put two chords in the first
    // half of a measure, and space is just to have one chord in each half.
    Comma,
    Space,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub enum Note {
    AFlat,
    A,
    ASharp,
    BFlat,
    B,
    CFlat,
    C,
    CSharp,
    DFlat,
    D,
    DSharp,
    EFlat,
    E,
    F,
    FSharp,
    GFlat,
    G,
    GSharp,
    // Indicates no note? Used as in "W/C" which visually just shows "/C"
    W,
}

impl fmt::Display for Note {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Note::AFlat => "Ab",
            Note::A => "A",
            Note::ASharp => "A#",
            Note::BFlat => "Bb",
            Note::B => "B",
            Note::CFlat => "Cb",
            Note::C => "C",
            Note::CSharp => "C#",
            Note::DFlat => "Db",
            Note::D => "D",
            Note::DSharp => "D#",
            Note::EFlat => "Eb",
            Note::E => "E",
            Note::F => "F",
            Note::FSharp => "F#",
            Note::GFlat => "Gb",
            Note::G => "G",
            Note::GSharp => "G#",
            Note::W => "W",
        }
        .fmt(f)
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub enum Number {
    Two,
    Three,
    Five,
    Six,
    Seven,
    Nine,
    Eleven,
    Thirteen,
}

impl fmt::Display for Number {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Number::Two => "2",
            Number::Three => "3",
            Number::Five => "5",
            Number::Six => "6",
            Number::Seven => "7",
            Number::Nine => "9",
            Number::Eleven => "11",
            Number::Thirteen => "13",
        }
        .fmt(f)
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub enum Flavor {
    Augmented(Option<Number>),
    Diminished(Option<Number>),
    DiminishedMajor(Option<Number>),
    HalfDiminished(Option<Number>),
    Minor(Option<Number>),
    MinorMajor(Option<Number>),
    Dominant(Option<Number>),
    Major(Option<Number>),
    SixthNinth,
    MinorSixthNinth,
}

impl fmt::Display for Flavor {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Flavor::Augmented(n) => match n {
                Some(n) => format!("+{}", n),
                None => "+".to_string(),
            },
            Flavor::Diminished(n) => match n {
                Some(n) => format!("o{}", n),
                None => "o".to_string(),
            },
            Flavor::DiminishedMajor(n) => match n {
                Some(n) => format!("o^{}", n),
                None => "o^".to_string(),
            },
            Flavor::HalfDiminished(n) => match n {
                Some(n) => format!("h{}", n),
                None => "h".to_string(),
            },
            Flavor::Minor(n) => match n {
                Some(n) => format!("-{}", n),
                None => "-".to_string(),
            },
            Flavor::MinorMajor(n) => match n {
                Some(n) => format!("-^{}", n),
                None => "-^".to_string(),
            },
            Flavor::Dominant(n) => match n {
                Some(n) => format!("{}", n),
                None => "".to_string(),
            },
            Flavor::Major(n) => match n {
                Some(n) => format!("^{}", n),
                None => "^".to_string(),
            },
            Flavor::SixthNinth => "69".to_string(),
            Flavor::MinorSixthNinth => "m69".to_string(),
        }
        .fmt(f)
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub enum AlteredNotes {
    Flat(Number),
    Sharp(Number),
    Add(Number),
    Sus,
    Alt,
}

impl fmt::Display for AlteredNotes {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            AlteredNotes::Flat(n) => format!("b{}", n),
            AlteredNotes::Sharp(n) => format!("#{}", n),
            AlteredNotes::Add(n) => format!("add{}", n),
            AlteredNotes::Sus => "sus".to_string(),
            AlteredNotes::Alt => "alt".to_string(),
        }
        .fmt(f)
    }
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
        map(tag("n"), |_| Token::Chord(Chord::NC)),
        map(tuple((note(), flavor(), altered_notes(), over())), |x| {
            Token::Chord(Chord::Some {
                root: x.0,
                flavor: x.1,
                altered_notes: x.2,
                bass_note: x.3,
            })
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
        map(tag("l"), |_| Token::SmallL),
        map(tag("f"), |_| Token::Fermata),
        map(tag(" "), |_| Token::Space),
    ))
}

fn bar<'a>() -> impl FnMut(&'a str) -> IResult<&'a str, Token> {
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

fn parse_tokens(input: &str) -> IResult<&str, Vec<Token>> {
    many0(alt((
        chord_token(),
        bar(),
        control(),
        comment(),
        alternate(),
        section_marker(),
        numbered_ending(),
        time_signature(),
    )))(input)
}

pub fn parse_music(text: &str) -> Result<Music, String> {
    let mut bars = vec![];
    let mut time_signature = TimeSignature { top: 4, bottom: 4 }; // Default time signature
    let mut bar = None;
    let mut count = 0;
    for token in all_consuming(parse_tokens)(text).unwrap().1 {
        match token {
            Token::Bar | Token::FinalBar | Token::DoubleBarEnd | Token::RepeatEnd => {
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
                count += 1;
            }
            Token::AlternateChord(c) => {
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
            Token::Space => {
                if bar.is_none() {
                    bar = Some(Bar::new(time_signature.top));
                    count = 0;
                }
                count += 1;
            }
            ignore => println!("Ignoring token: {:?}", ignore),
        }
    }
    Ok(Music {
        repeat_start: None,
        bars,
        raw: text.to_string(),
    })
}
