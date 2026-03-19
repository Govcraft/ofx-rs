#![forbid(unsafe_code)]
#![deny(clippy::unwrap_used, clippy::expect_used)]

//! `ofx-rs` -- A Rust library for parsing Open Financial Exchange (OFX) documents.
//!
//! This library provides a pure functional parser for OFX 1.x and 2.x documents.
//! It takes string input and produces typed data structures with no I/O.
//!
//! # Usage
//!
//! ```
//! use ofx_rs::parse;
//!
//! let ofx_content = r#"<?OFX OFXHEADER="200" VERSION="220" SECURITY="NONE" OLDFILEUID="NONE" NEWFILEUID="NONE"?>
//! <OFX>
//! <SIGNONMSGSRSV1>
//! <SONRS>
//! <STATUS><CODE>0</CODE><SEVERITY>INFO</SEVERITY></STATUS>
//! <DTSERVER>20230115120000</DTSERVER>
//! <LANGUAGE>ENG</LANGUAGE>
//! </SONRS>
//! </SIGNONMSGSRSV1>
//! </OFX>"#;
//!
//! let doc = parse(ofx_content).unwrap();
//! assert!(doc.signon().status().is_success());
//! ```

pub mod aggregates;
pub mod document;
pub mod error;
pub mod header;
mod parser;
mod sgml;
pub mod types;
pub mod xml;

pub use document::OfxDocument;
pub use error::OfxError;
pub use parser::parse;
