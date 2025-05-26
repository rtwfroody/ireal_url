use std::fmt;

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
