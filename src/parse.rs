use std::{fmt, vec};

use crate::{
    tokenize::{self, Token, Width},
    types::{Chord, TimeSignature},
};

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
        writeln!(f, "|")?;
        Ok(())
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum WrittenElement {
    SectionMarker(String),
    TimeSignature(TimeSignature),
    Chord(Chord, Width),
    NumberedEnding(u32),
    RepeatMeasure,
    RepeatTwoMeasures, // This is a special case because it takes 2 measures.
    Coda,
    Segno,
    Comment(String),
    AlternateChord(Chord),
    PauseSlash,
    Fermata,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct WrittenBar {
    repeat_start: bool,
    repeat_end: bool,
    double_start: bool,
    double_end: bool,
    final_bar: bool,
    elements: Vec<WrittenElement>,
}

impl WrittenBar {
    pub fn new() -> Self {
        WrittenBar {
            repeat_start: false,
            repeat_end: false,
            double_start: false,
            double_end: false,
            final_bar: false,
            elements: vec![],
        }
    }

    pub fn repeat() -> Self {
        WrittenBar {
            repeat_start: false,
            repeat_end: false,
            double_start: false,
            double_end: false,
            final_bar: false,
            elements: vec![WrittenElement::RepeatMeasure],
        }
    }

    pub fn is_empty(&self) -> bool {
        self.elements.is_empty() && !self.repeat_start && !self.repeat_end
    }
}

impl Default for WrittenBar {
    fn default() -> Self {
        Self::new()
    }
}

impl fmt::Display for WrittenBar {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "|")?;
        if self.double_start {
            // TODO: If we ended the previous bar on a double bar, we should not write this.
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
                WrittenElement::RepeatMeasure => {
                    write!(f, "            %           ")?;
                    count += 4; // TODO: Time signature
                }
                WrittenElement::RepeatTwoMeasures => {
                    write!(f, "                       %|")?;
                    write!(f, "%                       ")?;
                    count += 4; // TODO: Time signature
                }
                WrittenElement::Coda => {
                    write!(f, "𝄌")?;
                }
                WrittenElement::Segno => {
                    write!(f, "𝄋")?;
                }
                WrittenElement::Comment(s) => {
                    write!(f, " ({})", s)?;
                }
                WrittenElement::AlternateChord(c) => {
                    write!(f, " ({})", c)?;
                }
                WrittenElement::PauseSlash => {
                    write!(f, "{:>6}", "/")?;
                }
                WrittenElement::Fermata => {
                    write!(f, "𝄐")?;
                }
            }
        }
        // TODO: Take time signature into account
        while count < 4 {
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
    let mut written_bar: WrittenBar = Default::default();
    let mut width = Width::Wide;
    for token in tokens.iter() {
        match token {
            Token::RepeatStart => {
                written_bar.repeat_start = true;
            }
            Token::RepeatEnd => {
                written_bar.repeat_end = true;
                written_bars.push(written_bar);
                written_bar = Default::default();
            }
            Token::SectionMarker(s) => {
                written_bar
                    .elements
                    .push(WrittenElement::SectionMarker(s.clone()));
            }
            Token::TimeSignature(top, bottom) => {
                written_bar
                    .elements
                    .push(WrittenElement::TimeSignature(TimeSignature {
                        top: *top,
                        bottom: *bottom,
                    }));
            }
            Token::Chord(c) => {
                written_bar
                    .elements
                    .push(WrittenElement::Chord(c.clone(), width.clone()));
            }
            Token::Comma | Token::Space | Token::Blank | Token::VerticalSpace => {
                // Ignore these tokens
            }
            Token::Bar => {
                if !written_bar.is_empty() {
                    written_bars.push(written_bar);
                }
                written_bar = Default::default();
            }
            Token::Squeeze => {
                width = Width::Narrow;
            }
            Token::Unsqueeze => {
                width = Width::Wide;
            }
            Token::NumberedEnding(n) => {
                written_bar
                    .elements
                    .push(WrittenElement::NumberedEnding(*n));
            }
            Token::DoubleBarStart => {
                written_bar.double_start = true;
            }
            Token::DoubleBarEnd => {
                written_bar.double_end = true;
                written_bars.push(written_bar);
                written_bar = Default::default();
            }
            Token::BarAndRepeat => {
                written_bars.push(written_bar);
                written_bar = WrittenBar::repeat();
            }
            Token::FinalBar => {
                // Final bar, but there may be a coda after this.
                written_bar.final_bar = true;
                written_bars.push(written_bar);
                written_bar = Default::default();
            }
            Token::Coda => {
                written_bar.elements.push(WrittenElement::Coda);
            }
            Token::Segno => {
                written_bar.elements.push(WrittenElement::Segno);
            }
            Token::Comment(s) => {
                written_bar
                    .elements
                    .push(WrittenElement::Comment(s.trim().into()));
            }
            Token::AlternateChord(c) => {
                written_bar
                    .elements
                    .push(WrittenElement::AlternateChord(c.clone()));
            }
            Token::RepeatMeasure => {
                written_bar.elements.push(WrittenElement::RepeatMeasure);
            }
            Token::RepeatTwoMeasures => {
                written_bar.elements.push(WrittenElement::RepeatTwoMeasures);
            }
            Token::PauseSlash => {
                written_bar.elements.push(WrittenElement::PauseSlash);
            }
            Token::Fermata => {
                written_bar.elements.push(WrittenElement::Fermata);
            }
            Token::EndingMeasure => {
                println!("Ending measure found, but not implemented yet.");
            }
        }
    }
    if !written_bar.is_empty() {
        written_bars.push(written_bar);
    }

    Ok(Music {
        written_bars,
        raw: text.to_string(),
    })
}
