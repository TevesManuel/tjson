I wrote this little library on a free afternoon to improve my Rust skills. And I upload it for 2 things, CV and a small explanation for the average citizen of how the hell this works. Okay, that's not easy but it's not very difficult either. You just need to know the basics of the Rust language and I'll explain the rest.

What is UTF-8?

    It is a variable length character encoding scheme.
between 1 byte and 4 bytes, which has the most characters in the world, which is commonly used on the web and widely used in most computer systems, therefore it is important.

What is Peekable trait?

In rust the Peekable trait is an iterator that allows us to obtain a reference to the next element without consuming it with the peek() function, and advance to the next one to consume it with the next() function.

Ok, once I have clarified the “essential” that I consider does not enter into the basics of rust.
I divided the work into 4 structures, one for reading, one for values, one for tokenization and one for parsing.
In the reading one where I basically make an iterator with support for UTF-8 due to the variable length issue where when an invalid character is read with the standard function std::str::from utf8 this generates an error, where through the function error.valid_up_to() returns the number of valid bytes, then the valid bytes are read and then the iterator is moved to
beginning of the incomplete byte.
In the value structure, you basically define the types, and the conversion traits between the different values ​​to standard rust types.
In the tokenization structure, the reader's iterator is converted into a vector of tokens that contains the primitive types.
And then in the final parsing structure all the tokens are passed to a main object (as it should be in a JSON), and each key is assigned its respective value, either a primitive one like a number, or a complex one like a object or an array.
