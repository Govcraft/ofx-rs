use quick_xml::events::Event;

use crate::aggregates::{
    AvailableBalance, Balance, BankAccount, CcStatementResponse, CreditCardAccount, CurrencyInfo,
    LedgerBalance, Payee, StatementResponse, StatementTransactionBuilder, Status,
    TransactionList, TransactionWrapper,
};
use crate::document::{
    BankingMessageSet, CreditCardMessageSet, OfxDocument, SignonResponse,
};
use crate::error::OfxError;
use crate::header::{self, OfxHeader};
use crate::xml::{OfxReader, XmlError};

/// Parses a complete OFX document from a string.
///
/// This is the primary entry point for the library. It accepts the full
/// content of an OFX file (header + XML body) and returns a structured document.
///
/// # Errors
///
/// Returns `OfxError` if the header is missing or malformed, the XML is invalid,
/// or required OFX elements are missing.
pub fn parse(input: &str) -> Result<OfxDocument, OfxError> {
    let (ofx_header, xml_body) = header::parse_header(input)?;
    let xml_body = xml_body.trim();
    parse_ofx_body(xml_body, ofx_header)
}

// ---------------------------------------------------------------------------
// Tag name helpers -- compare against byte slices to avoid allocations
// ---------------------------------------------------------------------------

fn tag_name(e: &quick_xml::events::BytesStart<'_>) -> String {
    String::from_utf8_lossy(e.name().as_ref()).into_owned()
}

fn end_tag_name(e: &quick_xml::events::BytesEnd<'_>) -> String {
    String::from_utf8_lossy(e.name().as_ref()).into_owned()
}

/// Read the text content of the current element (after Start has been consumed).
fn read_text(reader: &mut OfxReader<'_>, tag: &str) -> Result<String, XmlError> {
    reader.read_text(tag)
}

/// Read a required text element by name inside an aggregate being walked.
/// This is for use when we've already received a Start event for `tag`.
fn parse_text_as<T: core::str::FromStr>(
    text: &str,
    element: &str,
) -> Result<T, XmlError> {
    text.parse::<T>().map_err(|_| XmlError::InvalidContent {
        element: element.to_owned(),
        value: text.to_owned(),
        reason: format!("failed to parse as {}", core::any::type_name::<T>()),
    })
}

// ---------------------------------------------------------------------------
// Top-level document parsing
// ---------------------------------------------------------------------------

fn parse_ofx_body(xml: &str, header: OfxHeader) -> Result<OfxDocument, OfxError> {
    let mut reader = OfxReader::new(xml);
    let mut signon: Option<SignonResponse> = None;
    let mut banking: Option<BankingMessageSet> = None;
    let mut credit_card: Option<CreditCardMessageSet> = None;

    // Find and enter the <OFX> root element
    loop {
        match reader.next_event()? {
            Event::Start(e) if tag_name(&e) == "OFX" => break,
            Event::Eof => {
                return Err(XmlError::MissingElement {
                    parent: "document".to_owned(),
                    element: "OFX".to_owned(),
                }
                .into());
            }
            _ => {}
        }
    }

    // Parse children of <OFX>
    loop {
        match reader.next_event()? {
            Event::Start(e) => {
                let name = tag_name(&e);
                match name.as_str() {
                    "SIGNONMSGSRSV1" => {
                        signon = Some(parse_signon_message_set(&mut reader)?);
                    }
                    "BANKMSGSRSV1" => {
                        banking = Some(parse_banking_message_set(&mut reader)?);
                    }
                    "CREDITCARDMSGSRSV1" => {
                        credit_card = Some(parse_cc_message_set(&mut reader)?);
                    }
                    _ => {
                        reader.skip_element(&name)?;
                    }
                }
            }
            Event::End(e) if end_tag_name(&e) == "OFX" => break,
            Event::Eof => break,
            _ => {}
        }
    }

    let signon = signon.ok_or_else(|| XmlError::MissingElement {
        parent: "OFX".to_owned(),
        element: "SIGNONMSGSRSV1".to_owned(),
    })?;

    let mut doc = OfxDocument::new(header, signon);
    if let Some(b) = banking {
        doc = doc.with_banking(b);
    }
    if let Some(cc) = credit_card {
        doc = doc.with_credit_card(cc);
    }

    Ok(doc)
}

// ---------------------------------------------------------------------------
// Signon message set
// ---------------------------------------------------------------------------

fn parse_signon_message_set(reader: &mut OfxReader<'_>) -> Result<SignonResponse, OfxError> {
    loop {
        match reader.next_event()? {
            Event::Start(e) => {
                let name = tag_name(&e);
                if name == "SONRS" {
                    let sonrs = parse_sonrs(reader)?;
                    reader.skip_to_end("SIGNONMSGSRSV1")?;
                    return Ok(sonrs);
                }
                reader.skip_element(&name)?;
            }
            Event::End(e) if end_tag_name(&e) == "SIGNONMSGSRSV1" => {
                return Err(XmlError::MissingElement {
                    parent: "SIGNONMSGSRSV1".to_owned(),
                    element: "SONRS".to_owned(),
                }
                .into());
            }
            Event::Eof => {
                return Err(XmlError::MalformedXml {
                    message: "unexpected EOF in SIGNONMSGSRSV1".to_owned(),
                }
                .into());
            }
            _ => {}
        }
    }
}

