use std::{fmt::Display, rc::Rc};

use colored::Colorize;

pub(super) struct FormatService;

#[macro_export]
macro_rules! table {
    ($($header:expr),+ ; $($columns:expr),+ ; $($alignment:expr),+) => {
        {
            use $crate::service::format::FormatType;
            use std::cmp::max;

            // ensure same length for input
            let header = [$($header),+];
            let alignment = [$($alignment),+];
            let mut columns = [$($columns),+];
            assert!(header.len() == alignment.len() && header.len() == columns.len(), "Header, columns and alignment must have the same length");

            // Get len of all columns
            let max_len = columns.iter().map(|col| col.len()).max().unwrap_or(0);

            // Resize all columns to the same length
            columns.iter_mut().for_each(|col| col.resize(max_len, "".into()));

            //  Calculate max widths for each column
            let max_len_columns = columns.iter().enumerate().map(|(idx, col)|
                {
                    let len = col.iter().map(|it| it.len()).max().unwrap_or(0);
                    max(header[idx].len(), len)
                }
            ).collect::<Vec<_>>();

            let header_padding = (0,0);
            let padding = (0,0);

            // Format Header to align with max columns width
            let mut header_formatted = Vec::new();
            for i in 0..header.len() {
                 let header = FormatType::align(&header[i], *&alignment[i], max_len_columns[i], header_padding);
                 header_formatted.push(header);
            }

            let header = header_formatted.join(" | ");
            let mut acc = FormatType::RawLine(header);

            for i in 0..max_len {
                let mut row = Vec::new();
                for j in 0..columns.len() {
                    let column = FormatType::align(&columns[j][i], alignment[j], max_len_columns[j], padding);
                    row.push(column);
                }
                let row = row.join("   ");
                acc = acc.chain(FormatType::RawLine(row));
            }
            acc
        }
    };
}

impl FormatService {
    pub fn run<T: FormatTypeable>(msg: T) {
        println!("{}", msg.format());
    }

    /// returns either a vec of [DialogOutput] which contain the user input or None if the dialog was canceled
    pub fn dialog(dialog: Vec<DialogEntry>) -> Option<Vec<DialogOutput>> {
        let mut output = Vec::new();
        for entry in dialog {
            match entry {
                DialogEntry::Message(msg) => {
                    println!("{}", msg);
                    continue;
                }
                DialogEntry::YesNoInput(msg) => {
                    let out = loop {
                        println!("{} [y/n] (q to cancel)", msg);
                        let mut input = String::new();
                        if std::io::stdin().read_line(&mut input).is_err() {
                            println!("Failed to read input");
                            continue;
                        }

                        match input.trim().to_lowercase().as_str() {
                            "y" | "yes" => break DialogOutput::YesNo(true),
                            "n" | "no" => break DialogOutput::YesNo(false),
                            "q" => return None,
                            _ => {
                                println!("Invalid input, please enter 'y' or 'n'");
                                continue;
                            }
                        }
                    };
                    output.push(out);
                }
                DialogEntry::NumberInput(msg) => {
                    let out = loop {
                        println!("{} (q to cancel)", msg);
                        let mut input = String::new();
                        if std::io::stdin().read_line(&mut input).is_err() {
                            println!("Failed to read input");
                            continue;
                        }

                        let input = input.trim();
                        if input.eq_ignore_ascii_case("q") {
                            return None;
                        }

                        match input.parse::<usize>() {
                            Ok(number) => break DialogOutput::Number(number),
                            Err(_) => {
                                println!("Invalid number, please try again");
                                continue;
                            }
                        }
                    };
                    output.push(out);
                }
            }
        }
        Some(output)
    }
}

pub(crate) enum DialogEntry {
    Message(String),
    YesNoInput(String),
    NumberInput(String),
}

pub(crate) enum DialogOutput {
    Text(String),
    YesNo(bool),
    Number(usize),
}

#[derive(Debug, Clone)]
pub(crate) enum FormatType {
    Bold(String),
    RawLine(String),
    Block(Rc<FormatType>, Rc<FormatType>),
    Chain(Vec<FormatType>),
    Success(String),
    Error(String),
    Info(String),
}

