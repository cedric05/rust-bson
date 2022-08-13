/// Parses Bson from byte array
///
///
/// ```rust
/// use bson2::Document;
///
/// let value: &[u8] = &[5, 0, 0, 0, 0];
/// let doc = Document::try_from(value);
/// assert_eq!(Ok(Document { data: [].to_vec() }), doc);
///
/// let value: &[u8] = &[16, 0, 0, 0, 2, 104, 105, 0, 3, 0, 0, 0, 104, 105, 0, 0];
/// let doc = Document::try_from(value).unwrap();
/// let string = doc.get_string("hi").unwrap();
/// assert_eq!("hi", string);
/// for (name, value) in doc.iter(){
///     println!("key is {name:?} and value is {value:?}")
/// }
/// ```
pub mod element;
pub mod parse;

pub use element::*;
#[cfg(test)]
mod test;
