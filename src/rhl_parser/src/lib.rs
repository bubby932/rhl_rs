mod stdlib;

pub mod preprocessing {
    use std::{io::{Error, ErrorKind}, collections::HashMap};

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
                            format!("Failed to get first word of preprocessor directive '{line}'")
                        )
                    )
                };

                match first_word {
                    "#define" => {
                        let ident = match words.next() {
                            Some(val) => val.to_owned(),
                            None => return Err(Error::new(ErrorKind::InvalidData, format!("Failed to get <IDENT> in preprocessor directive '{line}'")))
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
                            None => return Err(Error::new(ErrorKind::InvalidData, format!("Error in preprocessor directive `#undefine <ident>` - No value for IDENT")))
                        };

                        self.defs.remove(ident);
                    },
                    "#ifdef" => todo!("Implement #ifdef"),
                    "#ifundef" => todo!("Implement #ifundef"),
                    "#else" => todo!("Read until properly stacked #endif directive in #else"),
                    "#endif" => {
                        // This is intentionally empty. If we encounter a `#endif` directive, it should be ignored.
                        // All #endif directives are handled in their own functions, so there's no point in handling one.
                    },
                    "#with" => todo!("Implement #with directive."),
                    _ => return Err(Error::new(ErrorKind::InvalidData, format!("Invalid preprocessor directive `{first_word}`")))
                }

                self.index += 1;
            }

            Ok(&self.out)
        }
    }
}

pub mod tokenizing {

}

pub mod parsing {

}