fn parse_sonrs(reader: &mut OfxReader<'_>) -> Result<SignonResponse, OfxError> {
    let mut status: Option<Status> = None;
    let mut dtserver: Option<crate::types::OfxDateTime> = None;
    let mut language: Option<String> = None;
    let mut fi_org: Option<String> = None;
    let mut fi_id: Option<String> = None;
    let mut session_cookie: Option<String> = None;
    let mut access_key: Option<String> = None;

    loop {
        match reader.next_event()? {
            Event::Start(e) => {
                let name = tag_name(&e);
                match name.as_str() {
                    "STATUS" => status = Some(parse_status(reader)?),
                    "DTSERVER" => {
                        let text = read_text(reader, "DTSERVER")?;
                        dtserver = Some(parse_text_as(&text, "DTSERVER")?);
                    }
                    "LANGUAGE" => language = Some(read_text(reader, "LANGUAGE")?),
                    "FI" => {
                        let (org, id) = parse_fi(reader)?;
                        fi_org = org;
                        fi_id = id;
                    }
                    "SESSCOOKIE" => session_cookie = Some(read_text(reader, "SESSCOOKIE")?),
                    "ACCESSKEY" => access_key = Some(read_text(reader, "ACCESSKEY")?),
                    _ => reader.skip_element(&name)?,
                }
            }
            Event::End(e) if end_tag_name(&e) == "SONRS" => break,
            Event::Eof => {
                return Err(XmlError::MalformedXml {
                    message: "unexpected EOF in SONRS".to_owned(),
                }
                .into());
            }
            _ => {}
        }
    }

    let status = status.ok_or_else(|| XmlError::MissingElement {
        parent: "SONRS".to_owned(),
        element: "STATUS".to_owned(),
    })?;
    let dtserver = dtserver.ok_or_else(|| XmlError::MissingElement {
        parent: "SONRS".to_owned(),
        element: "DTSERVER".to_owned(),
    })?;
    let language = language.unwrap_or_else(|| "ENG".to_owned());

    let mut sonrs = SignonResponse::new(status, dtserver, language);
    if let Some(org) = fi_org {
        sonrs = sonrs.with_fi_org(org);
    }
    if let Some(id) = fi_id {
        sonrs = sonrs.with_fi_id(id);
    }
    if let Some(cookie) = session_cookie {
        sonrs = sonrs.with_session_cookie(cookie);
    }
    if let Some(key) = access_key {
        sonrs = sonrs.with_access_key(key);
    }
    Ok(sonrs)
}

fn parse_fi(reader: &mut OfxReader<'_>) -> Result<(Option<String>, Option<String>), OfxError> {
    let mut org: Option<String> = None;
    let mut fid: Option<String> = None;

    loop {
        match reader.next_event()? {
            Event::Start(e) => {
                let name = tag_name(&e);
                match name.as_str() {
                    "ORG" => org = Some(read_text(reader, "ORG")?),
                    "FID" => fid = Some(read_text(reader, "FID")?),
                    _ => reader.skip_element(&name)?,
                }
            }
            Event::End(e) if end_tag_name(&e) == "FI" => break,
            Event::Eof => {
                return Err(XmlError::MalformedXml {
                    message: "unexpected EOF in FI".to_owned(),
                }
                .into());
            }
            _ => {}
        }
    }
    Ok((org, fid))
}

// ---------------------------------------------------------------------------
// STATUS aggregate (used everywhere)
// ---------------------------------------------------------------------------

fn parse_status(reader: &mut OfxReader<'_>) -> Result<Status, OfxError> {
    let mut code: Option<u32> = None;
    let mut severity: Option<crate::types::Severity> = None;
    let mut message: Option<String> = None;

    loop {
        match reader.next_event()? {
            Event::Start(e) => {
                let name = tag_name(&e);
                match name.as_str() {
                    "CODE" => {
                        let text = read_text(reader, "CODE")?;
                        code = Some(parse_text_as(&text, "CODE")?);
                    }
                    "SEVERITY" => {
                        let text = read_text(reader, "SEVERITY")?;
                        severity = Some(parse_text_as(&text, "SEVERITY")?);
                    }
                    "MESSAGE" => message = Some(read_text(reader, "MESSAGE")?),
                    _ => reader.skip_element(&name)?,
                }
            }
            Event::End(e) if end_tag_name(&e) == "STATUS" => break,
            Event::Eof => {
                return Err(XmlError::MalformedXml {
                    message: "unexpected EOF in STATUS".to_owned(),
                }
                .into());
            }
            _ => {}
        }
    }

    let code = code.ok_or_else(|| XmlError::MissingElement {
        parent: "STATUS".to_owned(),
        element: "CODE".to_owned(),
    })?;
    let severity = severity.ok_or_else(|| XmlError::MissingElement {
        parent: "STATUS".to_owned(),
        element: "SEVERITY".to_owned(),
    })?;

    Ok(Status::new(code, severity, message))
}

// ---------------------------------------------------------------------------
// Banking message set
// ---------------------------------------------------------------------------

fn parse_banking_message_set(reader: &mut OfxReader<'_>) -> Result<BankingMessageSet, OfxError> {
    let mut responses = Vec::new();

    loop {
        match reader.next_event()? {
            Event::Start(e) => {
                let name = tag_name(&e);
                if name == "STMTTRNRS" {
                    responses.push(parse_stmttrnrs(reader)?);
                } else {
                    reader.skip_element(&name)?;
                }
            }
            Event::End(e) if end_tag_name(&e) == "BANKMSGSRSV1" => break,
            Event::Eof => {
                return Err(XmlError::MalformedXml {
                    message: "unexpected EOF in BANKMSGSRSV1".to_owned(),
                }
                .into());
            }
            _ => {}
        }
    }

    Ok(BankingMessageSet::new(responses))
}

fn parse_stmttrnrs(
    reader: &mut OfxReader<'_>,
) -> Result<TransactionWrapper<StatementResponse>, OfxError> {
    let mut trnuid: Option<String> = None;
    let mut status: Option<Status> = None;
    let mut client_cookie: Option<String> = None;
    let mut stmtrs: Option<StatementResponse> = None;

    loop {
        match reader.next_event()? {
            Event::Start(e) => {
                let name = tag_name(&e);
                match name.as_str() {
                    "TRNUID" => trnuid = Some(read_text(reader, "TRNUID")?),
                    "STATUS" => status = Some(parse_status(reader)?),
                    "CLTCOOKIE" => client_cookie = Some(read_text(reader, "CLTCOOKIE")?),
                    "STMTRS" => stmtrs = Some(parse_stmtrs(reader)?),
                    _ => reader.skip_element(&name)?,
                }
            }
            Event::End(e) if end_tag_name(&e) == "STMTTRNRS" => break,
            Event::Eof => {
                return Err(XmlError::MalformedXml {
                    message: "unexpected EOF in STMTTRNRS".to_owned(),
                }
                .into());
            }
            _ => {}
        }
    }

    let trnuid = trnuid.ok_or_else(|| XmlError::MissingElement {
        parent: "STMTTRNRS".to_owned(),
        element: "TRNUID".to_owned(),
    })?;
    let status = status.ok_or_else(|| XmlError::MissingElement {
        parent: "STMTTRNRS".to_owned(),
        element: "STATUS".to_owned(),
    })?;

    let mut wrapper = TransactionWrapper::new(trnuid, status, stmtrs);
    if let Some(cookie) = client_cookie {
        wrapper = wrapper.with_client_cookie(cookie);
    }
    Ok(wrapper)
}

