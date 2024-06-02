use crate::reader::JsonReader;

#[derive(Debug, Copy, Clone, PartialEq)]
pub enum Number {
    I64(i64),
    F64(f64),
}

#[derive(Debug, Clone, PartialEq)]
pub enum Token {
    CurlyOpen,
    CurlyClose,
    Quotes,
    Colon,
    String(String),
    Number(Number),
    ArrayOpen,
    ArrayClose,
    Comma,
    Boolean(bool),
    Null,
}

impl From<Number> for crate::value::Number {
    fn from(number: Number) -> Self {
        match number {
            Number::I64(val) => crate::value::Number::I64(val),
            Number::F64(val) => crate::value::Number::F64(val),
        }
    }
}

pub struct JsonTokenizer<T>
    where
        T: std::io::Read + std::io::Seek,
{
    tokens: Vec<Token>,
    iterator: std::iter::Peekable<JsonReader<T>>,
}

impl<T> JsonTokenizer<T>
where
    T: std::io::Read + std::io::Seek,
{
    pub fn new(json_reader: JsonReader<T>) -> JsonTokenizer<T> {
        JsonTokenizer {
            iterator: json_reader.peekable(),
            tokens: vec![],
        }
    }
}

impl<T> JsonTokenizer<T> where
    T: std::io::Read + std::io::Seek, {
    pub fn tokenize_json(&mut self) -> Result<&[Token], ()> {
        while let Some(character) = self.iterator.peek() {
            match *character {
                '-' | '0'..='9' => {
                    let number = self.parse_number()?;
                    self.tokens.push(Token::Number(number));
                },
                '"' => {
                    self.tokens.push(Token::Quotes);

                    let _ = self.iterator.next();

                    // Delegate parsing string value to a separate function.
                    let string = self.parse_string();

                    self.tokens.push(Token::String(string));
                    self.tokens.push(Token::Quotes);
                },
                'n' => {
                    let _ = self.iterator.next();
                    assert_eq!(Some('u'), self.iterator.next());
                    assert_eq!(Some('l'), self.iterator.next());
                    assert_eq!(Some('l'), self.iterator.next());
                    self.tokens.push(Token::Null);
                },
                't' => {
                    let _ = self.iterator.next();
                    assert_eq!(Some('r'), self.iterator.next());
                    assert_eq!(Some('u'), self.iterator.next());
                    assert_eq!(Some('e'), self.iterator.next());
                    self.tokens.push(Token::Boolean(true));
                }
                'f' => {
                    let _ = self.iterator.next();
                    assert_eq!(Some('a'), self.iterator.next());
                    assert_eq!(Some('l'), self.iterator.next());
                    assert_eq!(Some('s'), self.iterator.next());
                    assert_eq!(Some('e'), self.iterator.next());
                    self.tokens.push(Token::Boolean(false));
                },
                '{' => {
                    self.tokens.push(Token::CurlyOpen);
                    let _ = self.iterator.next();
                }
                '}' => {
                    self.tokens.push(Token::CurlyClose);
                    let _ = self.iterator.next();
                }
                '[' => {
                    self.tokens.push(Token::ArrayOpen);
                    let _ = self.iterator.next();
                }
                ']' => {
                    self.tokens.push(Token::ArrayClose);
                    let _ = self.iterator.next();
                }
                ',' => {
                    self.tokens.push(Token::Comma);
                    let _ = self.iterator.next();
                }
                ':' => {
                    self.tokens.push(Token::Colon);
                    let _ = self.iterator.next();
                },
                '\0' => break,
                character => {
                    if character.is_ascii_whitespace() {
                        let _ = self.iterator.next();
                        continue;
                    }

                    panic!("Unexpected character: \" {character} \"")
                }
            }
        }

        Ok(&self.tokens)
    }

    fn parse_string(&mut self) -> String {
        let mut string_characters = Vec::<char>::new();

        // Take each character by reference so they are not removed from the iterator.
        for character in self.iterator.by_ref() {
            if character == '"' {
                break;
            }
            string_characters.push(character);
        }

        String::from_iter(string_characters)
    }

    fn parse_number(&mut self) -> Result<Number, ()> {
        let mut number_characters = Vec::<char>::new();

        let mut is_decimal = false;

        let mut epsilon_characters = Vec::<char>::new();

        let mut is_epsilon_characters = false;

        while let Some(character) = self.iterator.peek() {
            match character {
                '-' => {
                    if is_epsilon_characters {
                        epsilon_characters.push('-');
                    } else {
                        number_characters.push('-');
                    }
                    let _ = self.iterator.next();
                }
                // Match any digit between 0 and 9, and store it into the `digit`
                digit @ '0'..='9' => {
                    if is_epsilon_characters {
                        epsilon_characters.push(*digit);
                    } else {
                        number_characters.push(*digit);
                    }
                    let _ = self.iterator.next();
                }
                '.' => {
                    number_characters.push('.');

                    is_decimal = true;

                    let _ = self.iterator.next();
                }
                // Match any of the characters that can signify end of the number
                '}' | ',' | ']' | ':' => {
                    break;
                }
                'e' | 'E' => {
                    if is_epsilon_characters {
                        panic!("[!] Double epsilon characters encountered");
                    }

                    is_epsilon_characters = true;

                    let _ = self.iterator.next();
                }
                other => {
                    if !other.is_ascii_whitespace() {
                        panic!("[!] Unexpected character while parsing number: {character}")
                    } else {
                        self.iterator.next();
                    }
                },
            }
        }

        if is_epsilon_characters {
            let base: f64 = String::from_iter(number_characters).parse().unwrap();
            let exponential: f64 = String::from_iter(epsilon_characters).parse().unwrap();
            Ok(Number::F64(base * 10_f64.powf(exponential)))
        } else if is_decimal {
            Ok(Number::F64(
                String::from_iter(number_characters).parse::<f64>().unwrap(),
            ))
        } else {
            Ok(Number::I64(
                String::from_iter(number_characters).parse::<i64>().unwrap(),
            ))
        }
    }

}