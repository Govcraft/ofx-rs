//! SGML-to-XML normalization for OFX 1.x documents.
//!
//! OFX versions 1.x use SGML, where closing tags are optional for leaf elements
//! and even for aggregate elements. For example:
//! ```text
//! <STATUS>
//! <CODE>0
//! <SEVERITY>INFO
//! <DTSERVER>20240402
//! ```
//! Here, `<CODE>` and `<SEVERITY>` are leaves closed implicitly, and `<STATUS>`
//! is an aggregate closed implicitly when `<DTSERVER>` (a non-child) appears.
//!
//! This module converts such input into well-formed XML by maintaining knowledge
//! of the OFX tag hierarchy and inserting closing tags as needed.

use std::collections::HashSet;

/// Tags that are known OFX aggregates (contain child elements, not text).
/// When we see a new opening tag that is NOT a known child of the current
/// aggregate, we auto-close back up the stack until we find the right parent.
fn aggregate_tags() -> HashSet<&'static str> {
    [
        "OFX",
        "SIGNONMSGSRSV1",
        "SIGNONMSGSRQV1",
        "SONRS",
        "SONRQ",
        "STATUS",
        "FI",
        "BANKMSGSRSV1",
        "BANKMSGSRQV1",
        "STMTTRNRS",
        "STMTTRNRQ",
        "STMTRS",
        "STMTRQ",
        "BANKACCTFROM",
        "BANKACCTTO",
        "BANKTRANLIST",
        "STMTTRN",
        "LEDGERBAL",
        "AVAILBAL",
        "BALLIST",
        "BAL",
        "CREDITCARDMSGSRSV1",
        "CREDITCARDMSGSRQV1",
        "CCSTMTTRNRS",
        "CCSTMTTRNRQ",
        "CCSTMTRS",
        "CCSTMTRQ",
        "CCACCTFROM",
        "CCACCTTO",
        "PAYEE",
        "CURRENCY",
        "ORIGCURRENCY",
        "INVSTMTMSGSRSV1",
        "INVSTMTTRNRS",
        "INVSTMTRS",
        "INVACCTFROM",
        "INVTRANLIST",
        "INVBANKTRAN",
        "INVPOSLIST",
        "INVBAL",
        "INVOOLIST",
        "SECLISTMSGSRSV1",
        "SECLIST",
        "BUYSTOCK",
        "INVBUY",
        "INVTRAN",
        "SECID",
        "POSSTOCK",
        "POSOPT",
        "INVPOS",
        "STOCKINFO",
        "SECINFO",
        "OPTINFO",
        "OOBUYSTOCK",
        "OO",
        "MFAchallengerq",
        "MFACHALLENGERQ",
        "MFACHALLENGERS",
        "CHALLENGERQ",
        "CHALLENGERS",
        "PINCHRQ",
        "PINCHRS",
        "OFXEXTENSION",
    ]
    .into_iter()
    .collect()
}

/// Emit a closing tag: `</NAME>`.
fn emit_close(output: &mut String, name: &str) {
    output.push_str("</");
    output.push_str(name);
    output.push('>');
}

/// Handle an explicit closing tag (e.g., `</STATUS>`).
fn handle_close_tag(
    close_name: &str,
    trimmed_text: &str,
    stack: &mut Vec<String>,
    output: &mut String,
    aggregates: &HashSet<&str>,
) {
    // If top-of-stack is a leaf that matches this close tag, close it directly.
    if let Some(top) = stack.last() {
        if top == close_name && !aggregates.contains(top.as_str()) {
            if !trimmed_text.is_empty() {
                output.push_str(trimmed_text);
            }
            emit_close(output, close_name);
            stack.pop();
            return;
        }
        // Top is an unclosed leaf that doesn't match -- close it first
        if !aggregates.contains(top.as_str()) {
            if !trimmed_text.is_empty() {
                output.push_str(trimmed_text);
            }
            emit_close(output, top);
            stack.pop();
        } else if !trimmed_text.is_empty() {
            output.push_str(trimmed_text);
        }
    }

    // Close tags up to and including the named one
    while let Some(top) = stack.last() {
        let is_target = top == close_name;
        emit_close(output, top);
        stack.pop();
        if is_target {
            break;
        }
    }
}