fn parse_stmtrs(reader: &mut OfxReader<'_>) -> Result<StatementResponse, OfxError> {
    let mut curdef: Option<crate::types::CurrencyCode> = None;
    let mut bank_acct: Option<BankAccount> = None;
    let mut transaction_list: Option<TransactionList> = None;
    let mut ledger_balance: Option<LedgerBalance> = None;
    let mut available_balance: Option<AvailableBalance> = None;
    let mut balance_list: Vec<Balance> = Vec::new();
    let mut marketing_info: Option<String> = None;

    loop {
        match reader.next_event()? {
            Event::Start(e) => {
                let name = tag_name(&e);
                match name.as_str() {
                    "CURDEF" => {
                        let text = read_text(reader, "CURDEF")?;
                        curdef = Some(parse_text_as(&text, "CURDEF")?);
                    }
                    "BANKACCTFROM" => bank_acct = Some(parse_bank_account(reader, "BANKACCTFROM")?),
                    "BANKTRANLIST" => transaction_list = Some(parse_bank_transaction_list(reader)?),
                    "LEDGERBAL" => ledger_balance = Some(parse_ledger_balance(reader)?),
                    "AVAILBAL" => available_balance = Some(parse_available_balance(reader)?),
                    "BALLIST" => balance_list = parse_balance_list(reader)?,
                    "MKTGINFO" => marketing_info = Some(read_text(reader, "MKTGINFO")?),
                    _ => reader.skip_element(&name)?,
                }
            }
            Event::End(e) if end_tag_name(&e) == "STMTRS" => break,
            Event::Eof => {
                return Err(XmlError::MalformedXml {
                    message: "unexpected EOF in STMTRS".to_owned(),
                }
                .into());
            }
            _ => {}
        }
    }

    let curdef = curdef.ok_or_else(|| XmlError::MissingElement {
        parent: "STMTRS".to_owned(),
        element: "CURDEF".to_owned(),
    })?;
    let bank_acct = bank_acct.ok_or_else(|| XmlError::MissingElement {
        parent: "STMTRS".to_owned(),
        element: "BANKACCTFROM".to_owned(),
    })?;

    let mut stmt = StatementResponse::new(curdef, bank_acct);
    if let Some(tl) = transaction_list {
        stmt = stmt.with_transaction_list(tl);
    }
    if let Some(lb) = ledger_balance {
        stmt = stmt.with_ledger_balance(lb);
    }
    if let Some(ab) = available_balance {
        stmt = stmt.with_available_balance(ab);
    }
    if !balance_list.is_empty() {
        stmt = stmt.with_balance_list(balance_list);
    }
    if let Some(info) = marketing_info {
        stmt = stmt.with_marketing_info(info);
    }

    Ok(stmt)
}

// ---------------------------------------------------------------------------
// Bank transaction list (BANKTRANLIST) -- contains multiple STMTTRN
// ---------------------------------------------------------------------------

fn parse_bank_transaction_list(
    reader: &mut OfxReader<'_>,
) -> Result<TransactionList, OfxError> {
    let mut dtstart: Option<crate::types::OfxDateTime> = None;
    let mut dtend: Option<crate::types::OfxDateTime> = None;
    let mut transactions = Vec::new();

    loop {
        match reader.next_event()? {
            Event::Start(e) => {
                let name = tag_name(&e);
                match name.as_str() {
                    "DTSTART" => {
                        let text = read_text(reader, "DTSTART")?;
                        dtstart = Some(parse_text_as(&text, "DTSTART")?);
                    }
                    "DTEND" => {
                        let text = read_text(reader, "DTEND")?;
                        dtend = Some(parse_text_as(&text, "DTEND")?);
                    }
                    "STMTTRN" => {
                        transactions.push(parse_statement_transaction(reader)?);
                    }
                    _ => reader.skip_element(&name)?,
                }
            }
            Event::End(e) if end_tag_name(&e) == "BANKTRANLIST" => break,
            Event::Eof => {
                return Err(XmlError::MalformedXml {
                    message: "unexpected EOF in BANKTRANLIST".to_owned(),
                }
                .into());
            }
            _ => {}
        }
    }

    let dtstart = dtstart.ok_or_else(|| XmlError::MissingElement {
        parent: "BANKTRANLIST".to_owned(),
        element: "DTSTART".to_owned(),
    })?;
    let dtend = dtend.ok_or_else(|| XmlError::MissingElement {
        parent: "BANKTRANLIST".to_owned(),
        element: "DTEND".to_owned(),
    })?;

    Ok(TransactionList::new(dtstart, dtend, transactions))
}

// ---------------------------------------------------------------------------
// Statement transaction (STMTTRN) -- full hierarchical parsing
// ---------------------------------------------------------------------------

fn apply_stmttrn_field(
    builder: StatementTransactionBuilder,
    reader: &mut OfxReader<'_>,
    name: &str,
) -> Result<StatementTransactionBuilder, OfxError> {
    Ok(match name {
        "TRNTYPE" => builder.transaction_type(parse_text_as(&read_text(reader, name)?, name)?),
        "DTPOSTED" => builder.date_posted(parse_text_as(&read_text(reader, name)?, name)?),
        "DTUSER" => builder.date_user(parse_text_as(&read_text(reader, name)?, name)?),
        "DTAVAIL" => builder.date_available(parse_text_as(&read_text(reader, name)?, name)?),
        "TRNAMT" => builder.amount(parse_text_as(&read_text(reader, name)?, name)?),
        "FITID" => builder.fit_id(parse_text_as(&read_text(reader, name)?, name)?),
        "CORRECTFITID" => builder.correction_id(parse_text_as(&read_text(reader, name)?, name)?),
        "CORRECTACTION" => builder.correction_action(parse_text_as(&read_text(reader, name)?, name)?),
        "SRVRTID" => builder.server_transaction_id(parse_text_as(&read_text(reader, name)?, name)?),
        "CHECKNUM" => builder.check_number(parse_text_as(&read_text(reader, name)?, name)?),
        "REFNUM" => builder.reference_number(read_text(reader, name)?),
        "SIC" => builder.sic(parse_text_as(&read_text(reader, name)?, name)?),
        "PAYEEID" => builder.payee_id(read_text(reader, name)?),
        "NAME" => builder.name(read_text(reader, name)?),
        "PAYEE" => builder.payee(parse_payee(reader)?),
        "BANKACCTTO" => builder.bank_account_to(parse_bank_account(reader, name)?),
        "CCACCTTO" => builder.cc_account_to(parse_cc_account(reader, name)?),
        "MEMO" => builder.memo(read_text(reader, name)?),
        "CURRENCY" | "ORIGCURRENCY" => builder.currency(parse_currency(reader, name)?),
        "INV401KSOURCE" => builder.inv401k_source(parse_text_as(&read_text(reader, name)?, name)?),
        _ => {
            reader.skip_element(name)?;
            builder
        }
    })
}