#[derive(Debug, Clone, Copy)]
pub enum FormatAlignment {
    Left,
    Right,
    Center,
}

impl FormatType {
    pub fn chain(mut self, other: FormatType) -> FormatType {
        match self {
            FormatType::Chain(ref mut chain) => chain.push(other),
            _ => {
                let chain = vec![self, other];
                self = FormatType::Chain(chain);
            }
        }
        self
    }

    pub fn block(self, body: FormatType) -> FormatType {
        FormatType::Block(Rc::new(self), Rc::new(body))
    }

    pub fn align(
        str: &str,
        alignment: FormatAlignment,
        max_len: usize,
        padding: (usize, usize),
    ) -> String {
        let (left, right) = match alignment {
            FormatAlignment::Left => (0, max_len - str.len()),
            FormatAlignment::Right => (max_len - str.len(), 0),
            FormatAlignment::Center => {
                let padding = max_len - str.len();
                let left = padding.div_ceil(2);
                let right = padding.div_floor(2);
                (left, right)
            }
        };
        let padding_left = " ".repeat(padding.0 + left);
        let padding_right = " ".repeat(padding.1 + right);
        format!("{}{}{}", padding_left, str, padding_right)
    }
}

impl std::fmt::Display for FormatType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            FormatType::Bold(msg) => write!(f, "{}", msg.bold()),
            FormatType::RawLine(msg) => write!(f, "{}\n", msg),
            FormatType::Success(msg) => write!(f, "{} {}", "[SUCCESS]".green(), msg),
            FormatType::Error(msg) => write!(f, "{} {}", "[ERROR]".red(), msg),
            FormatType::Info(msg) => write!(f, "{} {}", "[INFO]".yellow(), msg),
            FormatType::Block(header, content) => {
                write!(f, "{}", FormatType::Bold(header.as_ref().to_string()))?;
                write!(f, "{}", Offset(2, content.as_ref().clone()))
            }
            Self::Chain(chain) => {
                for item in chain {
                    write!(f, "{}", item)?;
                }
                Ok(())
            }
        }
    }
}

struct Offset(usize, FormatType);

impl Display for Offset {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let offset = " ".repeat(self.0);
        match &self.1 {
            FormatType::Chain(content) => {
                for item in content {
                    write!(f, "{}", Offset(2, item.clone()))?
                }
                Ok(())
            }
            _ => write!(f, "{}{}", offset, self.1),
        }
    }
}

pub(crate) trait FormatTypeable {
    fn format(self) -> FormatType;
}

impl FormatTypeable for FormatType {
    fn format(self) -> FormatType {
        self
    }
}

impl FormatTypeable for String {
    fn format(self) -> FormatType {
        FormatType::Success(self)
    }
}

impl FormatTypeable for &str {
    fn format(self) -> FormatType {
        FormatType::Success(self.to_string())
    }
}

impl FormatTypeable for anyhow::Error {
    fn format(self) -> FormatType {
        FormatType::Error(self.to_string())
    }
}

pub trait IntoFormatType {
    fn info(self) -> FormatType;
    fn success(self) -> FormatType;
    fn error(self) -> FormatType;
    fn line(self) -> FormatType;
}

impl IntoFormatType for String {
    fn info(self) -> FormatType {
        FormatType::Info(self)
    }

    fn success(self) -> FormatType {
        FormatType::Success(self)
    }

    fn error(self) -> FormatType {
        FormatType::Error(self)
    }

    fn line(self) -> FormatType {
        FormatType::RawLine(self)
    }
}

impl IntoFormatType for &str {
    fn info(self) -> FormatType {
        FormatType::Info(self.to_string())
    }

    fn success(self) -> FormatType {
        FormatType::Success(self.to_string())
    }

    fn error(self) -> FormatType {
        FormatType::Error(self.to_string())
    }

    fn line(self) -> FormatType {
        FormatType::RawLine(self.to_string())
    }
}
