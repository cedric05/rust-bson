use nom::{
    bytes::complete::{tag, take, take_until},
    error::ParseError,
    number::complete::{be_u8, le_f64, le_i32, le_i64, le_u64},
    sequence::tuple,
    IResult,
};

use super::element::*;

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

pub(crate) fn parse_decimal_128(input: &[u8]) -> IResult<&[u8], KeyPair<Element>> {
    let (input, (_, ename, decimal)) = tuple((
        tag(&[ELEMENT_TYPE_DECIMAL128]),
        parse_estring,
        take(16usize),
    ))(input)?;
    let decimal = [
        decimal[0],
        decimal[1],
        decimal[2],
        decimal[3],
        decimal[4],
        decimal[5],
        decimal[6],
        decimal[7],
        decimal[8],
        decimal[9],
        decimal[10],
        decimal[11],
        decimal[12],
        decimal[13],
        decimal[14],
        decimal[15],
    ];
    Ok((input, (ename, Element::Decimal(decimal))))
}

pub(crate) fn parse_double(input: &[u8]) -> IResult<&[u8], KeyPair<Element>> {
    let (input, (_, ename, double)) =
        tuple((tag(&[ELEMENT_TYPE_DOUBLE]), parse_estring, le_f64))(input)?;
    Ok((input, (ename, Element::Double(double))))
}

pub(crate) fn parse_int32(input: &[u8]) -> IResult<&[u8], KeyPair<Element>> {
    let (input, (_, ename, int32)) =
        tuple((tag(&[ELEMENT_TYPE_INT32]), parse_estring, le_i32))(input)?;
    Ok((input, (ename, Element::Int32(int32))))
}

pub(crate) fn parse_int64(input: &[u8]) -> IResult<&[u8], KeyPair<Element>> {
    let (input, (_, ename, int64)) =
        tuple((tag(&[ELEMENT_TYPE_INT64]), parse_estring, le_i64))(input)?;
    Ok((input, (ename, Element::Int64(int64))))
}

pub(crate) fn parse_uint64(input: &[u8]) -> IResult<&[u8], KeyPair<Element>> {
    let (input, (_, ename, uint64)) =
        tuple((tag(&[ELEMENT_TYPE_TIMESTAMP]), parse_estring, le_u64))(input)?;
    Ok((input, (ename, Element::Timestamp(uint64))))
}

pub(crate) fn parse_null(input: &[u8]) -> IResult<&[u8], KeyPair<Element>> {
    let (input, (_, ename)) = tuple((tag(&[ELEMENT_TYPE_NULL]), parse_estring))(input)?;
    Ok((input, (ename, Element::Null)))
}

pub(crate) fn parse_symbol(input: &[u8]) -> IResult<&[u8], KeyPair<Element>> {
    let (input, (_, ename, size)) =
        tuple((tag(&[ELEMENT_TYPE_SYMBOL]), parse_estring, le_i32))(input)?;
    let (input, string) = take(size as usize - 1)(input)?;
    let (input, _) = be_u8(input)?;
    let string = map_utf8_error(input, string)?;
    Ok((input, (ename, Element::Symbol(string))))
}

pub(crate) fn parse_string(input: &[u8]) -> IResult<&[u8], KeyPair<Element>> {
    let (input, (_, ename, size)) =
        tuple((tag(&[ELEMENT_TYPE_STRING]), parse_estring, le_i32))(input)?;
    let (input, string) = take(size as usize - 1)(input)?;
    let (input, _) = be_u8(input)?;
    let string = map_utf8_error(input, string)?;
    Ok((input, (ename, Element::String(string))))
}

pub(crate) fn parse_javascript(input: &[u8]) -> IResult<&[u8], KeyPair<Element>> {
    let (input, (_, ename, _size)) =
        tuple((tag(&[ELEMENT_TYPE_JAVASCRIPTCODE]), parse_estring, le_i32))(input)?;
    let (input, string) = take(_size as usize)(input)?;
    let (input, _) = be_u8(input)?;
    let string = map_utf8_error(input, string)?;
    Ok((input, (ename, Element::Javascript(string))))
}

pub(crate) fn parse_javascript_with_scope(input: &[u8]) -> IResult<&[u8], KeyPair<Element>> {
    let (input, (_, ename, total_size, string_size)) = tuple((
        tag(&[ELEMENT_TYPE_JAVASCRIPTCODE]),
        parse_estring,
        le_i32,
        le_i32,
    ))(input)?;

    let (input, string) = take(string_size as usize)(input)?;
    let (input, _null_byte) = be_u8(input)?;
    let (input, sub_document) = take((total_size - string_size - 1) as usize)(input)?;
    let (_, embed_document_size) = le_i32(sub_document)?;
    let (_, document) = take((embed_document_size - 5) as usize)(sub_document)?;
    let (input, _) = be_u8(input)?;
    let string = map_utf8_error(input, string)?;
    Ok((
        input,
        (
            ename,
            Element::JavascriptCode(
                string,
                Document {
                    data: document.to_vec(),
                },
            ),
        ),
    ))
}

pub(crate) fn parse_object_id(input: &[u8]) -> IResult<&[u8], KeyPair<Element>> {
    let (input, (_, ename, ar)) =
        tuple((tag(&[ELEMENT_TYPE_OBJECT_ID]), parse_estring, take(12usize)))(input)?;
    let ob = [
        ar[0], ar[1], ar[2], ar[3], ar[4], ar[5], ar[6], ar[7], ar[8], ar[9], ar[10], ar[11],
    ]
    .into();
    Ok((input, (ename, Element::ObjectId(ob))))
}