fn parse_statement_transaction(
    reader: &mut OfxReader<'_>,
) -> Result<crate::aggregates::StatementTransaction, OfxError> {
    let mut builder = StatementTransactionBuilder::new();

    loop {
        match reader.next_event()? {
            Event::Start(e) => {
                let name = tag_name(&e);
                builder = apply_stmttrn_field(builder, reader, &name)?;
            }
            Event::End(e) if end_tag_name(&e) == "STMTTRN" => break,
            Event::Eof => {
                return Err(XmlError::MalformedXml {
                    message: "unexpected EOF in STMTTRN".to_owned(),
                }
                .into());
            }
            _ => {}
        }
    }

    builder.build().map_err(|msg| {
        OfxError::Xml(XmlError::MalformedXml {
            message: format!("failed to build STMTTRN: {msg}"),
        })
    })
}

// ---------------------------------------------------------------------------
// Balance aggregates
// ---------------------------------------------------------------------------

fn parse_ledger_balance(reader: &mut OfxReader<'_>) -> Result<LedgerBalance, OfxError> {
    let mut balamt: Option<crate::types::OfxAmount> = None;
    let mut dtasof: Option<crate::types::OfxDateTime> = None;

    loop {
        match reader.next_event()? {
            Event::Start(e) => {
                let name = tag_name(&e);
                match name.as_str() {
                    "BALAMT" => {
                        let text = read_text(reader, "BALAMT")?;
                        balamt = Some(parse_text_as(&text, "BALAMT")?);
                    }
                    "DTASOF" => {
                        let text = read_text(reader, "DTASOF")?;
                        dtasof = Some(parse_text_as(&text, "DTASOF")?);
                    }
                    _ => reader.skip_element(&name)?,
                }
            }
            Event::End(e) if end_tag_name(&e) == "LEDGERBAL" => break,
            Event::Eof => {
                return Err(XmlError::MalformedXml {
                    message: "unexpected EOF in LEDGERBAL".to_owned(),
                }
                .into());
            }
            _ => {}
        }
    }

    let balamt = balamt.ok_or_else(|| XmlError::MissingElement {
        parent: "LEDGERBAL".to_owned(),
        element: "BALAMT".to_owned(),
    })?;
    let dtasof = dtasof.ok_or_else(|| XmlError::MissingElement {
        parent: "LEDGERBAL".to_owned(),
        element: "DTASOF".to_owned(),
    })?;

    Ok(LedgerBalance::new(balamt, dtasof))
}

fn parse_available_balance(reader: &mut OfxReader<'_>) -> Result<AvailableBalance, OfxError> {
    let mut balamt: Option<crate::types::OfxAmount> = None;
    let mut dtasof: Option<crate::types::OfxDateTime> = None;

    loop {
        match reader.next_event()? {
            Event::Start(e) => {
                let name = tag_name(&e);
                match name.as_str() {
                    "BALAMT" => {
                        let text = read_text(reader, "BALAMT")?;
                        balamt = Some(parse_text_as(&text, "BALAMT")?);
                    }
                    "DTASOF" => {
                        let text = read_text(reader, "DTASOF")?;
                        dtasof = Some(parse_text_as(&text, "DTASOF")?);
                    }
                    _ => reader.skip_element(&name)?,
                }
            }
            Event::End(e) if end_tag_name(&e) == "AVAILBAL" => break,
            Event::Eof => {
                return Err(XmlError::MalformedXml {
                    message: "unexpected EOF in AVAILBAL".to_owned(),
                }
                .into());
            }
            _ => {}
        }
    }

    let balamt = balamt.ok_or_else(|| XmlError::MissingElement {
        parent: "AVAILBAL".to_owned(),
        element: "BALAMT".to_owned(),
    })?;
    let dtasof = dtasof.ok_or_else(|| XmlError::MissingElement {
        parent: "AVAILBAL".to_owned(),
        element: "DTASOF".to_owned(),
    })?;

    Ok(AvailableBalance::new(balamt, dtasof))
}

fn parse_balance_list(reader: &mut OfxReader<'_>) -> Result<Vec<Balance>, OfxError> {
    let mut balances = Vec::new();

    loop {
        match reader.next_event()? {
            Event::Start(e) => {
                let name = tag_name(&e);
                if name == "BAL" {
                    balances.push(parse_bal(reader)?);
                } else {
                    reader.skip_element(&name)?;
                }
            }
            Event::End(e) if end_tag_name(&e) == "BALLIST" => break,
            Event::Eof => {
                return Err(XmlError::MalformedXml {
                    message: "unexpected EOF in BALLIST".to_owned(),
                }
                .into());
            }
            _ => {}
        }
    }

    Ok(balances)
}

