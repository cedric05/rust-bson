use std::fmt::Debug;

use nom::{
    bytes::complete::{tag, take, take_until},
    error::ParseError,
    number::complete::{be_u8, le_f64, le_i32, le_i64, le_u64},
    sequence::tuple,
    IResult,
};

const ELEMENT_TYPE_DOUBLE: u8 = 0x01;
const ELEMENT_TYPE_STRING: u8 = 0x02;
const ELEMENT_TYPE_EMBED_DOCUMENT: u8 = 0x03;
const ELEMENT_TYPE_ARRAY_DOCUMENT: u8 = 0x04;
const ELEMENT_TYPE_BINARY: u8 = 0x05;
const ELEMENT_TYPE_UNDEFINED: u8 = 0x06;
const ELEMENT_TYPE_OBJECT_ID: u8 = 0x07;
const ELEMENT_TYPE_BOOLEAN: u8 = 0x08;
const ELEMENT_TYPE_DATETIME: u8 = 0x09;
const ELEMENT_TYPE_NULL: u8 = 0x0A;
const ELEMENT_TYPE_CSTRING: u8 = 0x0B;
const ELEMENT_TYPE_DBPOINTER: u8 = 0x0C;
const ELEMENT_TYPE_JAVASCRIPTCODE: u8 = 0x0D;
const ELEMENT_TYPE_SYMBOL: u8 = 0x0E;
const ELEMENT_TYPE_JAVASCRIPTCODEWITHSCOPE: u8 = 0x0F;
const ELEMENT_TYPE_INT32: u8 = 0x10;
const ELEMENT_TYPE_TIMESTAMP: u8 = 0x11;
const ELEMENT_TYPE_INT64: u8 = 0x12;
const ELEMENT_TYPE_DECIMAL128: u8 = 0x13;
const ELEMENT_TYPE_MIN: u8 = 0xFF;
const ELEMENT_TYPE_MAX: u8 = 0x7F;

const NULL_BYTE: &str = "\x00";

#[derive(Debug)]
pub enum BinaryType {
    BinaryGeneric = 0x00,
    BinaryFunction = 0x01,
    BinaryBinary = 0x02,
    BinaryOldUuid = 0x03,
    BinaryUuid = 0x04,
    BinaryMd5 = 0x05,
    BinaryEncrypted = 0x06,
    BinaryCompressed = 0x07,
    BinaryUserDefined = 0x80,
}

impl TryFrom<u8> for BinaryType {
    type Error = BsonError;

    fn try_from(value: u8) -> Result<Self, Self::Error> {
        let value = match value {
            0x00 => BinaryType::BinaryGeneric,
            0x01 => BinaryType::BinaryFunction,
            0x02 => BinaryType::BinaryBinary,
            0x03 => BinaryType::BinaryOldUuid,
            0x04 => BinaryType::BinaryUuid,
            0x05 => BinaryType::BinaryMd5,
            0x06 => BinaryType::BinaryEncrypted,
            0x07 => BinaryType::BinaryCompressed,
            0x80 => BinaryType::BinaryUserDefined,
            _ => todo!(),
        };
        Ok(value)
    }
}

#[derive(Debug)]
pub struct Binary {
    pub binary_type: BinaryType,
    // todo replace with md5, uuid, ....
    data: Vec<u8>,
}

impl Binary {
    pub fn data(&self) -> &[u8] {
        &self.data
    }
}

#[derive(Debug)]
pub enum Element {
    Double(f64),
    String(String),
    EmbededDocument(Document),
    ArrayDocument(Array),
    Binary(Binary),
    Cstring(String, String),
    Undefined,
    ObjectId([u8; 12]),
    Boolean(bool),
    DateTime(u64),
    Null,
    RegularExpression { pattern: String, options: String },
    DbPointer([u8; 12]),
    Javascript(String),
    JavascriptCode(String, Document),
    Int32(i32),
    Timestamp(u64),
    Int64(i64),
    Decimal(f64),
    Min,
    Max,
}

pub type KeyPair<T> = (String, T);