/// Handle an opening tag (e.g., `<STMTTRN>`).
fn handle_open_tag(
    tag_content: &str,
    trimmed_text: &str,
    stack: &mut Vec<String>,
    output: &mut String,
    aggregates: &HashSet<&str>,
) {
    let tag_name = tag_content
        .split_whitespace()
        .next()
        .unwrap_or(tag_content);

    // If current top-of-stack is a leaf, close it first
    if let Some(top) = stack.last() {
        if !aggregates.contains(top.as_str()) {
            if !trimmed_text.is_empty() {
                output.push_str(trimmed_text);
            }
            emit_close(output, top);
            stack.pop();
        } else if !trimmed_text.is_empty() {
            output.push_str(trimmed_text);
        }
    }

    // For aggregate tags, auto-close to find a valid parent.
    // For leaf tags in a wrong-parent aggregate, close back up.
    if aggregates.contains(tag_name) {
        close_to_valid_parent(stack, output, tag_name, aggregates);
    } else if let Some(top) = stack.last()
        && aggregates.contains(top.as_str())
        && !is_valid_leaf_parent(top, tag_name)
    {
        close_to_valid_leaf_parent(stack, output, tag_name, aggregates);
    }

    output.push('<');
    output.push_str(tag_content);
    output.push('>');
    stack.push(tag_name.to_owned());
}

/// Converts OFX SGML content into well-formed XML.
///
/// Maintains a stack of open tags and uses knowledge of which tags are
/// aggregates vs. leaves to insert closing tags in the correct positions.
#[allow(clippy::cognitive_complexity)]
pub fn normalize_sgml_to_xml(input: &str) -> String {
    let aggregates = aggregate_tags();
    let mut output = String::with_capacity(input.len() + input.len() / 3);
    let mut stack: Vec<String> = Vec::new();
    let mut pending_text = String::new();

    let mut pos = 0;
    let bytes = input.as_bytes();
    let len = bytes.len();

    while pos < len {
        if bytes[pos] == b'<' {
            // Flush pending text for any current open leaf
            let trimmed_text = pending_text.trim().to_owned();
            pending_text.clear();

            // Read tag content
            let tag_start = pos + 1;
            let tag_end = match input[tag_start..].find('>') {
                Some(i) => tag_start + i,
                None => break,
            };
            let tag_content = &input[tag_start..tag_end];
            pos = tag_end + 1;

            if let Some(close_name) = tag_content.strip_prefix('/') {
                handle_close_tag(
                    close_name.trim(), &trimmed_text, &mut stack, &mut output, &aggregates,
                );
            } else if tag_content.starts_with('?') || tag_content.starts_with('!') {
                output.push('<');
                output.push_str(tag_content);
                output.push('>');
            } else {
                handle_open_tag(
                    tag_content, &trimmed_text, &mut stack, &mut output, &aggregates,
                );
            }
        } else {
            pending_text.push(input.as_bytes()[pos] as char);
            pos += 1;
        }
    }

    // Close any remaining open tags
    let trimmed_text = pending_text.trim().to_owned();
    if let Some(top) = stack.last()
        && !aggregates.contains(top.as_str())
        && !trimmed_text.is_empty()
    {
        output.push_str(&trimmed_text);
    }
    while let Some(top) = stack.pop() {
        output.push_str("</");
        output.push_str(&top);
        output.push('>');
    }

    output
}

/// Known parent-child relationships for OFX aggregates.
/// Returns true if `child` is a valid direct child aggregate of `parent`.
fn is_valid_child(parent: &str, child: &str) -> bool {
    let children: &[&str] = match parent {
        "OFX" => &["SIGNONMSGSRSV1", "SIGNONMSGSRQV1", "BANKMSGSRSV1", "BANKMSGSRQV1",
                    "CREDITCARDMSGSRSV1", "CREDITCARDMSGSRQV1", "INVSTMTMSGSRSV1",
                    "SECLISTMSGSRSV1"],
        "SIGNONMSGSRSV1" => &["SONRS"],
        "SIGNONMSGSRQV1" => &["SONRQ"],
        "SONRS" | "SONRQ" => &["STATUS", "FI"],
        "BANKMSGSRSV1" => &["STMTTRNRS", "STMTTRNRQ"],
        "STMTTRNRS" => &["STATUS", "STMTRS"],
        "STMTTRNRQ" => &["STMTRQ"],
        "STMTRS" => &["BANKACCTFROM", "BANKTRANLIST", "LEDGERBAL", "AVAILBAL", "BALLIST"],
        "STMTRQ" => &["BANKACCTFROM"],
        "BANKTRANLIST" | "INVBANKTRAN" => &["STMTTRN"],
        "STMTTRN" => &["PAYEE", "BANKACCTTO", "CCACCTTO", "CURRENCY", "ORIGCURRENCY"],
        "BALLIST" => &["BAL"],
        "CREDITCARDMSGSRSV1" => &["CCSTMTTRNRS", "CCSTMTTRNRQ"],
        "CCSTMTTRNRS" => &["STATUS", "CCSTMTRS"],
        "CCSTMTRS" => &["CCACCTFROM", "BANKTRANLIST", "LEDGERBAL", "AVAILBAL", "BALLIST"],
        "INVSTMTMSGSRSV1" => &["INVSTMTTRNRS"],
        "INVSTMTTRNRS" => &["STATUS", "INVSTMTRS"],
        "INVSTMTRS" => &["INVACCTFROM", "INVTRANLIST", "INVPOSLIST", "INVBAL", "INVOOLIST"],
        "INVTRANLIST" => &["BUYSTOCK", "INVBANKTRAN"],
        "BUYSTOCK" => &["INVBUY"],
        "INVBUY" => &["INVTRAN", "SECID"],
        "INVPOSLIST" => &["POSSTOCK", "POSOPT"],
        "POSSTOCK" | "POSOPT" => &["INVPOS"],
        "INVPOS" | "OO" | "SECINFO" => &["SECID"],
        "INVBAL" => &["BALLIST"],
        "INVOOLIST" => &["OOBUYSTOCK"],
        "OOBUYSTOCK" => &["OO"],
        "SECLISTMSGSRSV1" => &["SECLIST"],
        "SECLIST" => &["STOCKINFO", "OPTINFO"],
        "STOCKINFO" | "OPTINFO" => &["SECINFO", "SECID"],
        _ => return false,
    };
    children.contains(&child)
}