fn parse_bal(reader: &mut OfxReader<'_>) -> Result<Balance, OfxError> {
    let mut name: Option<String> = None;
    let mut desc: Option<String> = None;
    let mut bal_type: Option<crate::types::BalanceType> = None;
    let mut value: Option<crate::types::OfxAmount> = None;
    let mut dtasof: Option<crate::types::OfxDateTime> = None;
    let mut currency: Option<crate::types::CurrencyCode> = None;

    loop {
        match reader.next_event()? {
            Event::Start(e) => {
                let tag = tag_name(&e);
                match tag.as_str() {
                    "NAME" => name = Some(read_text(reader, "NAME")?),
                    "DESC" => desc = Some(read_text(reader, "DESC")?),
                    "BALTYPE" => {
                        let text = read_text(reader, "BALTYPE")?;
                        bal_type = Some(parse_text_as(&text, "BALTYPE")?);
                    }
                    "VALUE" => {
                        let text = read_text(reader, "VALUE")?;
                        value = Some(parse_text_as(&text, "VALUE")?);
                    }
                    "DTASOF" => {
                        let text = read_text(reader, "DTASOF")?;
                        dtasof = Some(parse_text_as(&text, "DTASOF")?);
                    }
                    "CURRENCY" => {
                        let text = read_text(reader, "CURRENCY")?;
                        currency = Some(parse_text_as(&text, "CURRENCY")?);
                    }
                    _ => reader.skip_element(&tag)?,
                }
            }
            Event::End(e) if end_tag_name(&e) == "BAL" => break,
            Event::Eof => {
                return Err(XmlError::MalformedXml {
                    message: "unexpected EOF in BAL".to_owned(),
                }
                .into());
            }
            _ => {}
        }
    }

    let name = name.ok_or_else(|| XmlError::MissingElement {
        parent: "BAL".to_owned(),
        element: "NAME".to_owned(),
    })?;
    let desc = desc.ok_or_else(|| XmlError::MissingElement {
        parent: "BAL".to_owned(),
        element: "DESC".to_owned(),
    })?;
    let bal_type = bal_type.ok_or_else(|| XmlError::MissingElement {
        parent: "BAL".to_owned(),
        element: "BALTYPE".to_owned(),
    })?;
    let value = value.ok_or_else(|| XmlError::MissingElement {
        parent: "BAL".to_owned(),
        element: "VALUE".to_owned(),
    })?;

    Ok(Balance::new(name, desc, bal_type, value, dtasof, currency))
}

// ---------------------------------------------------------------------------
// Account aggregates
// ---------------------------------------------------------------------------

fn parse_bank_account(
    reader: &mut OfxReader<'_>,
    closing_tag: &str,
) -> Result<BankAccount, OfxError> {
    let mut bank_id: Option<crate::types::BankId> = None;
    let mut branch_id: Option<crate::types::BranchId> = None;
    let mut acct_id: Option<crate::types::AccountId> = None;
    let mut acct_type: Option<crate::types::AccountType> = None;
    let mut acct_key: Option<String> = None;

    loop {
        match reader.next_event()? {
            Event::Start(e) => {
                let name = tag_name(&e);
                match name.as_str() {
                    "BANKID" => {
                        let text = read_text(reader, "BANKID")?;
                        bank_id = Some(parse_text_as(&text, "BANKID")?);
                    }
                    "BRANCHID" => {
                        let text = read_text(reader, "BRANCHID")?;
                        branch_id = Some(parse_text_as(&text, "BRANCHID")?);
                    }
                    "ACCTID" => {
                        let text = read_text(reader, "ACCTID")?;
                        acct_id = Some(parse_text_as(&text, "ACCTID")?);
                    }
                    "ACCTTYPE" => {
                        let text = read_text(reader, "ACCTTYPE")?;
                        acct_type = Some(parse_text_as(&text, "ACCTTYPE")?);
                    }
                    "ACCTKEY" => acct_key = Some(read_text(reader, "ACCTKEY")?),
                    _ => reader.skip_element(&name)?,
                }
            }
            Event::End(e) if end_tag_name(&e) == closing_tag => break,
            Event::Eof => {
                return Err(XmlError::MalformedXml {
                    message: format!("unexpected EOF in {closing_tag}"),
                }
                .into());
            }
            _ => {}
        }
    }

    let bank_id = bank_id.ok_or_else(|| XmlError::MissingElement {
        parent: closing_tag.to_owned(),
        element: "BANKID".to_owned(),
    })?;
    let acct_id = acct_id.ok_or_else(|| XmlError::MissingElement {
        parent: closing_tag.to_owned(),
        element: "ACCTID".to_owned(),
    })?;
    let acct_type = acct_type.ok_or_else(|| XmlError::MissingElement {
        parent: closing_tag.to_owned(),
        element: "ACCTTYPE".to_owned(),
    })?;

    Ok(BankAccount::new(
        bank_id, branch_id, acct_id, acct_type, acct_key,
    ))
}

fn parse_cc_account(
    reader: &mut OfxReader<'_>,
    closing_tag: &str,
) -> Result<CreditCardAccount, OfxError> {
    let mut acct_id: Option<crate::types::AccountId> = None;
    let mut acct_key: Option<String> = None;

    loop {
        match reader.next_event()? {
            Event::Start(e) => {
                let name = tag_name(&e);
                match name.as_str() {
                    "ACCTID" => {
                        let text = read_text(reader, "ACCTID")?;
                        acct_id = Some(parse_text_as(&text, "ACCTID")?);
                    }
                    "ACCTKEY" => acct_key = Some(read_text(reader, "ACCTKEY")?),
                    _ => reader.skip_element(&name)?,
                }
            }
            Event::End(e) if end_tag_name(&e) == closing_tag => break,
            Event::Eof => {
                return Err(XmlError::MalformedXml {
                    message: format!("unexpected EOF in {closing_tag}"),
                }
                .into());
            }
            _ => {}
        }
    }

    let acct_id = acct_id.ok_or_else(|| XmlError::MissingElement {
        parent: closing_tag.to_owned(),
        element: "ACCTID".to_owned(),
    })?;

    Ok(CreditCardAccount::new(acct_id, acct_key))
}

// ---------------------------------------------------------------------------
// Payee aggregate
// ---------------------------------------------------------------------------

