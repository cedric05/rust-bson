# bson

Encoding and decoding support for BSON in Rust


## Overview of the BSON Format

BSON, short for Binary JSON, is a binary-encoded serialization of JSON-like documents.
Like JSON, BSON supports the embedding of documents and arrays within other documents
and arrays. BSON also contains extensions that allow representation of data types that
are not part of the JSON spec. For example, BSON has a datetime type and a binary data type.

```text
// JSON equivalent
{"hello": "world"}

// BSON encoding
\x16\x00\x00\x00                   // total document size
\x02                               // 0x02 = type String
hello\x00                          // field name
\x06\x00\x00\x00world\x00          // field value
\x00                               // 0x00 = type EOO ('end of object')
```

BSON is the primary data representation for [MongoDB](https://www.mongodb.com/), and this crate is used in the
[`mongodb`](https://docs.rs/mongodb/latest/mongodb/) driver crate in its API and implementation.

For more information about BSON itself, see [bsonspec.org](http://bsonspec.org).