/// Returns true if `parent` aggregate can directly contain a leaf element named `leaf`.
/// Leaf elements are any tag that is not an aggregate; we check by exclusion from
/// known aggregate-only parents. If a parent is in this list, it must contain
/// the leaf directly, otherwise the leaf belongs to a sibling or ancestor.
fn is_valid_leaf_parent(parent: &str, leaf: &str) -> bool {
    // STATUS contains CODE and SEVERITY
    if parent == "STATUS" {
        return matches!(leaf, "CODE" | "SEVERITY" | "MESSAGE" | "CODEVALIDATION" | "USERKEY");
    }
    // FI contains ORG and FID
    if parent == "FI" {
        return matches!(leaf, "ORG" | "FID");
    }
    // BANKACCTFROM/TO contain BANKID, ACCTID, ACCTTYPE, BRANCHID
    if matches!(parent, "BANKACCTFROM" | "BANKACCTTO") {
        return matches!(leaf, "BANKID" | "ACCTID" | "ACCTTYPE" | "BRANCHID");
    }
    // CCACCTFROM/TO contain ACCTID, ACCTTYPE
    if matches!(parent, "CCACCTFROM" | "CCACCTTO") {
        return matches!(leaf, "ACCTID" | "ACCTTYPE" | "ACCTKEY");
    }
    // BANKTRANLIST contains DTSTART, DTEND (STMTTRN is an aggregate, handled separately)
    if parent == "BANKTRANLIST" {
        return matches!(leaf, "DTSTART" | "DTEND");
    }
    // STMTTRN contains all the transaction leaf fields
    if parent == "STMTTRN" {
        return matches!(
            leaf,
            "TRNTYPE" | "DTPOSTED" | "DTUSER" | "DTAVAIL" | "TRNAMT" | "FITID"
            | "SRVRTID" | "CHECKNUM" | "REFNUM" | "SIC" | "PAYEEID" | "NAME"
            | "EXTDBINFON" | "MEMO" | "CORRECTFITID" | "CORRECTACTION" | "INV401KSOURCE"
        );
    }
    // LEDGERBAL and AVAILBAL contain BALAMT and DTASOF
    if matches!(parent, "LEDGERBAL" | "AVAILBAL") {
        return matches!(leaf, "BALAMT" | "DTASOF");
    }
    // INVACCTFROM
    if parent == "INVACCTFROM" {
        return matches!(leaf, "BROKERID" | "ACCTID" | "ACCTTYPE");
    }
    // SECID
    if parent == "SECID" {
        return matches!(leaf, "UNIQUEID" | "UNIQUEIDTYPE");
    }
    // SECINFO
    if parent == "SECINFO" {
        return matches!(leaf, "SECNAME" | "TICKER" | "FIID" | "RATING" | "UNITPRICE" | "DTASOF" | "CURRENCY" | "MEMO");
    }
    // For aggregates not listed here, assume any leaf could be a direct child
    // (conservative: don't close unnecessarily)
    true
}