fn parse_payee(reader: &mut OfxReader<'_>) -> Result<Payee, OfxError> {
    let mut name: Option<String> = None;
    let mut addr1: Option<String> = None;
    let mut addr2: Option<String> = None;
    let mut addr3: Option<String> = None;
    let mut city: Option<String> = None;
    let mut state: Option<String> = None;
    let mut postalcode: Option<String> = None;
    let mut country: Option<String> = None;
    let mut phone: Option<String> = None;

    loop {
        match reader.next_event()? {
            Event::Start(e) => {
                let tag = tag_name(&e);
                match tag.as_str() {
                    "NAME" => name = Some(read_text(reader, "NAME")?),
                    "ADDR1" => addr1 = Some(read_text(reader, "ADDR1")?),
                    "ADDR2" => addr2 = Some(read_text(reader, "ADDR2")?),
                    "ADDR3" => addr3 = Some(read_text(reader, "ADDR3")?),
                    "CITY" => city = Some(read_text(reader, "CITY")?),
                    "STATE" => state = Some(read_text(reader, "STATE")?),
                    "POSTALCODE" => postalcode = Some(read_text(reader, "POSTALCODE")?),
                    "COUNTRY" => country = Some(read_text(reader, "COUNTRY")?),
                    "PHONE" => phone = Some(read_text(reader, "PHONE")?),
                    _ => reader.skip_element(&tag)?,
                }
            }
            Event::End(e) if end_tag_name(&e) == "PAYEE" => break,
            Event::Eof => {
                return Err(XmlError::MalformedXml {
                    message: "unexpected EOF in PAYEE".to_owned(),
                }
                .into());
            }
            _ => {}
        }
    }

    let name = name.ok_or_else(|| XmlError::MissingElement {
        parent: "PAYEE".to_owned(),
        element: "NAME".to_owned(),
    })?;
    let addr1 = addr1.ok_or_else(|| XmlError::MissingElement {
        parent: "PAYEE".to_owned(),
        element: "ADDR1".to_owned(),
    })?;
    let city = city.ok_or_else(|| XmlError::MissingElement {
        parent: "PAYEE".to_owned(),
        element: "CITY".to_owned(),
    })?;
    let state = state.ok_or_else(|| XmlError::MissingElement {
        parent: "PAYEE".to_owned(),
        element: "STATE".to_owned(),
    })?;
    let postalcode = postalcode.ok_or_else(|| XmlError::MissingElement {
        parent: "PAYEE".to_owned(),
        element: "POSTALCODE".to_owned(),
    })?;
    let phone = phone.ok_or_else(|| XmlError::MissingElement {
        parent: "PAYEE".to_owned(),
        element: "PHONE".to_owned(),
    })?;

    let mut payee = Payee::new(name, addr1, city, state, postalcode, phone);
    if let Some(a2) = addr2 {
        payee = payee.with_address2(a2);
    }
    if let Some(a3) = addr3 {
        payee = payee.with_address3(a3);
    }
    if let Some(c) = country {
        payee = payee.with_country(c);
    }
    Ok(payee)
}

// ---------------------------------------------------------------------------
// Currency aggregate
// ---------------------------------------------------------------------------

fn parse_currency(reader: &mut OfxReader<'_>, closing_tag: &str) -> Result<CurrencyInfo, OfxError> {
    let mut currate: Option<crate::types::OfxAmount> = None;
    let mut cursym: Option<crate::types::CurrencyCode> = None;

    loop {
        match reader.next_event()? {
            Event::Start(e) => {
                let name = tag_name(&e);
                match name.as_str() {
                    "CURRATE" => {
                        let text = read_text(reader, "CURRATE")?;
                        currate = Some(parse_text_as(&text, "CURRATE")?);
                    }
                    "CURSYM" => {
                        let text = read_text(reader, "CURSYM")?;
                        cursym = Some(parse_text_as(&text, "CURSYM")?);
                    }
                    _ => reader.skip_element(&name)?,
                }
            }
            Event::End(e) if end_tag_name(&e) == closing_tag => break,
            Event::Eof => {
                return Err(XmlError::MalformedXml {
                    message: format!("unexpected EOF in {closing_tag}"),
                }
                .into());
            }
            _ => {}
        }
    }

    let currate = currate.ok_or_else(|| XmlError::MissingElement {
        parent: closing_tag.to_owned(),
        element: "CURRATE".to_owned(),
    })?;
    let cursym = cursym.ok_or_else(|| XmlError::MissingElement {
        parent: closing_tag.to_owned(),
        element: "CURSYM".to_owned(),
    })?;

    Ok(match closing_tag {
        "ORIGCURRENCY" => CurrencyInfo::OrigCurrency {
            code: cursym,
            rate: currate,
        },
        _ => CurrencyInfo::Currency {
            code: cursym,
            rate: currate,
        },
    })
}

// ---------------------------------------------------------------------------
// Credit card message set
// ---------------------------------------------------------------------------

fn parse_cc_message_set(reader: &mut OfxReader<'_>) -> Result<CreditCardMessageSet, OfxError> {
    let mut responses = Vec::new();

    loop {
        match reader.next_event()? {
            Event::Start(e) => {
                let name = tag_name(&e);
                if name == "CCSTMTTRNRS" {
                    responses.push(parse_ccstmttrnrs(reader)?);
                } else {
                    reader.skip_element(&name)?;
                }
            }
            Event::End(e) if end_tag_name(&e) == "CREDITCARDMSGSRSV1" => break,
            Event::Eof => {
                return Err(XmlError::MalformedXml {
                    message: "unexpected EOF in CREDITCARDMSGSRSV1".to_owned(),
                }
                .into());
            }
            _ => {}
        }
    }

    Ok(CreditCardMessageSet::new(responses))
}

fn parse_ccstmttrnrs(
    reader: &mut OfxReader<'_>,
) -> Result<TransactionWrapper<CcStatementResponse>, OfxError> {
    let mut trnuid: Option<String> = None;
    let mut status: Option<Status> = None;
    let mut client_cookie: Option<String> = None;
    let mut ccstmtrs: Option<CcStatementResponse> = None;

    loop {
        match reader.next_event()? {
            Event::Start(e) => {
                let name = tag_name(&e);
                match name.as_str() {
                    "TRNUID" => trnuid = Some(read_text(reader, "TRNUID")?),
                    "STATUS" => status = Some(parse_status(reader)?),
                    "CLTCOOKIE" => client_cookie = Some(read_text(reader, "CLTCOOKIE")?),
                    "CCSTMTRS" => ccstmtrs = Some(parse_ccstmtrs(reader)?),
                    _ => reader.skip_element(&name)?,
                }
            }
            Event::End(e) if end_tag_name(&e) == "CCSTMTTRNRS" => break,
            Event::Eof => {
                return Err(XmlError::MalformedXml {
                    message: "unexpected EOF in CCSTMTTRNRS".to_owned(),
                }
                .into());
            }
            _ => {}
        }
    }

    let trnuid = trnuid.ok_or_else(|| XmlError::MissingElement {
        parent: "CCSTMTTRNRS".to_owned(),
        element: "TRNUID".to_owned(),
    })?;
    let status = status.ok_or_else(|| XmlError::MissingElement {
        parent: "CCSTMTTRNRS".to_owned(),
        element: "STATUS".to_owned(),
    })?;

    let mut wrapper = TransactionWrapper::new(trnuid, status, ccstmtrs);
    if let Some(cookie) = client_cookie {
        wrapper = wrapper.with_client_cookie(cookie);
    }
    Ok(wrapper)
}

