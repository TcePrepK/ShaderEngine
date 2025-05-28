use crate::utils::colorized_text::Colorize;
use std::fmt::Display;

#[macro_export]
macro_rules! quote {
    ($string:expr) => {
        format!("\"{}\"", $string)
    };
}

#[macro_export]
macro_rules! error {
    ($($arg:tt)*) => {
        {
            use std::io::Write;
            let mut buffer = std::io::stderr();
            writeln!(buffer, "\x1b[91merror:\x1b[0m {}", format_args!($($arg)*)).unwrap();
        }
    };
}

pub struct NestedConsoleLogger {
    indent: usize,
    header_fixes: (String, String),
    info_prefix: String,
}

impl Default for NestedConsoleLogger {
    fn default() -> Self {
        NestedConsoleLogger {
            indent: 0,
            header_fixes: ("<".blue().to_string(), ">".blue().to_string()),
            info_prefix: "-".blue().to_string(),
        }
    }
}

#[allow(dead_code)]
impl NestedConsoleLogger {
    pub fn log<T: Display>(&mut self, message: T) {
        println!("{}{}", "  ".repeat(self.indent), message);
    }

    pub fn info<T: Display>(&mut self, message: T) {
        println!(
            "{}{} {}",
            "  ".repeat(self.indent),
            self.info_prefix.blue(),
            message
        );
    }

    pub fn error<T: Display>(&mut self, message: T) {
        println!(
            "{}{} {} {}",
            "  ".repeat(self.indent),
            "! Error !".red(),
            message,
            "! Error !".red(),
        );
    }

    pub fn open_scope<T: Display>(&mut self, message: T) {
        if message.to_string().len() > 0 {
            println!(
                "{}{} {} {}",
                "  ".repeat(self.indent),
                self.header_fixes.0,
                message,
                self.header_fixes.1
            );
        }
        self.indent += 1;
    }

    pub fn close_scope<T: Display>(&mut self, message: T) {
        self.indent -= 1;
        if message.to_string().len() > 0 {
            println!(
                "{}{} {} {}",
                "  ".repeat(self.indent),
                self.header_fixes.0,
                message,
                self.header_fixes.1
            );
        }
    }

    pub fn panic<T: Display>(&mut self, message: T) {
        self.indent = 0;
        println!(
            "{} {} {}",
            self.header_fixes.0, message, self.header_fixes.1
        );
    }
}
