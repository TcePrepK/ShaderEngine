use crate::utils::colorized_text::Colorize;
use regex::Regex;
use std::cell::RefCell;
use std::fmt::Display;
use std::fs;
use std::path::Path;
use std::rc::Rc;

const COLORIZED_DECODER_REGEX: &str = r"(\x1b\[(\d{1,2})m(.*?)\x1b\[0m)";
const HTML_START: &str = r###"
<!DOCTYPE html>
<html lang="en">
<head>
    <meta charset="UTF-8">
    <title>Generated HTML Log</title>
    <style>
        body {
            --color-30: #000000; /* black */
            --color-31: #e06c75; /* red */
            --color-32: #98c379; /* green */
            --color-33: #e5c07b; /* yellow */
            --color-34: #4c3df5; /* blue */
            --color-35: #c594ff; /* magenta */
            --color-36: #7cc7ff; /* cyan */
            --color-37: #666666; /* gray */
        }

        body {
            font-family: monospace;
            background: #1e1e2f;
            color: var(--color-36);

            margin: 0;
            padding: 2rem;
        }

        * {
            padding: 0.1rem 0;
            margin: 0;
        }

        #log_title {
            text-align: center;
            margin-bottom: 1rem;
            font-size: 2rem;
            color: var(--color-31);
        }

        #log_title:before {
            content: "< ";
        }

        #log_title:after {
            content: " >";
        }

        details {
            margin-left: 2rem;
        }

        summary {
            cursor: pointer;
        }

        summary:before {
            content: "< ";
            color: var(--color-36);
        }

        summary:after {
            content: " >";
            color: var(--color-36);
        }

        p {
            margin-left: 2rem;
        }
    </style>
</head>
<body>"###;

trait HTMLElement {
    fn to_html(&self) -> String;
}

pub struct Summary {
    pub text: String,
}

impl HTMLElement for Summary {
    fn to_html(&self) -> String {
        format!("<summary>{}</summary>", format_text(&self.text))
    }
}

pub struct Paragraph {
    text: String,
}

impl HTMLElement for Paragraph {
    fn to_html(&self) -> String {
        format!("<p>{}</p>", format_text(&self.text))
    }
}

pub struct Details {
    open: bool,
    pub summary: Summary,
    elements: Vec<Rc<RefCell<dyn HTMLElement>>>,
}

impl HTMLElement for Details {
    fn to_html(&self) -> String {
        let open_attr = if self.open { " open" } else { "" };
        let mut inner_html = self.summary.to_html();
        inner_html.push_str(
            &self
                .elements
                .iter()
                .map(|el| el.borrow().to_html())
                .collect::<Vec<_>>()
                .join("\n"),
        );
        format!("<details{open_attr}>\n{inner_html}\n</details>")
    }
}

fn format_text(text: &str) -> String {
    let decoder_regex = Regex::new(COLORIZED_DECODER_REGEX).unwrap();

    decoder_regex
        .replace_all(text, |cap: &regex::Captures| {
            let color = cap.get(2).unwrap().as_str();
            let text = cap.get(3).unwrap().as_str();
            format!(
                "<span style=\"color: var(--color-{})\">{}</span>",
                color, text
            )
        })
        .into_owned()
}

pub struct HTMLLogger {
    name: String,
    elements: Vec<Rc<RefCell<dyn HTMLElement>>>,
    stack: Vec<Rc<RefCell<Details>>>,
}

impl HTMLLogger {
    pub fn new<T: Display>(name: T) -> Self {
        Self {
            name: name.to_string(),
            elements: vec![],
            stack: vec![],
        }
    }
}

impl HTMLLogger {
    pub fn log<T: Display>(&mut self, text: T) {
        if !self.stack.is_empty() {
            self.current_scope()
                .borrow_mut()
                .elements
                .push(Rc::new(RefCell::new(Paragraph {
                    text: text.to_string(),
                })));
        } else {
            self.elements.push(Rc::new(RefCell::new(Paragraph {
                text: text.to_string(),
            })));
        }
    }

    pub fn info<T: Display>(&mut self, text: T) {
        let info_text = format!("{} {}", "-".cyan(), text);
        if !self.stack.is_empty() {
            self.current_scope()
                .borrow_mut()
                .elements
                .push(Rc::new(RefCell::new(Paragraph { text: info_text })));
        } else {
            self.elements
                .push(Rc::new(RefCell::new(Paragraph { text: info_text })));
        }
    }

    pub fn error<T: Display>(&mut self, text: T) {
        self.log(text);
        self.stack.clear();
        self.to_html()
    }

    pub fn open_scope<T: Display>(&mut self, text: T) -> Rc<RefCell<Details>> {
        let new_scope = Rc::new(RefCell::new(Details {
            open: false,
            summary: Summary {
                text: text.to_string(),
            },
            elements: vec![],
        }));

        if !self.stack.is_empty() {
            self.current_scope()
                .borrow_mut()
                .elements
                .push(new_scope.clone());
        } else {
            self.elements.push(new_scope.clone());
        }
        self.stack.push(new_scope.clone());
        new_scope
    }

    pub fn close_scope(&mut self) {
        if !self.stack.is_empty() {
            self.stack.pop();
        }
    }

    pub fn panic(&mut self) {
        self.stack.clear();
        self.to_html()
    }

    pub fn to_html(&self) {
        // Generate the HTML content
        let mut html_content = self.generate_header();
        for element in self.elements.iter() {
            html_content += &element.borrow().to_html();
        }
        html_content = HTML_START.to_string() + &html_content + "</body></html>";

        // Generate the logs directory if it doesn't exist
        let logs_dir = Path::new("logs");
        if !logs_dir.exists() {
            fs::create_dir("logs").unwrap();
        }

        // Write the HTML content to the file
        fs::write(format!("logs/{}.html", self.name), html_content).unwrap();
    }
}

impl HTMLLogger {
    fn generate_header(&self) -> String {
        format!("<h1 id=\"log_title\">{}</h1>", self.name)
    }

    fn current_scope(&mut self) -> Rc<RefCell<Details>> {
        self.stack.last_mut().unwrap().clone()
    }
}