fn parse_ccstmtrs(reader: &mut OfxReader<'_>) -> Result<CcStatementResponse, OfxError> {
    let mut curdef: Option<crate::types::CurrencyCode> = None;
    let mut cc_acct: Option<CreditCardAccount> = None;
    let mut transaction_list: Option<TransactionList> = None;
    let mut ledger_balance: Option<LedgerBalance> = None;
    let mut available_balance: Option<AvailableBalance> = None;
    let mut balance_list: Vec<Balance> = Vec::new();
    let mut marketing_info: Option<String> = None;

    loop {
        match reader.next_event()? {
            Event::Start(e) => {
                let name = tag_name(&e);
                match name.as_str() {
                    "CURDEF" => {
                        let text = read_text(reader, "CURDEF")?;
                        curdef = Some(parse_text_as(&text, "CURDEF")?);
                    }
                    "CCACCTFROM" => cc_acct = Some(parse_cc_account(reader, "CCACCTFROM")?),
                    "BANKTRANLIST" => transaction_list = Some(parse_bank_transaction_list(reader)?),
                    "LEDGERBAL" => ledger_balance = Some(parse_ledger_balance(reader)?),
                    "AVAILBAL" => available_balance = Some(parse_available_balance(reader)?),
                    "BALLIST" => balance_list = parse_balance_list(reader)?,
                    "MKTGINFO" => marketing_info = Some(read_text(reader, "MKTGINFO")?),
                    _ => reader.skip_element(&name)?,
                }
            }
            Event::End(e) if end_tag_name(&e) == "CCSTMTRS" => break,
            Event::Eof => {
                return Err(XmlError::MalformedXml {
                    message: "unexpected EOF in CCSTMTRS".to_owned(),
                }
                .into());
            }
            _ => {}
        }
    }

    let curdef = curdef.ok_or_else(|| XmlError::MissingElement {
        parent: "CCSTMTRS".to_owned(),
        element: "CURDEF".to_owned(),
    })?;
    let cc_acct = cc_acct.ok_or_else(|| XmlError::MissingElement {
        parent: "CCSTMTRS".to_owned(),
        element: "CCACCTFROM".to_owned(),
    })?;

    let mut stmt = CcStatementResponse::new(curdef, cc_acct);
    if let Some(tl) = transaction_list {
        stmt = stmt.with_transaction_list(tl);
    }
    if let Some(lb) = ledger_balance {
        stmt = stmt.with_ledger_balance(lb);
    }
    if let Some(ab) = available_balance {
        stmt = stmt.with_available_balance(ab);
    }
    if !balance_list.is_empty() {
        stmt = stmt.with_balance_list(balance_list);
    }
    if let Some(info) = marketing_info {
        stmt = stmt.with_marketing_info(info);
    }

    Ok(stmt)
}

#[cfg(test)]
mod tests {
    use super::*;

    const SIMPLE_BANK_OFX: &str = r#"<?OFX OFXHEADER="200" VERSION="220" SECURITY="NONE" OLDFILEUID="NONE" NEWFILEUID="NONE"?>
<OFX>
<SIGNONMSGSRSV1>
<SONRS>
<STATUS>
<CODE>0</CODE>
<SEVERITY>INFO</SEVERITY>
</STATUS>
<DTSERVER>20230115120000</DTSERVER>
<LANGUAGE>ENG</LANGUAGE>
<FI>
<ORG>MyBank</ORG>
<FID>1234</FID>
</FI>
</SONRS>
</SIGNONMSGSRSV1>
<BANKMSGSRSV1>
<STMTTRNRS>
<TRNUID>1001</TRNUID>
<STATUS>
<CODE>0</CODE>
<SEVERITY>INFO</SEVERITY>
</STATUS>
<STMTRS>
<CURDEF>USD</CURDEF>
<BANKACCTFROM>
<BANKID>123456789</BANKID>
<ACCTID>987654321</ACCTID>
<ACCTTYPE>CHECKING</ACCTTYPE>
</BANKACCTFROM>
<BANKTRANLIST>
<DTSTART>20230101</DTSTART>
<DTEND>20230131</DTEND>
<STMTTRN>
<TRNTYPE>DEBIT</TRNTYPE>
<DTPOSTED>20230115</DTPOSTED>
<TRNAMT>-50.00</TRNAMT>
<FITID>20230115001</FITID>
<NAME>GROCERY STORE</NAME>
<MEMO>Weekly groceries</MEMO>
</STMTTRN>
</BANKTRANLIST>
<LEDGERBAL>
<BALAMT>1500.00</BALAMT>
<DTASOF>20230115120000</DTASOF>
</LEDGERBAL>
</STMTRS>
</STMTTRNRS>
</BANKMSGSRSV1>
</OFX>"#;

    #[test]
    fn parse_simple_bank_statement() {
        let doc = parse(SIMPLE_BANK_OFX).unwrap();

        assert_eq!(doc.header().version(), "220".parse().unwrap());
        assert!(doc.signon().status().is_success());
        assert_eq!(doc.signon().language(), "ENG");

        let banking = doc.banking().expect("banking should be present");
        assert_eq!(banking.statement_responses().len(), 1);

        let wrapper = &banking.statement_responses()[0];
        assert!(wrapper.status().is_success());
        assert_eq!(wrapper.transaction_uid(), "1001");

        let stmt = wrapper.response().expect("response should be present");
        assert_eq!(stmt.currency_default().as_str(), "USD");
        assert_eq!(stmt.bank_account().bank_id().as_str(), "123456789");
        assert_eq!(stmt.bank_account().account_id().as_str(), "987654321");

        let txn_list = stmt
            .transaction_list()
            .expect("transaction list should be present");
        assert_eq!(txn_list.len(), 1);

        let txn = &txn_list.transactions()[0];
        assert_eq!(txn.transaction_type(), crate::types::TransactionType::Debit);
        assert_eq!(txn.name(), Some("GROCERY STORE"));
        assert_eq!(txn.memo(), Some("Weekly groceries"));

        let ledger = stmt
            .ledger_balance()
            .expect("ledger balance should be present");
        assert_eq!(
            ledger.amount().as_decimal(),
            rust_decimal::Decimal::new(150000, 2)
        );
    }

