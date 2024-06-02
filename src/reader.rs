use std::collections::VecDeque;

pub struct JsonReader<T>
where
    T: std::io::Read + std::io::Seek,
{
    reader: std::io::BufReader<T>,
    character_buffer: VecDeque<char>,
}

impl<T> JsonReader<T>
where
    T: std::io::Read + std::io::Seek,
{
    pub fn new(reader: std::io::BufReader<T>) -> Self {
        JsonReader {
            reader,
            character_buffer: VecDeque::with_capacity(4),
        }
    }

    #[must_use]
    pub fn from_bytes(bytes: &[u8]) -> JsonReader<std::io::Cursor<&[u8]>> {
        JsonReader {
            reader: std::io::BufReader::new(std::io::Cursor::new(bytes)),
            character_buffer: VecDeque::with_capacity(4),
        }
    }
}

impl<T> Iterator for JsonReader<T>
where
    T: std::io::Read + std::io::Seek,
{
    type Item = char;

    #[allow(clippy::cast_possible_wrap)]
    fn next(&mut self) -> Option<Self::Item> {
        if !self.character_buffer.is_empty() {
            return self.character_buffer.pop_front();
        }

        let mut utf8_buffer = [0, 0, 0, 0];
        //Read 4 bytes of the self.reader
        let _ = std::io::Read::read(&mut self.reader, &mut utf8_buffer);

        match std::str::from_utf8(&utf8_buffer) {
            Ok(string) => {
                // Collect the valid characters into character_buffer
                self.character_buffer = string.chars().collect();
                // Return the first character from character_buffer
                self.character_buffer.pop_front()
            }
            Err(error) => {
                let valid_bytes = error.valid_up_to();
                //Read the complete character(s)
                let string = std::str::from_utf8(&utf8_buffer[..valid_bytes]).unwrap();
                //Calculate bytes of the incomplete character
                let remaining_bytes = 4 - valid_bytes;
                //Moves the cursor to the 1st byte of the incomplete character to read it in the next iteration
                let _ = self.reader.seek_relative(-(remaining_bytes as i64));

                self.character_buffer = string.chars().collect();
                self.character_buffer.pop_front()
            }
        }
    }
}