fn parse_double(input: &[u8]) -> IResult<&[u8], KeyPair<Element>> {
    let (input, (_, ename, double)) =
        tuple((tag(&[ELEMENT_TYPE_DOUBLE]), parse_estring, le_f64))(input)?;
    Ok((input, (ename, Element::Double(double))))
}

fn parse_int32(input: &[u8]) -> IResult<&[u8], KeyPair<Element>> {
    let (input, (_, ename, int32)) =
        tuple((tag(&[ELEMENT_TYPE_INT32]), parse_estring, le_i32))(input)?;
    Ok((input, (ename, Element::Int32(int32))))
}

fn parse_int64(input: &[u8]) -> IResult<&[u8], KeyPair<Element>> {
    let (input, (_, ename, int64)) =
        tuple((tag(&[ELEMENT_TYPE_INT64]), parse_estring, le_i64))(input)?;
    Ok((input, (ename, Element::Int64(int64))))
}

fn parse_uint64(input: &[u8]) -> IResult<&[u8], KeyPair<Element>> {
    let (input, (_, ename, uint64)) =
        tuple((tag(&[ELEMENT_TYPE_TIMESTAMP]), parse_estring, le_u64))(input)?;
    Ok((input, (ename, Element::Timestamp(uint64))))
}

fn parse_null(input: &[u8]) -> IResult<&[u8], KeyPair<Element>> {
    let (input, (_, ename)) = tuple((tag(&[ELEMENT_TYPE_NULL]), parse_estring))(input)?;
    Ok((input, (ename, Element::Null)))
}

fn parse_string(input: &[u8]) -> IResult<&[u8], KeyPair<Element>> {
    let (input, (_, ename, size)) =
        tuple((tag(&[ELEMENT_TYPE_STRING]), parse_estring, le_i32))(input)?;
    let (input, string) = take(size as usize - 1)(input)?;
    let (input, _) = be_u8(input)?;
    Ok((
        input,
        (
            ename,
            Element::String(String::from_utf8(string.to_vec()).unwrap()),
        ),
    ))
}

fn parse_javascript(input: &[u8]) -> IResult<&[u8], KeyPair<Element>> {
    let (input, (_, ename, _size)) =
        tuple((tag(&[ELEMENT_TYPE_JAVASCRIPTCODE]), parse_estring, le_i32))(input)?;
    let (input, string) = take(_size as usize)(input)?;
    let (input, _) = be_u8(input)?;
    Ok((
        input,
        (
            ename,
            Element::Javascript(String::from_utf8(string.to_vec()).unwrap()),
        ),
    ))
}

fn parse_object_id(input: &[u8]) -> IResult<&[u8], KeyPair<Element>> {
    let (input, (_, ename, ar)) =
        tuple((tag(&[ELEMENT_TYPE_OBJECT_ID]), parse_estring, take(12usize)))(input)?;
    let ob = [
        ar[0], ar[1], ar[2], ar[3], ar[4], ar[5], ar[6], ar[7], ar[8], ar[9], ar[10], ar[11],
    ];
    Ok((input, (ename, Element::ObjectId(ob))))
}

fn parse_boolean(input: &[u8]) -> IResult<&[u8], KeyPair<Element>> {
    let (input, (_, ename, double)) =
        tuple((tag(&[ELEMENT_TYPE_BOOLEAN]), parse_estring, be_u8))(input)?;
    Ok((input, (ename, Element::Boolean(double != 0))))
}

fn parse_estring(input: &[u8]) -> IResult<&[u8], String> {
    let (input, ename) = (take_until(NULL_BYTE))(input)?;
    let (input, _ignore_x00) = be_u8(input)?;
    Ok((input, (String::from_utf8(ename.to_vec()).unwrap())))
}

fn parse_cstring(input: &[u8]) -> IResult<&[u8], KeyPair<Element>> {
    let (input, ename) = parse_estring(input)?;
    let (input, cstring1) = (take_until(NULL_BYTE))(input)?;
    let (input, _) = be_u8(input)?;
    let cstring1 = String::from_utf8(cstring1.to_vec()).unwrap();
    let (input, cstring2) = (take_until(NULL_BYTE))(input)?;
    let (input, _) = be_u8(input)?;
    let cstring2 = String::from_utf8(cstring2.to_vec()).unwrap();
    Ok((input, (ename, Element::Cstring(cstring1, cstring2))))
}