    #[test]
    fn parse_multiple_transactions() {
        let input = r#"<?OFX OFXHEADER="200" VERSION="220" SECURITY="NONE" OLDFILEUID="NONE" NEWFILEUID="NONE"?>
<OFX>
<SIGNONMSGSRSV1>
<SONRS>
<STATUS><CODE>0</CODE><SEVERITY>INFO</SEVERITY></STATUS>
<DTSERVER>20230131120000</DTSERVER>
<LANGUAGE>ENG</LANGUAGE>
</SONRS>
</SIGNONMSGSRSV1>
<BANKMSGSRSV1>
<STMTTRNRS>
<TRNUID>2001</TRNUID>
<STATUS><CODE>0</CODE><SEVERITY>INFO</SEVERITY></STATUS>
<STMTRS>
<CURDEF>USD</CURDEF>
<BANKACCTFROM>
<BANKID>111222333</BANKID>
<ACCTID>444555666</ACCTID>
<ACCTTYPE>CHECKING</ACCTTYPE>
</BANKACCTFROM>
<BANKTRANLIST>
<DTSTART>20230101</DTSTART>
<DTEND>20230131</DTEND>
<STMTTRN>
<TRNTYPE>DEBIT</TRNTYPE>
<DTPOSTED>20230105</DTPOSTED>
<TRNAMT>-25.00</TRNAMT>
<FITID>TXN001</FITID>
<NAME>COFFEE SHOP</NAME>
</STMTTRN>
<STMTTRN>
<TRNTYPE>CREDIT</TRNTYPE>
<DTPOSTED>20230110</DTPOSTED>
<TRNAMT>3000.00</TRNAMT>
<FITID>TXN002</FITID>
<NAME>PAYROLL</NAME>
<MEMO>Direct deposit</MEMO>
</STMTTRN>
<STMTTRN>
<TRNTYPE>DEBIT</TRNTYPE>
<DTPOSTED>20230115</DTPOSTED>
<TRNAMT>-1200.00</TRNAMT>
<FITID>TXN003</FITID>
<NAME>RENT PAYMENT</NAME>
<CHECKNUM>1042</CHECKNUM>
</STMTTRN>
<STMTTRN>
<TRNTYPE>DEBIT</TRNTYPE>
<DTPOSTED>20230120</DTPOSTED>
<TRNAMT>-89.50</TRNAMT>
<FITID>TXN004</FITID>
<NAME>ELECTRIC COMPANY</NAME>
<MEMO>Monthly bill</MEMO>
</STMTTRN>
<STMTTRN>
<TRNTYPE>DEBIT</TRNTYPE>
<DTPOSTED>20230125</DTPOSTED>
<TRNAMT>-42.99</TRNAMT>
<FITID>TXN005</FITID>
<NAME>STREAMING SERVICE</NAME>
<SIC>7812</SIC>
</STMTTRN>
</BANKTRANLIST>
<LEDGERBAL>
<BALAMT>1642.51</BALAMT>
<DTASOF>20230131120000</DTASOF>
</LEDGERBAL>
<AVAILBAL>
<BALAMT>1542.51</BALAMT>
<DTASOF>20230131120000</DTASOF>
</AVAILBAL>
</STMTRS>
</STMTTRNRS>
</BANKMSGSRSV1>
</OFX>"#;

        let doc = parse(input).unwrap();
        let banking = doc.banking().unwrap();
        let stmt = banking.statement_responses()[0].response().unwrap();

        // Verify all 5 transactions were parsed
        let txn_list = stmt.transaction_list().unwrap();
        assert_eq!(txn_list.len(), 5);

        // Verify each transaction's unique data
        assert_eq!(txn_list.transactions()[0].fit_id().as_str(), "TXN001");
        assert_eq!(txn_list.transactions()[0].name(), Some("COFFEE SHOP"));
        assert_eq!(
            txn_list.transactions()[0].amount().as_decimal(),
            rust_decimal::Decimal::new(-2500, 2)
        );

        assert_eq!(txn_list.transactions()[1].fit_id().as_str(), "TXN002");
        assert_eq!(
            txn_list.transactions()[1].transaction_type(),
            crate::types::TransactionType::Credit
        );
        assert_eq!(txn_list.transactions()[1].memo(), Some("Direct deposit"));

        assert_eq!(txn_list.transactions()[2].fit_id().as_str(), "TXN003");
        assert_eq!(
            txn_list.transactions()[2].check_number().unwrap().as_str(),
            "1042"
        );

        assert_eq!(txn_list.transactions()[3].fit_id().as_str(), "TXN004");
        assert_eq!(txn_list.transactions()[3].memo(), Some("Monthly bill"));

        assert_eq!(txn_list.transactions()[4].fit_id().as_str(), "TXN005");
        assert_eq!(txn_list.transactions()[4].sic(), Some(7812));

        // Verify both balances
        let ledger = stmt.ledger_balance().unwrap();
        assert_eq!(
            ledger.amount().as_decimal(),
            rust_decimal::Decimal::new(164251, 2)
        );

        let avail = stmt.available_balance().unwrap();
        assert_eq!(
            avail.amount().as_decimal(),
            rust_decimal::Decimal::new(154251, 2)
        );
    }

    #[test]
    fn parse_missing_header_returns_error() {
        let result = parse("<OFX></OFX>");
        assert!(result.is_err());
    }

    #[test]
    fn parse_missing_signon_returns_error() {
        let input = r#"<?OFX OFXHEADER="200" VERSION="220" SECURITY="NONE" OLDFILEUID="NONE" NEWFILEUID="NONE"?><OFX></OFX>"#;
        let result = parse(input);
        assert!(result.is_err());
    }
}