pub(crate) fn parse_boolean(input: &[u8]) -> IResult<&[u8], KeyPair<Element>> {
    let (input, (_, ename, double)) =
        tuple((tag(&[ELEMENT_TYPE_BOOLEAN]), parse_estring, be_u8))(input)?;
    Ok((input, (ename, Element::Boolean(double != 0))))
}

fn map_utf8_error<'a>(
    input: &'a [u8],
    ename: &'a [u8],
) -> Result<String, nom::Err<nom::error::Error<&'a [u8]>>> {
    match String::from_utf8(ename.to_vec()) {
        Ok(ename) => Ok(ename),
        Err(_) => Err(nom::Err::Error(nom::error::Error {
            input,
            code: nom::error::ErrorKind::Fail,
        })),
    }
}

pub(crate) fn parse_estring(input: &[u8]) -> IResult<&[u8], String> {
    let (input, ename) = (take_until(NULL_BYTE))(input)?;
    let (input, _ignore_x00) = be_u8(input)?;
    let ename = map_utf8_error(input, ename)?;
    Ok((input, ename))
}

pub(crate) fn parse_cstring(input: &[u8]) -> IResult<&[u8], KeyPair<Element>> {
    let (input, ename) = parse_estring(input)?;
    let (input, cstring1) = (take_until(NULL_BYTE))(input)?;
    let (input, _) = be_u8(input)?;
    let cstring1 = map_utf8_error(input, cstring1)?;
    let (input, cstring2) = (take_until(NULL_BYTE))(input)?;
    let (input, _) = be_u8(input)?;
    let cstring2 = map_utf8_error(input, cstring2)?;
    Ok((input, (ename, Element::Cstring(cstring1, cstring2))))
}

pub(crate) fn parse_dbpointer(input: &[u8]) -> IResult<&[u8], KeyPair<Element>> {
    let (input, (_, ename, arr)) =
        tuple((tag(&[ELEMENT_TYPE_DBPOINTER]), parse_estring, take(12usize)))(input)?;
    let db_pointer = [
        arr[0], arr[1], arr[2], arr[3], arr[4], arr[5], arr[6], arr[7], arr[8], arr[9], arr[10],
        arr[11],
    ];
    Ok((input, (ename, Element::ObjectId(db_pointer.into()))))
}

pub(crate) fn parse_embeded_document(input: &[u8]) -> IResult<&[u8], KeyPair<Element>> {
    let (input, (_, ename, _size)) =
        tuple((tag(&[ELEMENT_TYPE_EMBED_DOCUMENT]), parse_estring, le_i32))(input)?;
    let (input, next_doc) = take(_size as usize - 5)(input)?;
    let (input, _) = be_u8(input)?;
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

pub(crate) fn parse_document(input: &[u8]) -> IResult<&[u8], Document> {
    let (input, size) = le_i32(input)?;
    let (input, next_doc) = take(size as usize - 5)(input)?;
    Ok((
        input,
        Document {
            data: next_doc.to_vec(),
        },
    ))
}

pub(crate) fn parse_array_document(input: &[u8]) -> IResult<&[u8], KeyPair<Element>> {
    let (input, (_, ename, _size)) =
        tuple((tag(&[ELEMENT_TYPE_ARRAY_DOCUMENT]), parse_estring, le_i32))(input)?;
    let (input, next_doc) = take(_size as usize - 5)(input)?;
    let (input, _) = be_u8(input)?;
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

pub(crate) fn parse_min(input: &[u8]) -> IResult<&[u8], KeyPair<Element>> {
    let (input, (_, ename)) = tuple((tag(&[ELEMENT_TYPE_MIN]), parse_estring))(input)?;
    Ok((input, (ename, Element::Min)))
}

pub(crate) fn parse_undefined(input: &[u8]) -> IResult<&[u8], KeyPair<Element>> {
    let (input, (_, ename)) = tuple((tag(&[ELEMENT_TYPE_UNDEFINED]), parse_estring))(input)?;
    Ok((input, (ename, Element::Min)))
}

pub(crate) fn parse_max(input: &[u8]) -> IResult<&[u8], KeyPair<Element>> {
    let (input, (_, ename)) = tuple((tag(&[ELEMENT_TYPE_MAX]), parse_estring))(input)?;
    Ok((input, (ename, Element::Max)))
}

pub(crate) fn parse_binary(input: &[u8]) -> IResult<&[u8], KeyPair<Element>> {
    let (input, (_, ename, size, binary_type)) =
        tuple((tag(&[ELEMENT_TYPE_BINARY]), parse_estring, le_i32, be_u8))(input)?;

    let (input, byte_array) = take(size as usize)(input)?;

    match BinaryType::try_from(binary_type) {
        Ok(binary_type) => Ok((
            input,
            (
                ename,
                Element::Binary(Binary {
                    binary_type,
                    data: byte_array.to_vec(),
                }),
            ),
        )),
        Err(_err) => Err(nom::Err::Error(nom::error::Error {
            input,
            code: nom::error::ErrorKind::Fail,
        })),
    }
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
        ELEMENT_TYPE_SYMBOL => parse_symbol,
        ELEMENT_TYPE_JAVASCRIPTCODEWITHSCOPE => parse_javascript_with_scope,
        ELEMENT_TYPE_INT32 => parse_int32,
        ELEMENT_TYPE_TIMESTAMP => parse_uint64,
        ELEMENT_TYPE_INT64 => parse_int64,
        ELEMENT_TYPE_DECIMAL128 => parse_decimal_128,
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
