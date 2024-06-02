mod reader;
mod value;
mod token;

use std::collections::HashMap;

pub struct JsonParser;

impl JsonParser {
    pub fn parse_from_bytes<'a>(input: &str) -> Result<crate::value::Value, ()> {
        let json_reader = crate::reader::JsonReader::<std::io::Cursor<&'static [u8]>>::from_bytes(input.as_bytes());
        let mut json_tokenizer = crate::token::JsonTokenizer::new(json_reader);
        let tokens = json_tokenizer.tokenize_json()?;
    
        Ok(Self::tokens_to_value(tokens))
    }

    pub fn parse(reader: std::fs::File) -> Result<crate::value::Value, ()> {
        let reader = std::io::BufReader::new(reader);
        let json_reader = crate::reader::JsonReader::new(reader);
        let mut json_tokenizer = crate::token::JsonTokenizer::new(json_reader);
        let tokens = json_tokenizer.tokenize_json()?;

        Ok(Self::tokens_to_value(tokens))
    }

    fn tokens_to_value(tokens: &[crate::token::Token]) -> crate::value::Value {
        let mut iterator = tokens.iter().peekable();

        let mut value = crate::value::Value::Null;

        while let Some(token) = iterator.next() {
            match token {
                crate::token::Token::CurlyOpen => {
                    value = crate::value::Value::Object(Self::process_object(&mut iterator));
                }
                crate::token::Token::String(string) => {
                    value = crate::value::Value::String(string.clone());
                }
                crate::token::Token::Number(number) => {
                    value = crate::value::Value::Number(<crate::token::Number as Into<crate::value::Number>>::into(*number))
                }
                crate::token::Token::ArrayOpen => {
                    value = crate::value::Value::Array(Self::process_array(&mut iterator));
                }
                crate::token::Token::Boolean(boolean) => value = crate::value::Value::Boolean(*boolean),
                crate::token::Token::Null => value = crate::value::Value::Null,
                crate::token::Token::Comma
                | crate::token::Token::CurlyClose
                | crate::token::Token::Quotes
                | crate::token::Token::Colon
                | crate::token::Token::ArrayClose => {}
            }
        }

        value
    }
    
    fn process_array(iterator: &mut std::iter::Peekable<std::slice::Iter<crate::token::Token>>) -> Vec<crate::value::Value> {
        let mut internal_value = Vec::<crate::value::Value>::new();

        while let Some(token) = iterator.next() {
            match token {
                crate::token::Token::CurlyOpen => {
                    internal_value.push(crate::value::Value::Object(Self::process_object(iterator)));
                }
                crate::token::Token::String(string) => internal_value.push(crate::value::Value::String(string.clone())),
                crate::token::Token::Number(number) => internal_value.push(crate::value::Value::Number(<crate::token::Number as Into<crate::value::Number>>::into(*number))),
                crate::token::Token::ArrayOpen => {
                    internal_value.push(crate::value::Value::Array(Self::process_array(iterator)));
                }
                crate::token::Token::ArrayClose => {
                    break;
                }
                crate::token::Token::Boolean(boolean) => internal_value.push(crate::value::Value::Boolean(*boolean)),
                crate::token::Token::Null => internal_value.push(crate::value::Value::Null),
                // Ignore delimiters
                crate::token::Token::Comma | crate::token::Token::CurlyClose | crate::token::Token::Quotes | crate::token::Token::Colon => {}
            }
        }

        internal_value
    }
    fn process_object(iterator: &mut std::iter::Peekable<std::slice::Iter<crate::token::Token>>) -> HashMap<String, crate::value::Value> {
        let mut is_key = true;

        let mut current_key: Option<&str> = None;

        let mut value = HashMap::<String, crate::value::Value>::new();

        while let Some(token) = iterator.next() {
            match token {
                crate::token::Token::CurlyOpen => {
                    if let Some(current_key) = current_key {
                        value.insert(
                            current_key.to_string(),
                            crate::value::Value::Object(Self::process_object(iterator)),
                        );
                    }
                }
                crate::token::Token::CurlyClose => {
                    break;
                }
                crate::token::Token::Quotes | crate::token::Token::ArrayClose => {}
                crate::token::Token::Colon => {
                    is_key = false;
                }
                crate::token::Token::String(string) => {
                    if is_key {
                        //Key
                        current_key = Some(string);
                    } else if let Some(key) = current_key {
                        //Value
                        value.insert(key.to_string(), crate::value::Value::String(string.clone()));
                        current_key = None;
                    }
                }
                crate::token::Token::Number(number) => {
                    if let Some(key) = current_key {
                        value.insert(key.to_string(), crate::value::Value::Number(<crate::token::Number as Into<crate::value::Number>>::into(*number)));
                        current_key = None;
                    }
                }
                crate::token::Token::ArrayOpen => {
                    if let Some(key) = current_key {
                        value.insert(key.to_string(), crate::value::Value::Array(Self::process_array(iterator)));
                        current_key = None;
                    }
                }
                //New key
                crate::token::Token::Comma => is_key = true,
                crate::token::Token::Boolean(boolean) => {
                    if let Some(key) = current_key {
                        value.insert(key.to_string(), crate::value::Value::Boolean(*boolean));
                        current_key = None;
                    }
                }
                crate::token::Token::Null => {
                    if let Some(key) = current_key {
                        value.insert(key.to_string(), crate::value::Value::Null);
                        current_key = None;
                    }
                }
            }
        }

        value
    }
}

//Run test with --nocapture
#[cfg(test)]
mod tests {
    #[test]
    fn from_file() {
        let file = std::fs::File::open("sample.json").unwrap();
        dbg!(crate::JsonParser::parse(file).unwrap());
    }
    #[test]
    fn from_bytes() {
        let input_json_string = r#"{"key1":"value1","key2":"value2"}"#;
        dbg!(crate::JsonParser::parse_from_bytes(input_json_string).unwrap());
    }
}