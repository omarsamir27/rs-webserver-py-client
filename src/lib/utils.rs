use std::fmt::Display;
use std::path::Path;

pub fn array_stringify<T: Display>(arr: &[T], delim: char) -> String {
    let mut string = String::new();
    for elem in arr {
        string.push_str(elem.to_string().as_str());
        string.push(delim);
    }
    string.pop();
    string
}

pub fn deduce_file_mime(path:&Path) -> String {
    let guess = new_mime_guess::from_path(path);
    if guess.first().is_some() {
        return guess.first().unwrap().to_string()
    }
    else {
        tree_magic::from_filepath(path)
    }
}