use ofx_rs::parse;

fn try_parse(name: &str, input: &str) {
    match parse(input) {
        Ok(doc) => {
            let signon = doc.signon();
            println!("  [OK] {name}");
            println!("       signon status: code={}, severity={}", signon.status().code(), signon.status().severity());
            println!("       language: {}", signon.language());
            if let Some(org) = signon.fi_org() {
                println!("       FI org: {org}");
            }

            if let Some(banking) = doc.banking() {
                for (i, wrapper) in banking.statement_responses().iter().enumerate() {
                    println!("       bank stmt #{i}: trnuid={}, status={}", wrapper.transaction_uid(), wrapper.status().code());
                    if let Some(stmt) = wrapper.response() {
                        println!("         currency: {}", stmt.currency_default().as_str());
                        println!("         account: bank_id={}, acct_id={}, type={}",
                            stmt.bank_account().bank_id().as_str(),
                            stmt.bank_account().account_id().as_str(),
                            stmt.bank_account().account_type());
                        if let Some(tl) = stmt.transaction_list() {
                            println!("         transactions: {}", tl.len());
                            for (j, txn) in tl.transactions().iter().enumerate() {
                                println!("           [{j}] type={} amount={} fitid={} name={:?}",
                                    txn.transaction_type(), txn.amount(), txn.fit_id().as_str(),
                                    txn.name().unwrap_or("(none)"));
                            }
                        }
                        if let Some(lb) = stmt.ledger_balance() {
                            println!("         ledger balance: {}", lb.amount());
                        }
                        if let Some(ab) = stmt.available_balance() {
                            println!("         available balance: {}", ab.amount());
                        }
                        if !stmt.balance_list().is_empty() {
                            println!("         balance list: {} items", stmt.balance_list().len());
                        }
                    }
                }
            }

            if let Some(cc) = doc.credit_card() {
                for (i, wrapper) in cc.statement_responses().iter().enumerate() {
                    println!("       cc stmt #{i}: trnuid={}, status={}", wrapper.transaction_uid(), wrapper.status().code());
                    if let Some(stmt) = wrapper.response() {
                        println!("         currency: {}", stmt.currency_default().as_str());
                        println!("         account: {}", stmt.credit_card_account().account_id().as_str());
                        if let Some(tl) = stmt.transaction_list() {
                            println!("         transactions: {}", tl.len());
                        }
                        if let Some(lb) = stmt.ledger_balance() {
                            println!("         ledger balance: {}", lb.amount());
                        }
                    }
                }
            }
        }
        Err(e) => {
            println!("  [FAIL] {name}");
            println!("         error: {e}");
        }
    }
}

#[test]
fn parse_all_test_vectors() {
    println!();
    println!("=== OFX Test Vector Results ===");
    println!();

    try_parse("stmtrs.ofx", include_str!("../test-vectors/stmtrs.ofx"));
    println!();

    try_parse("stmtrs_euro.ofx", include_str!("../test-vectors/stmtrs_euro.ofx"));
    println!();

    try_parse("mercury-checking.ofx", include_str!("../test-vectors/mercury-checking.ofx"));
    println!();

    try_parse("br_nubank_cc.ofx", include_str!("../test-vectors/br_nubank_cc.ofx"));
    println!();

    try_parse("invstmtrs.ofx", include_str!("../test-vectors/invstmtrs.ofx"));
    println!();
}
