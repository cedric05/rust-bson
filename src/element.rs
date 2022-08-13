use super::parse::*;

pub type Decimal = [u8; 128 / 8];
pub type DbPointer = [u8; 12];
pub type JavascriptCode = (String, Document);
pub type KeyPair<T> = (String, T);

#[derive(Debug, PartialEq)]
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

#[derive(Debug, PartialEq)]
pub struct Binary {
    pub binary_type: BinaryType,
    // todo replace with md5, uuid, ....
    pub data: Vec<u8>,
}

#[derive(Clone, Copy, PartialEq, PartialOrd, Eq, Ord, Hash)]
pub struct ObjectId {
    pub id: [u8; 12],
}

impl From<[u8; 12]> for ObjectId {
    fn from(bytes: [u8; 12]) -> Self {
        Self { id: bytes }
    }
}

impl std::fmt::Display for ObjectId {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        f.write_str(&hex::encode(self.id))
    }
}

impl std::fmt::Debug for ObjectId {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        f.debug_tuple("ObjectId")
            .field(&hex::encode(self.id))
            .finish()
    }
}

#[derive(Debug, PartialEq)]
pub enum Element {
    Double(f64),
    String(String),
    EmbededDocument(Document),
    ArrayDocument(Array),
    Binary(Binary),
    Cstring(String, String),
    Undefined,
    ObjectId(ObjectId),
    Boolean(bool),
    DateTime(i64),
    Null,
    RegularExpression { pattern: String, options: String },
    DbPointer([u8; 12]),
    Javascript(String),
    Symbol(String),
    JavascriptCode(String, Document),
    Int32(i32),
    Timestamp(u64),
    Int64(i64),
    Decimal(Decimal),
    Min,
    Max,
}

macro_rules! element_as {
    ($func_name:ident, $type:ident, $element_type:path ) => {
        pub fn $func_name(self) -> Result<$type, BsonError> {
            match self {
                $element_type(res) => Ok(res),
                _ => Err(BsonError::Generic),
            }
        }
    };
}
impl Element {
    element_as!(as_float, f64, Element::Double);
    element_as!(as_string, String, Element::String);
    element_as!(as_document, Document, Element::EmbededDocument);
    element_as!(as_binary, Binary, Element::Binary);
    element_as!(as_object_id, ObjectId, Element::ObjectId);
    element_as!(as_bool, bool, Element::Boolean);
    element_as!(as_datetime, i64, Element::DateTime);
    element_as!(as_dbpointer, DbPointer, Element::DbPointer);
    element_as!(as_javascript, String, Element::Javascript);
    element_as!(as_symbol, String, Element::Symbol);
    element_as!(as_array, Array, Element::ArrayDocument);
    element_as!(as_int32, i32, Element::Int32);
    element_as!(as_timestamp, u64, Element::Timestamp);
    element_as!(as_i64, i64, Element::Int64);
    element_as!(as_decimal128, Decimal, Element::Decimal);

    pub fn is_undefined(&self) -> Result<bool, BsonError> {
        Ok(match self {
            Element::Undefined => true,
            _ => false,
        })
    }

    pub fn is_null(&self) -> Result<bool, BsonError> {
        Ok(match self {
            Element::Null => true,
            _ => false,
        })
    }

    pub fn is_max(&self) -> Result<bool, BsonError> {
        Ok(match self {
            Element::Max => true,
            _ => false,
        })
    }

    pub fn is_min(&self) -> Result<bool, BsonError> {
        Ok(match self {
            Element::Min => true,
            _ => false,
        })
    }
}

#[derive(Debug, PartialEq)]
pub enum BsonError {
    Generic,
    ParseError,
    KeyNotFound,
    Utf8Error,
}

#[derive(Debug, PartialEq)]
pub struct Document {
    pub data: Vec<u8>,
}

#[derive(Debug, PartialEq)]
pub struct Array {
    pub data: Vec<u8>,
}

fn find_value(input: &[u8], key: &str) -> Result<Element, BsonError> {
    let mut mut_input = input;
    while !mut_input.is_empty() {
        let (input1, (ename, element)) = parse_any(mut_input).map_err(|_| BsonError::ParseError)?;
        mut_input = input1;
        if ename == key {
            return Ok(element);
        }
    }
    Err(BsonError::KeyNotFound)
}

macro_rules! match_element_doc {
    ($func_name:ident, $type:ident, $element_type:path ) => {
        pub fn $func_name(&self, key: &str) -> Result<$type, BsonError> {
            let value = self.get_value(key)?;
            match value {
                $element_type(res) => Ok(res),
                _ => Err(BsonError::Generic),
            }
        }
    };
}

macro_rules! match_element_arr {
    ($func_name:ident, $type:ident, $element_type:path ) => {
        pub fn $func_name(&self, key: usize) -> Result<$type, BsonError> {
            let value = self.get_value(key)?;
            match value {
                $element_type(res) => Ok(res),
                _ => Err(BsonError::Generic),
            }
        }
    };
}