/// Close aggregate tags until we find a valid parent for a leaf tag.
fn close_to_valid_leaf_parent(
    stack: &mut Vec<String>,
    output: &mut String,
    leaf: &str,
    aggregates: &HashSet<&str>,
) {
    // Find how many aggregates to close
    let mut close_count = 0;
    for (i, tag) in stack.iter().rev().enumerate() {
        if is_valid_leaf_parent(tag, leaf) {
            close_count = i;
            break;
        }
        if aggregates.contains(tag.as_str()) {
            close_count = i + 1;
        }
    }
    for _ in 0..close_count {
        if let Some(top) = stack.pop() {
            output.push_str("</");
            output.push_str(&top);
            output.push('>');
        }
    }
}

/// Close aggregate tags on the stack until we find a valid parent for `new_tag`.
fn close_to_valid_parent(
    stack: &mut Vec<String>,
    output: &mut String,
    new_tag: &str,
    aggregates: &HashSet<&str>,
) {
    // Walk the stack from top to find a tag that accepts new_tag as a child
    let mut close_count = 0;
    for (i, tag) in stack.iter().rev().enumerate() {
        if is_valid_child(tag, new_tag) {
            close_count = i;
            break;
        }
        // If we've walked through an aggregate that doesn't accept this child,
        // we need to close it
        if aggregates.contains(tag.as_str()) {
            close_count = i + 1;
        }
    }

    // Close the identified tags
    for _ in 0..close_count {
        if let Some(top) = stack.pop() {
            output.push_str("</");
            output.push_str(&top);
            output.push('>');
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn simple_leaf_elements() {
        let input = "<CODE>0\n<SEVERITY>INFO\n";
        let result = normalize_sgml_to_xml(input);
        assert!(result.contains("<CODE>0</CODE>"));
        assert!(result.contains("<SEVERITY>INFO</SEVERITY>"));
    }

    #[test]
    fn aggregate_with_explicit_close() {
        let input = "<STATUS>\n<CODE>0\n<SEVERITY>INFO\n</STATUS>\n";
        let result = normalize_sgml_to_xml(input);
        assert!(result.contains("<CODE>0</CODE>"));
        assert!(result.contains("<SEVERITY>INFO</SEVERITY>"));
        assert!(result.contains("</STATUS>"));
    }

    #[test]
    fn aggregate_with_implicit_close() {
        // DTSERVER is a sibling of STATUS in SONRS, so STATUS should auto-close
        let input = "<SONRS>\n<STATUS>\n<CODE>0\n<SEVERITY>INFO\n<DTSERVER>20240402\n<LANGUAGE>ENG\n</SONRS>\n";
        let result = normalize_sgml_to_xml(input);
        assert!(result.contains("<CODE>0</CODE>"));
        assert!(result.contains("<SEVERITY>INFO</SEVERITY>"));
        assert!(result.contains("</STATUS>"));
        assert!(result.contains("<DTSERVER>20240402</DTSERVER>"));
        assert!(result.contains("<LANGUAGE>ENG</LANGUAGE>"));
        assert!(result.contains("</SONRS>"));
    }

    #[test]
    fn already_valid_xml_passes_through() {
        let input = "<STATUS><CODE>0</CODE><SEVERITY>INFO</SEVERITY></STATUS>";
        let result = normalize_sgml_to_xml(input);
        assert!(result.contains("<CODE>0</CODE>"));
        assert!(result.contains("<SEVERITY>INFO</SEVERITY>"));
        assert!(result.contains("</STATUS>"));
    }

    #[test]
    fn datetime_with_timezone_brackets() {
        let input = "<DTSERVER>20260319000000.000[+0:UTC]\n<LANGUAGE>ENG\n";
        let result = normalize_sgml_to_xml(input);
        assert!(
            result.contains("<DTSERVER>20260319000000.000[+0:UTC]</DTSERVER>"),
            "got: {result}"
        );
        assert!(result.contains("<LANGUAGE>ENG</LANGUAGE>"));
    }

    #[test]
    fn mercury_style_signon() {
        let input = "<OFX>\n<SIGNONMSGSRSV1>\n<SONRS>\n<STATUS>\n<CODE>0\n<SEVERITY>INFO\n<DTSERVER>20260319\n<LANGUAGE>ENG\n<BANKMSGSRSV1>\n</BANKMSGSRSV1>\n</OFX>";
        let result = normalize_sgml_to_xml(input);
        // STATUS should be auto-closed before DTSERVER
        assert!(result.contains("</STATUS>"));
        // SONRS and SIGNONMSGSRSV1 should be auto-closed before BANKMSGSRSV1
        assert!(result.contains("</SONRS>"));
        assert!(result.contains("</SIGNONMSGSRSV1>"));
        assert!(result.contains("<BANKMSGSRSV1>"));
    }
}
