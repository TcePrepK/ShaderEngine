use std::fmt::{Display, Formatter, Result};
use std::ops::Add;

#[derive(Clone, Debug)]
pub struct ColoredText {
    text: String,
}

#[allow(dead_code)]
impl ColoredText {
    fn new(text: String) -> ColoredText {
        ColoredText { text }
    }

    pub fn as_str(&self) -> &str {
        &self.text
    }

    pub fn println(&self) {
        println!("{}", self);
    }
}

impl Into<String> for ColoredText {
    fn into(self) -> String {
        self.text
    }
}

impl Display for ColoredText {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result {
        write!(f, "{}", self.text)
    }
}

impl Add for ColoredText {
    type Output = ColoredText;

    fn add(self, rhs: Self) -> Self::Output {
        ColoredText::new(self.text + &rhs.text)
    }
}

#[allow(dead_code)]
pub trait Colorize {
    fn black(&self) -> ColoredText;
    fn red(&self) -> ColoredText;
    fn green(&self) -> ColoredText;
    fn yellow(&self) -> ColoredText;
    fn blue(&self) -> ColoredText;
    fn magenta(&self) -> ColoredText;
    fn cyan(&self) -> ColoredText;
    fn white(&self) -> ColoredText;
}

macro_rules! single_colorize {
    ($name:ident, $code:expr) => {
        fn $name(&self) -> ColoredText {
            ColoredText::new(format!("\x1b[{}m{}\x1b[0m", $code, self))
        }
    };
}

macro_rules! impl_colorize {
    ($ty:ty) => {
        impl Colorize for $ty {
            single_colorize!(black, 30);
            single_colorize!(red, 31);
            single_colorize!(green, 32);
            single_colorize!(yellow, 33);
            single_colorize!(blue, 34);
            single_colorize!(magenta, 35);
            single_colorize!(cyan, 36);
            single_colorize!(white, 37);
        }
    };
}

impl_colorize!(String);
impl_colorize!(&str);
impl_colorize!(ColoredText);