impl Document {
    fn get_value(&self, key: &str) -> Result<Element, BsonError> {
        find_value(&self.data, key)
    }

    match_element_doc!(get_float, f64, Element::Double);
    match_element_doc!(get_string, String, Element::String);
    match_element_doc!(get_document, Document, Element::EmbededDocument);
    match_element_doc!(get_array, Array, Element::ArrayDocument);
    match_element_doc!(get_binary, Binary, Element::Binary);
    match_element_doc!(get_object_id, ObjectId, Element::ObjectId);
    match_element_doc!(get_bool, bool, Element::Boolean);
    match_element_doc!(get_datetime, i64, Element::DateTime);
    match_element_doc!(get_dbpointer, DbPointer, Element::DbPointer);
    match_element_doc!(get_javascript, String, Element::Javascript);
    match_element_doc!(get_symbol, String, Element::Symbol);
    match_element_doc!(get_int32, i32, Element::Int32);
    match_element_doc!(get_timestamp, u64, Element::Timestamp);
    match_element_doc!(get_i64, i64, Element::Int64);
    match_element_doc!(get_decimal128, Decimal, Element::Decimal);

    pub fn is_undefined(&self, key: &str) -> Result<bool, BsonError> {
        Ok(match self.get_value(key)? {
            Element::Undefined => true,
            _ => false,
        })
    }

    pub fn is_null(&self, key: &str) -> Result<bool, BsonError> {
        Ok(match self.get_value(key)? {
            Element::Null => true,
            _ => false,
        })
    }

    pub fn is_max(&self, key: &str) -> Result<bool, BsonError> {
        Ok(match self.get_value(key)? {
            Element::Max => true,
            _ => false,
        })
    }

    pub fn is_min(&self, key: &str) -> Result<bool, BsonError> {
        Ok(match self.get_value(key)? {
            Element::Min => true,
            _ => false,
        })
    }

    pub fn get_any(&self, key: &str) -> Result<Element, BsonError> {
        self.get_value(key)
    }

    pub fn iter<'a>(&'a self) -> DocumentIter<'a> {
        return DocumentIter { doc: &self.data };
    }
}

impl Array {
    fn get_value(&self, key: usize) -> Result<Element, BsonError> {
        find_value(&self.data, key.to_string().as_str())
    }

    pub fn get_any(&self, key: usize) -> Result<Element, BsonError> {
        self.get_value(key)
    }

    match_element_arr!(get_float, f64, Element::Double);
    match_element_arr!(get_string, String, Element::String);
    match_element_arr!(get_document, Document, Element::EmbededDocument);
    match_element_arr!(get_array, Array, Element::ArrayDocument);
    match_element_arr!(get_binary, Binary, Element::Binary);
    match_element_arr!(get_object_id, ObjectId, Element::ObjectId);
    match_element_arr!(get_bool, bool, Element::Boolean);
    match_element_arr!(get_datetime, i64, Element::DateTime);
    match_element_arr!(get_javascript, String, Element::Javascript);
    match_element_arr!(get_symbol, String, Element::Symbol);
    match_element_arr!(get_dbpointer, DbPointer, Element::DbPointer);
    match_element_arr!(get_int32, i32, Element::Int32);
    match_element_arr!(get_timestamp, u64, Element::Timestamp);
    match_element_arr!(get_i64, i64, Element::Int64);
    match_element_arr!(get_decimal128, Decimal, Element::Decimal);
    pub fn is_undefined(&self, key: usize) -> Result<bool, BsonError> {
        Ok(match self.get_value(key)? {
            Element::Undefined => true,
            _ => false,
        })
    }

    pub fn is_null(&self, key: usize) -> Result<bool, BsonError> {
        Ok(match self.get_value(key)? {
            Element::Null => true,
            _ => false,
        })
    }

    pub fn is_max(&self, key: usize) -> Result<bool, BsonError> {
        Ok(match self.get_value(key)? {
            Element::Max => true,
            _ => false,
        })
    }

    pub fn is_min(&self, key: usize) -> Result<bool, BsonError> {
        Ok(match self.get_value(key)? {
            Element::Min => true,
            _ => false,
        })
    }

    pub fn iter<'a>(&'a self) -> DocumentIter<'a> {
        return DocumentIter { doc: &self.data };
    }
}

impl TryFrom<&[u8]> for Document {
    type Error = BsonError;

    fn try_from(input: &[u8]) -> Result<Self, Self::Error> {
        let (_input, doc) = parse_document(input).map_err(|_| BsonError::Generic)?;
        Ok(doc)
    }
}

pub struct DocumentIter<'a> {
    doc: &'a [u8],
}

impl<'a> Iterator for DocumentIter<'a> {
    type Item = KeyPair<Element>;

    fn next(&mut self) -> Option<Self::Item> {
        if !self.doc.is_empty() {
            match parse_any(self.doc) {
                Ok((input, (name, element))) => {
                    self.doc = input;
                    Some((name, element))
                }
                Err(_) => None,
            }
        } else {
            None
        }
    }
}
