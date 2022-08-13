use super::element::*;

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

#[test]
fn test_array() {
    let value: &[u8] = &[
        21, 0, 0, 0, 4, 104, 105, 0, 12, 0, 0, 0, 16, 48, 0, 1, 0, 0, 0, 0, 0,
    ];
    let doc = Document::try_from(value).unwrap();
    let int = doc.get_array("hi").unwrap();
    let out = int.get_int32(0).unwrap();
    assert_eq!(1, out);
}

#[test]
fn test_object_id() {
    let value: &[u8] = &[
        21, 0, 0, 0, 7, 104, 105, 0, 98, 245, 249, 202, 24, 228, 234, 219, 142, 160, 247, 91, 0,
    ];
    let doc = Document::try_from(value).unwrap();
    let object_id = doc.get_object_id("hi").unwrap();
    assert_eq!(
        ObjectId {
            id: [0x62, 0xf5, 0xf9, 0xca, 0x18, 0xe4, 0xea, 0xdb, 0x8e, 0xa0, 0xf7, 0x5b]
        },
        object_id
    );

    assert_eq!(format!("{}", object_id), "62f5f9ca18e4eadb8ea0f75b");
    assert_eq!(
        format!("{:?}", object_id),
        "ObjectId(\"62f5f9ca18e4eadb8ea0f75b\")"
    );
}

#[test]
fn test_empty() {
    let value: &[u8] = &[5, 0, 0, 0, 0];
    let doc = Document::try_from(value);
    assert_eq!(Ok(Document { data: [].to_vec() }), doc);
}

#[test]
fn test_iter() {
    let value: &[u8] = &[
        21, 0, 0, 0, 7, 104, 105, 0, 98, 245, 249, 202, 24, 228, 234, 219, 142, 160, 247, 91, 0,
    ];
    let doc = Document::try_from(value).unwrap();
    let object_id = doc.iter().next();
    assert_eq!(
        Some((
            "hi".to_string(),
            Element::ObjectId(ObjectId {
                id: [0x62, 0xf5, 0xf9, 0xca, 0x18, 0xe4, 0xea, 0xdb, 0x8e, 0xa0, 0xf7, 0x5b]
            })
        )),
        object_id
    );
}

#[test]
fn test_complex() {
    let value: &[u8] = &[
        157, 0, 0, 0, 8, 98, 111, 111, 108, 0, 1, 2, 115, 116, 114, 105, 110, 103, 0, 7, 0, 0, 0,
        115, 116, 114, 105, 110, 103, 0, 1, 102, 108, 111, 97, 116, 0, 92, 143, 194, 245, 40, 92,
        11, 64, 4, 97, 114, 114, 97, 121, 0, 76, 0, 0, 0, 16, 48, 0, 1, 0, 0, 0, 8, 49, 0, 1, 8,
        50, 0, 0, 1, 51, 0, 0, 0, 0, 0, 0, 0, 240, 63, 3, 52, 0, 20, 0, 0, 0, 2, 116, 101, 115,
        116, 0, 5, 0, 0, 0, 116, 101, 115, 116, 0, 0, 16, 53, 0, 100, 0, 0, 0, 7, 54, 0, 98, 246,
        223, 90, 2, 39, 224, 203, 106, 0, 169, 25, 0, 10, 110, 117, 108, 108, 0, 3, 100, 105, 99,
        116, 0, 16, 0, 0, 0, 2, 104, 105, 0, 3, 0, 0, 0, 104, 105, 0, 0, 0,
    ];
    let doc = Document::try_from(value).unwrap();
    assert_eq!(Ok(true), doc.get_bool("bool"));
    assert_eq!(Ok("string".to_string()), doc.get_string("string"));
    assert_eq!(Ok(3.42), doc.get_float("float"));
    assert_eq!(Ok(3.42), doc.get_float("float"));
    let arry = doc.get_array("array").unwrap();
    assert_eq!(Ok(1), arry.get_int32(0));
    assert_eq!(Ok(true), arry.get_bool(1));
    assert_eq!(Ok(false), arry.get_bool(2));
    assert_eq!(Ok(1.0), arry.get_float(3));
    let document = arry.get_document(4).unwrap();
    assert_eq!(Ok("test".to_string()), document.get_string("test"));
    assert_eq!(Ok(100), arry.get_int32(5));
    assert_eq!(
        Ok(ObjectId {
            id: [0x62, 0xf6, 0xdf, 0x5a, 0x02, 0x27, 0xe0, 0xcb, 0x6a, 0x00, 0xa9, 0x19]
        }),
        arry.get_object_id(6)
    );
    assert_eq!(Ok(Element::Null), doc.get_any("null"));
    assert_eq!(
        Ok("hi".to_string()),
        doc.get_document("dict").unwrap().get_string("hi")
    );
}

