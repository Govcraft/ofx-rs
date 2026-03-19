//! OFX header parsing -- handles both OFX 2.x XML processing instruction
//! headers and legacy OFX 1.x SGML-style headers.

pub mod error;
pub mod ofx_header;
pub mod ofx_version;
pub mod security_level;

pub use error::HeaderError;
pub use ofx_header::{OfxHeader, parse_header};
pub use ofx_version::{InvalidOfxVersion, OfxVersion};
pub use security_level::{InvalidSecurityLevel, SecurityLevel};
