# ofx-rs

A Rust library for parsing Open Financial Exchange (OFX) documents into strongly-typed structures.

`ofx-rs` handles both OFX 1.x (SGML) and OFX 2.x (XML) formats through a single entry point, producing precise domain types that prevent common financial data bugs at compile time. The parser is pure -- no I/O, no unsafe code, no runtime surprises.

## Quick Start

Add the dependency to your project:

```sh
cargo add ofx-rs
```

Parse an OFX document with a single function call:

```rust
use ofx_rs::parse;

// You provide the file content; the library does no I/O itself.
let ofx_content = std::fs::read_to_string("statement.ofx").unwrap();
let doc = parse(&ofx_content).unwrap();

// Check signon status
assert!(doc.signon().status().is_success());

// Access the first bank statement
let banking = doc.banking().expect("no banking data");
let stmt = banking.statement_responses()[0]
    .response()
    .expect("no statement response");

println!("Account: {}", stmt.bank_account().account_id().as_str());
println!("Currency: {}", stmt.currency_default().as_str());

if let Some(balance) = stmt.ledger_balance() {
    println!("Balance: {}", balance.amount());
}
```

## Supported Formats

The library parses both major OFX format families transparently:

- **OFX 2.x (XML)** -- Well-formed XML with a processing instruction header (`<?OFX ... ?>`). Parsed directly by quick-xml.
- **OFX 1.x (SGML)** -- Uses SGML where closing tags are optional. The library normalizes these documents to well-formed XML automatically before parsing, using knowledge of the OFX tag hierarchy to insert missing close tags.

Format detection is automatic based on the header. You always call `parse()` the same way regardless of version.

## Examples

### Iterating Over Transactions

```rust
use ofx_rs::parse;
use ofx_rs::types::TransactionType;

let doc = parse(&ofx_content)?;

let banking = doc.banking().expect("no banking data");
let stmt = banking.statement_responses()[0]
    .response()
    .expect("no statement response");

// Account information
let acct = stmt.bank_account();
println!("Bank: {}", acct.bank_id().as_str());
println!("Account: {}", acct.account_id().as_str());
println!("Type: {}", acct.account_type());

// Iterate and filter transactions
if let Some(txn_list) = stmt.transaction_list() {
    for txn in txn_list.transactions() {
        println!(
            "{} {:>10} {}",
            txn.date_posted(),
            txn.amount(),
            txn.name().unwrap_or("(unnamed)")
        );
    }

    let debits: Vec<_> = txn_list
        .transactions()
        .iter()
        .filter(|t| t.transaction_type() == TransactionType::Debit)
        .collect();
    println!("Found {} debits", debits.len());
}
```

### Parsing a Credit Card Statement

```rust
use ofx_rs::parse;

let doc = parse(&ofx_content)?;

if let Some(cc) = doc.credit_card() {
    let stmt = cc.statement_responses()[0]
        .response()
        .expect("no cc statement");

    println!("Card: {}", stmt.credit_card_account().account_id().as_str());

    if let Some(balance) = stmt.ledger_balance() {
        println!("Balance: {}", balance.amount());
    }
}
```

### Handling OFX 1.x (SGML) Files

OFX 1.x files use SGML syntax where closing tags are optional:

```text
OFXHEADER:100
DATA:OFXSGML
VERSION:102
SECURITY:NONE
ENCODING:USASCII
CHARSET:1252
COMPRESSION:NONE
OLDFILEUID:NONE
NEWFILEUID:NONE

<OFX>
<SIGNONMSGSRSV1>
<SONRS>
<STATUS>
<CODE>0
<SEVERITY>INFO
<DTSERVER>20230115
<LANGUAGE>ENG
```

The library normalizes this automatically. The API is identical regardless of format version:

```rust
use ofx_rs::parse;

// Works the same for both OFX 1.x and 2.x
let doc = parse(&sgml_content)?;
println!("OFX version: {}", doc.header().version());
```

## API Overview

`parse()` returns an `OfxDocument` that mirrors the OFX document tree:

```text
OfxDocument
  |-- header: OfxHeader          (version, security level, file UIDs)
  |-- signon: SignonResponse     (status, server datetime, language, FI info)
  |-- banking: BankingMessageSet (optional)
  |     |-- statement_responses: Vec<TransactionWrapper<StatementResponse>>
  |           |-- status: Status
  |           |-- response: StatementResponse
  |                 |-- currency_default: CurrencyCode
  |                 |-- bank_account: BankAccount
  |                 |-- transaction_list: TransactionList (optional)
  |                 |     |-- transactions: Vec<StatementTransaction>
  |                 |-- ledger_balance: LedgerBalance (optional)
  |                 |-- available_balance: AvailableBalance (optional)
  |                 |-- balance_list: Vec<Balance>
  |-- credit_card: CreditCardMessageSet (optional)
        |-- statement_responses: Vec<TransactionWrapper<CcStatementResponse>>
              |-- (same structure as banking, with CreditCardAccount)
```

Every field is accessed through methods on the returned structs. Optional fields return `Option<&T>`, and collections return slices.

## Type System

Financial data demands precision. `ofx-rs` uses domain-specific types rather than raw strings and floats:

| Type | Wraps | Purpose |
|------|-------|---------|
| `OfxAmount` | `rust_decimal::Decimal` | Exact financial arithmetic -- no floating-point rounding |
| `OfxDateTime` | `time::OffsetDateTime` | OFX datetime format with timezone support |
| `CurrencyCode` | Validated `String` | ISO 4217 currency codes (USD, EUR, BRL) |
| `TransactionType` | Enum (18 variants) | CREDIT, DEBIT, CHECK, ATM, POS, XFER, and more |
| `AccountType` | Enum | CHECKING, SAVINGS, MONEYMRKT, CREDITLINE |
| `BankId`, `AccountId`, `FitId` | Validated newtypes | Length-validated identifiers that reject empty strings |
| `CheckNumber` | Validated newtype | Check number with spec-compliant length constraints |

`OfxAmount` supports arithmetic operations (`Add`, `Sub`, `Neg`) and convenience methods like `is_negative()` and `is_zero()`:

```rust
let amount: ofx_rs::types::OfxAmount = "-50.00".parse().unwrap();
assert!(amount.is_negative());

// Decimal precision preserved
assert_eq!(amount.as_decimal(), rust_decimal::Decimal::new(-5000, 2));
```

`OfxDateTime` parses the OFX datetime format with right-truncation and timezone offsets:

```rust
// Date only
let dt: ofx_rs::types::OfxDateTime = "20230115".parse().unwrap();

// Full datetime with timezone
let dt: ofx_rs::types::OfxDateTime = "20230115120000[-5:EST]".parse().unwrap();
```

## Error Handling

All errors are structured and non-panicking. The top-level `OfxError` enum distinguishes three failure categories:

- **`OfxError::Header`** -- The OFX header is missing, malformed, or contains an unrecognized version or security level.
- **`OfxError::Xml`** -- The XML body is malformed, a required element is missing, or an element contains invalid content.
- **`OfxError::Aggregate`** -- A required field within an OFX aggregate (like a transaction missing its FITID) is absent.

`OfxError` and its inner types are marked `#[non_exhaustive]`, so match statements require a wildcard arm. Each variant provides specific context about what went wrong and where:

```rust
use ofx_rs::{parse, OfxError};

match parse(input) {
    Ok(doc) => { /* use document */ }
    Err(OfxError::Header(e)) => eprintln!("Bad header: {e}"),
    Err(OfxError::Xml(e)) => eprintln!("XML error: {e}"),
    Err(OfxError::Aggregate(e)) => eprintln!("Aggregate error: {e}"),
    Err(e) => eprintln!("Other error: {e}"),
}
```

## Dependencies

`ofx-rs` depends on three crates, chosen for correctness over convenience:

- **[quick-xml](https://crates.io/crates/quick-xml)** -- Fast, zero-copy XML parsing
- **[rust_decimal](https://crates.io/crates/rust_decimal)** -- Exact decimal arithmetic for financial amounts
- **[time](https://crates.io/crates/time)** -- Date and time handling with timezone support

No runtime, no async, no macros, no build scripts.

## Minimum Supported Rust Version

This crate uses `edition = "2024"` and requires **Rust 1.85** or later.

## License

MIT