fn parse_dbpointer(input: &[u8]) -> IResult<&[u8], KeyPair<Element>> {
    let (input, (_, ename, double)) =
        tuple((tag(&[ELEMENT_TYPE_DBPOINTER]), parse_estring, take(12usize)))(input)?;
    let var_name = [
        double[0], double[1], double[2], double[3], double[4], double[5], double[6], double[7],
        double[8], double[9], double[10], double[11],
    ];
    Ok((input, (ename, Element::ObjectId(var_name))))
}
fn parse_embeded_document(input: &[u8]) -> IResult<&[u8], KeyPair<Element>> {
    let (input, (_, ename, _size)) =
        tuple((tag(&[ELEMENT_TYPE_EMBED_DOCUMENT]), parse_estring, le_i32))(input)?;
    let (input, next_doc) = take(_size as usize)(input)?;
    Ok((
        input,
        (
            ename,
            Element::EmbededDocument(Document {
                data: next_doc.to_vec(),
            }),
        ),
    ))
}

fn parse_document(input: &[u8]) -> IResult<&[u8], Document> {
    let (input, size) = le_i32(input)?;
    let (input, next_doc) = take(size as usize - 5)(input)?;
    Ok((
        input,
        Document {
            data: next_doc.to_vec(),
        },
    ))
}

fn parse_array_document(input: &[u8]) -> IResult<&[u8], KeyPair<Element>> {
    let (input, (_, ename, _size)) =
        tuple((tag(&[ELEMENT_TYPE_ARRAY_DOCUMENT]), parse_estring, le_i32))(input)?;
    let (input, next_doc) = take(_size as usize)(input)?;
    Ok((
        input,
        (
            ename,
            Element::ArrayDocument(Array {
                data: next_doc.to_vec(),
            }),
        ),
    ))
}

fn parse_min(input: &[u8]) -> IResult<&[u8], KeyPair<Element>> {
    let (input, (_, ename)) = tuple((tag(&[ELEMENT_TYPE_MIN]), parse_estring))(input)?;
    Ok((input, (ename, Element::Min)))
}

fn parse_undefined(input: &[u8]) -> IResult<&[u8], KeyPair<Element>> {
    let (input, (_, ename)) = tuple((tag(&[ELEMENT_TYPE_UNDEFINED]), parse_estring))(input)?;
    Ok((input, (ename, Element::Min)))
}

fn parse_max(input: &[u8]) -> IResult<&[u8], KeyPair<Element>> {
    let (input, (_, ename)) = tuple((tag(&[ELEMENT_TYPE_MAX]), parse_estring))(input)?;
    Ok((input, (ename, Element::Max)))
}

