pub(super) struct FormatService;

const ACTIVE_CHAR: &str = "*";

impl FormatService {
    pub fn active_item_table<F>(descriptors: Vec<String>, active: F)
    where
        F: Fn(usize) -> bool,
    {
        for (i, descriptor) in descriptors.iter().enumerate() {
            let active_char = if active(i) {
                ACTIVE_CHAR.to_owned()
            } else {
                ACTIVE_CHAR.chars().map(|_| " ").collect()
            };
            println!("{} {}", active_char, descriptor);
        }
    }

    pub fn error(msg: &str) {
        eprintln!("{}", msg);
    }
    pub fn success(msg: &str) {
        println!("[SUCCESS] {}", msg)
    }
    pub fn info(msg: &str) {
        println!("[INFO] {}", msg)
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
