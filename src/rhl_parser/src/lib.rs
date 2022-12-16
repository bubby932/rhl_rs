mod stdlib;

pub mod preprocessing {
    use std::{io::{Error, ErrorKind}, collections::HashMap, fs};

    use crate::stdlib;

    /// # Definition
    /// An RHL definiton is an optional string, since it can either be a code snippet or just an empty value.
    /// This is used for the (albeit basic) macro/constant system.
    type Definition = Option<String>;

    /// # RoseHipLang Preprocessor
    /// The RHL Preprocessor operates similar to the one in the C programming language.
    /// It runs over the source, line by line.
    /// Lines beginning with a '#' are designated as a preprocessor directive and parsed, then handled.
    /// In the event a preprocessor directive cannot be parsed, it is assumed to be invalid and an error is returned.
    /// 
    /// # Directives
    /// * `#ifdef <ident>` - Only outputs the code to the paired `#endif` or `#else` directive if IDENT is defined.
    /// * `#ifundef <ident>` - Only outputs the code to the paired `#endif` or `#else` directive if IDENT is ***not*** defined.
    /// * `#endif` - The counterpart to `#ifdef <ident>`
    /// * `#else` - Can be placed inside of an `#ifdef` pair to output the code after it only if IDENT is not defined.
    /// * `#with <$ident / "path">` - Cuts & pastes the code from either the stdlib module $IDENT or the file at PATH.
    /// * `#define <ident>` - Defines the identifier IDENT without a value.
    /// * `#define <ident> <...src>` - Defines the identifier IDENT with the value of all the code after it.
    /// * `#undefine <ident>` - Un-defines an identifier, regardless of whether or not it has a value associated with it.
    pub struct Preprocessor<'a> {
        lines : Vec<&'a str>,
        out : String,
        defs : HashMap<String, Definition>,
        index : usize
    }
    
    impl std::fmt::Display for Preprocessor<'_> {
        fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
            writeln!(f, "Preprocessor Data:")?;
            writeln!(f, "  Input:")?;
            for l in &self.lines {
                writeln!(f, "    {l}")?;
            };
            writeln!(f, "  Output:")?;
            writeln!(f, "{}", self.out)?;
            Ok(())
        }
    }

    impl Preprocessor<'_> {
        pub fn new<'a>(input: &'a str) -> Preprocessor<'a> {
            Preprocessor { 
                lines: input.lines().collect::<Vec<&'a str>>(),
                out: String::new(),
                defs: HashMap::new(),
                index: 0
            }
        }

        pub fn define(&mut self, key : String, value : Definition) {
            self.defs.insert(key, value);
        }

        pub fn run<'a>(&'a mut self) -> Result<&'a str, Error> {
            while self.index < self.lines.len() {
                let mut line = self.lines[self.index].to_owned();

                if !line.starts_with("#") { 
                    let mut iter = self.defs.iter();
                    while let Some(val) = iter.next() {
                        if let Some(v) = val.1 {
                            line = line.replace(val.0, v);
                        }
                    }
                    self.out.push_str(&line);
                    self.index += 1;
                    continue;
                }

                let mut words = line.split_whitespace();

                let first_word = match words.next() {
                    Some(val) => val,
                    None => return Err(
                        Error::new(
                            ErrorKind::InvalidData, 
                            format!("Failed to get first word of preprocessor directive '{line}' at line {}", self.index)
                        )
                    )
                };

                match first_word {
                    "#define" => {
                        let ident = match words.next() {
                            Some(val) => val.to_owned(),
                            None => return Err(Error::new(ErrorKind::InvalidData, format!("Failed to get <IDENT> in preprocessor directive '{line}' at line {}.", self.index)))
                        };

                        let expr = match words.next() {
                            Some(val) => Some(val.to_owned()),
                            None => None
                        };

                        self.defs.insert(ident, expr);
                    },
                    "#undefine" => {
                        let ident = match words.next() {
                            Some(v) => v,
                            None => return Err(Error::new(ErrorKind::InvalidData, format!("Error in preprocessor directive `#undefine <ident>` - failed to get identifier at line {}.", self.index)))
                        };

                        self.defs.remove(ident);
                    },
                    "#ifdef" => {
                        let ident = match words.next() {
                            Some(val) => val,
                            None => return Err(Error::new(ErrorKind::InvalidData, format!("Invalid preprocessor directive `{line}` - expected IDENT, got EOL.")))
                        };

                        let defined = self.defs.contains_key(ident);

                        self.index += 1;

                        if defined {
                            self.read_until_endif_or_else()?;
                        }
                    },
                    "#ifundef" => {
                        let ident = match words.next() {
                            Some(val) => val,
                            None => return Err(Error::new(ErrorKind::InvalidData, format!("Invalid preprocessor directive `{line}` - expected IDENT, got EOL.")))
                        };

                        let defined = self.defs.contains_key(ident);

                        self.index += 1;

                        if !defined {
                            self.read_until_endif_or_else()?;
                        }
                    },
                    "#else" => self.read_until_endif_or_else()?,
                    "#endif" => {
                        return Err(Error::new(ErrorKind::InvalidData, format!("Unexpected #endif directive at line {}.", self.index)));
                    },
                    "#with" => {
                        let second_word = match words.next() {
                            Some(val) => val,
                            None => return Err(Error::new(ErrorKind::InvalidData, format!("Expected file path or library name after #with directive at line {}", self.index)))
                        };

                        let src : String = if second_word.starts_with("$") {
                            match stdlib::BUILTIN_LIBS.get(second_word) {
                                Some(src) => src.to_owned(),
                                None => return Err(Error::new(ErrorKind::InvalidData, format!("No stdlib module with identifier {second_word} at line {}.", self.index)))
                            }
                        } else {
                            // We've already parsed the directive before now, we'll be fine.
                            let path = line.split_once(" ").unwrap();
                            fs::read_to_string(path.1)?
                        };

                        let mut p = Preprocessor::new(&src);
                        self.out.push_str(&p.run()?);
                    },
                    _ => return Err(Error::new(ErrorKind::InvalidData, format!("Invalid preprocessor directive `{first_word}` at line {}.", self.index)))
                }

                self.index += 1;
            }

            Ok(&self.out)
        }

        fn read_until_endif_or_else(&mut self) -> Result<(), Error> {
            let mut height: u16 = 0; 

            while self.index < self.lines.len() {
                if !self.lines[self.index].starts_with("#") {
                    continue;
                }

                let directive : &str = match self.lines[self.index].split_once(" ") {
                    Some(x) => x.0,
                    None => self.lines[self.index]
                };

                match directive {
                    "#ifdef" => height += 1,
                    "#ifundef" => height += 1,
                    "#endif" => {
                        height -= 1;
                        if height <= 0 {
                            self.index += 1;
                            return Ok(());
                        }
                    },
                    "#else" => {
                        if height - 1 <= 0 {
                            self.index += 1;
                            return Ok(());
                        }
                    }
                    _ => continue
                }

                self.index += 1;
            }

            Err(Error::new(ErrorKind::UnexpectedEof, "Expected preprocessor directive `#endif` or `#else`, got EOF."))
        }
    }
}

pub mod tokenizing {

}

pub mod parsing {

}