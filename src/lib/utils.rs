use std::fmt::Display;

pub fn array_stringify<T:Display>(arr:&[T], delim:char) -> String{
    let mut string = String::new();
    for elem in arr{
        string.push_str(elem.to_string().as_str());
        string.push(delim);
    };
    string.pop();
    string
}