#[test]
fn test_complex_iter() {
    let value: &[u8] = &[
        157, 0, 0, 0, 8, 98, 111, 111, 108, 0, 1, 2, 115, 116, 114, 105, 110, 103, 0, 7, 0, 0, 0,
        115, 116, 114, 105, 110, 103, 0, 1, 102, 108, 111, 97, 116, 0, 92, 143, 194, 245, 40, 92,
        11, 64, 4, 97, 114, 114, 97, 121, 0, 76, 0, 0, 0, 16, 48, 0, 1, 0, 0, 0, 8, 49, 0, 1, 8,
        50, 0, 0, 1, 51, 0, 0, 0, 0, 0, 0, 0, 240, 63, 3, 52, 0, 20, 0, 0, 0, 2, 116, 101, 115,
        116, 0, 5, 0, 0, 0, 116, 101, 115, 116, 0, 0, 16, 53, 0, 100, 0, 0, 0, 7, 54, 0, 98, 246,
        223, 90, 2, 39, 224, 203, 106, 0, 169, 25, 0, 10, 110, 117, 108, 108, 0, 3, 100, 105, 99,
        116, 0, 16, 0, 0, 0, 2, 104, 105, 0, 3, 0, 0, 0, 104, 105, 0, 0, 0,
    ];
    let document = Document::try_from(value).unwrap();
    let mut doc = document.iter();
    assert_eq!(
        ("bool".to_string(), Element::Boolean(true)),
        doc.next().unwrap()
    );
    assert_eq!(
        ("string".to_string(), Element::String("string".to_string())),
        doc.next().unwrap()
    );

    assert_eq!(
        ("float".to_string(), Element::Double(3.42)),
        doc.next().unwrap()
    );
    let arry = doc.next().unwrap().1.as_array().unwrap();
    assert_eq!(Ok(1), arry.get_int32(0));
    assert_eq!(Ok(true), arry.get_bool(1));
    assert_eq!(Ok(false), arry.get_bool(2));
    assert_eq!(Ok(1.0), arry.get_float(3));
    let document = arry.get_document(4).unwrap();
    assert_eq!(Ok("test".to_string()), document.get_string("test"));
    assert_eq!(Ok(100), arry.get_int32(5));
    assert_eq!(
        Ok(ObjectId {
            id: [0x62, 0xf6, 0xdf, 0x5a, 0x02, 0x27, 0xe0, 0xcb, 0x6a, 0x00, 0xa9, 0x19]
        }),
        arry.get_object_id(6)
    );
    assert_eq!(Element::Null, doc.next().unwrap().1);
}

#[test]
fn test_binary() {
    let data: &[u8] = &[16, 0, 0, 0, 5, 97, 0, 3, 0, 0, 0, 0, 97, 104, 105, 0];
    let binary = Document::try_from(data).unwrap();
    let binary = binary.get_binary("a").unwrap();
    assert_eq!(
        Binary {
            binary_type: BinaryType::BinaryGeneric,
            data: [97, 104, 105].to_vec()
        },
        binary
    );
}