fn parse_binary(input: &[u8]) -> IResult<&[u8], KeyPair<Element>> {
    let (input, (_, ename, size, binary_type)) =
        tuple((tag(&[ELEMENT_TYPE_BINARY]), parse_estring, le_i32, be_u8))(input)?;

    let (input, byte_array) = take(size as usize)(input)?;

    Ok((
        input,
        (
            ename,
            Element::Binary(Binary {
                binary_type: BinaryType::try_from(binary_type).unwrap(),
                data: byte_array.to_vec(),
            }),
        ),
    ))
}
pub fn parse_any(input: &[u8]) -> IResult<&[u8], KeyPair<Element>> {
    if input.len() == 0 {
        return Err(nom::Err::Error(nom::error::Error::from_error_kind(
            input,
            nom::error::ErrorKind::Fail,
        )));
    }
    let element_type = input[0];

    let (input, out) = match element_type {
        ELEMENT_TYPE_DOUBLE => parse_double,
        ELEMENT_TYPE_STRING => parse_string,
        ELEMENT_TYPE_EMBED_DOCUMENT => parse_embeded_document,
        ELEMENT_TYPE_ARRAY_DOCUMENT => parse_array_document,
        ELEMENT_TYPE_BINARY => parse_binary,
        ELEMENT_TYPE_UNDEFINED => parse_undefined,
        ELEMENT_TYPE_OBJECT_ID => parse_object_id,
        ELEMENT_TYPE_BOOLEAN => parse_boolean,
        ELEMENT_TYPE_DATETIME => parse_int64,
        ELEMENT_TYPE_NULL => parse_null,
        ELEMENT_TYPE_CSTRING => parse_cstring,
        ELEMENT_TYPE_DBPOINTER => parse_dbpointer,
        ELEMENT_TYPE_JAVASCRIPTCODE => parse_javascript,
        ELEMENT_TYPE_SYMBOL => parse_string,
        ELEMENT_TYPE_JAVASCRIPTCODEWITHSCOPE => todo!(),
        ELEMENT_TYPE_INT32 => parse_int32,
        ELEMENT_TYPE_TIMESTAMP => parse_uint64,
        ELEMENT_TYPE_INT64 => parse_int64,
        ELEMENT_TYPE_DECIMAL128 => todo!(),
        ELEMENT_TYPE_MIN => parse_min,
        ELEMENT_TYPE_MAX => parse_max,
        _ => {
            return Err(nom::Err::Error(nom::error::Error::from_error_kind(
                input,
                nom::error::ErrorKind::Fail,
            )));
        }
    }(input)?;
    Ok((input, out))
}

#[derive(Debug)]
pub enum BsonError {
    Generic,
    NomError,
}

#[derive(Debug)]
pub struct Document {
    data: Vec<u8>,
}

#[derive(Debug)]
pub struct Array {
    data: Vec<u8>,
}

fn find_value(input: &[u8], key: &str) -> Result<Element, BsonError> {
    let mut mut_input = input;
    while !mut_input.is_empty() {
        let (input, (ename, element)) = parse_any(input).map_err(|_| BsonError::NomError)?;
        mut_input = input;
        if ename == key {
            return Ok(element);
        }
    }
    Err(BsonError::Generic)
}

impl Document {
    fn get_value(&self, key: &str) -> Result<Element, BsonError> {
        find_value(&self.data, key)
    }

    pub fn get_string(&self, key: &str) -> Result<String, BsonError> {
        let value = self.get_value(key)?;
        match value {
            Element::String(res) => Ok(res),
            _ => Err(BsonError::Generic),
        }
    }

    pub fn get_int32(&self, key: &str) -> Result<i32, BsonError> {
        let value = self.get_value(key)?;
        match value {
            Element::Int32(res) => Ok(res),
            _ => Err(BsonError::Generic),
        }
    }

    pub fn get_any(&self, key: &str) -> Result<Element, BsonError> {
        self.get_value(key)
    }
}

impl Array {
    fn get_value(&self, key: usize) -> Result<Element, BsonError> {
        find_value(&self.data, key.to_string().as_str())
    }

    pub fn get_string(&self, key: usize) -> Result<String, BsonError> {
        let value = self.get_value(key)?;
        match value {
            Element::String(res) => Ok(res),
            _ => Err(BsonError::Generic),
        }
    }
    pub fn get_any(&self, key: usize) -> Result<Element, BsonError> {
        self.get_value(key)
    }
}

impl TryFrom<&[u8]> for Document {
    type Error = BsonError;

    fn try_from(input: &[u8]) -> Result<Self, Self::Error> {
        let (_input, doc) = parse_document(input).map_err(|_| BsonError::Generic)?;
        Ok(doc)
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_string_key() {
        let value: &[u8] = &[16, 0, 0, 0, 2, 104, 105, 0, 3, 0, 0, 0, 104, 105, 0, 0];
        let doc = Document::try_from(value).unwrap();
        let string = doc.get_string("hi").unwrap();
        assert_eq!("hi", string);
    }

    #[test]
    fn test_double() {
        let value: &[u8] = &[13, 0, 0, 0, 16, 104, 105, 0, 5, 0, 0, 0, 0];
        let doc = Document::try_from(value).unwrap();
        let int = doc.get_int32("hi").unwrap();
        assert_eq!(5, int);
    }
}
