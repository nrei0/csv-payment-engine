use std::collections::HashMap;
use std::io::{Cursor, Read};

use csv_payment_engine::engine::engine::Engine;
use csv_payment_engine::output::csv_output::write_accounts_csv;
use csv_payment_engine::source::csv_source::CsvSource;
use csv_payment_engine::util::amount::parse_amount_fixed_decimal;

#[derive(Debug)]
struct AccountRow {
    #[allow(dead_code)]
    client: u16,
    available: i128,
    held: i128,
    total: i128,
    locked: bool,
}

fn parse_output_csv_to_map(output: &str) -> HashMap<u16, AccountRow> {
    let mut reader = csv::ReaderBuilder::new()
        .trim(csv::Trim::All)
        .from_reader(output.as_bytes());

    let mut accounts = HashMap::new();

    for result in reader.records() {
        let record = result.expect("output csv record should be readable");
        let client: u16 = record[0].trim().parse().unwrap();
        let available = parse_amount_fixed_decimal(record[1].trim()).unwrap();
        let held = parse_amount_fixed_decimal(record[2].trim()).unwrap();
        let total = parse_amount_fixed_decimal(record[3].trim()).unwrap();
        let locked: bool = record[4].trim().parse().unwrap();

        accounts.insert(
            client,
            AccountRow {
                client,
                available,
                held,
                total,
                locked,
            },
        );
    }

    accounts
}

fn run_pipeline_ignore_engine_errors(input_csv: &str) -> String {
    let source = CsvSource::new(Cursor::new(input_csv.as_bytes()));
    let mut engine = Engine::new();

    for tx in source {
        let tx = tx.expect("source should parse transactions for this test");
        let _ = engine.apply(&tx);
    }

    let mut out = Vec::new();
    write_accounts_csv(&mut out, engine.accounts()).expect("write_accounts_csv should succeed");
    String::from_utf8(out).expect("output should be utf-8")
}

fn run_and_parse_accounts(input_csv: &str) -> HashMap<u16, AccountRow> {
    let output = run_pipeline_ignore_engine_errors(input_csv);
    parse_output_csv_to_map(&output)
}

#[test]
fn sample_csv() {
    let input = "type, client, tx, amount\n\
deposit, 1, 1, 1.0000\n\
deposit, 2, 2, 2.5000\n\
deposit, 1, 3, 2.0000\n\
withdrawal, 1, 4, 1.5000\n\
withdrawal, 2, 5, 3.0000\n\
deposit, 3, 6, 10.0000\n\
withdrawal, 3, 7, 1.2345\n\
deposit, 2, 8, 0.0001\n\
deposit, 4, 9, 5\n\
withdrawal, 4, 10, 2.0000\n\
dispute, 1, 1,\n\
resolve, 1, 1,\n\
dispute, 1, 1,\n\
chargeback, 1, 1,\n\
deposit, 1, 11, 100.0000\n\
withdrawal, 1, 12, 1.0000\n\
dispute, 2, 2,\n\
dispute, 2, 2,\n\
resolve, 2, 2,\n\
resolve, 2, 2,\n\
dispute, 2, 5,\n\
chargeback, 2, 5,\n\
deposit, 2, 13, 7.7777\n\
withdrawal, 2, 14, 1.1111\n\
dispute, 3, 6,\n\
withdrawal, 3, 15, 9.0000\n\
resolve, 3, 6,\n\
withdrawal, 3, 16, 9.0000\n\
deposit, 5, 17, 3.3333\n\
withdrawal, 5, 18, 3.3334\n\
withdrawal, 5, 19, 3.3333\n\
dispute, 5, 17,\n\
chargeback, 5, 17,\n\
withdrawal, 5, 20, 0.0001\n\
deposit, 6, 21, 1.0000\n\
dispute, 6, 999999,\n\
resolve, 6, 999999,\n\
chargeback, 6, 999999,\n\
deposit, 7, 22, 2.0000\n\
withdrawal, 7, 23, 1.9999\n\
dispute, 7, 23,\n\
resolve, 7, 23,\n\
withdrawal, 7, 24, 0.0002\n\
deposit, 8, 25, 10.0000\n\
withdrawal, 8, 26, 5.0000\n\
dispute, 8, 25,\n\
chargeback, 8, 25,\n\
deposit, 8, 27, 1.0000\n";

    let output = run_pipeline_ignore_engine_errors(input);
    let got = parse_output_csv_to_map(&output);

    let expected: Vec<(u16, i128, i128, i128, bool)> = vec![
        // client, available, held, total, locked
        (1, 5_000, 0, 5_000, true), // locked after chargeback on tx 1
        (2, 91_667, 0, 91_667, false),
        (3, 87_655, 0, 87_655, false),
        (4, 30_000, 0, 30_000, false),
        (5, 0, 0, 0, false),
        (6, 10_000, 0, 10_000, false),
        (7, 1, 0, 1, false), // 0.0001 remaining
        (8, 60_000, 0, 60_000, false),
    ];

    for (client, avail, held, total, locked) in expected {
        let row = got
            .get(&client)
            .unwrap_or_else(|| panic!("missing client {client} in output"));

        assert_eq!(row.available, avail, "client {client} available mismatch");
        assert_eq!(row.held, held, "client {client} held mismatch");
        assert_eq!(row.total, total, "client {client} total mismatch");
        assert_eq!(row.locked, locked, "client {client} locked mismatch");
        assert_eq!(
            row.available + row.held,
            row.total,
            "client {client} invariant broken"
        );
    }
}

#[test]
fn source_missing_amount_for_deposit_should_error() {
    let input = "type, client, tx, amount\n\
deposit, 1, 1,\n";

    let mut source = CsvSource::new(Cursor::new(input.as_bytes()));
    let first = source.next().expect("should have one row");
    assert!(first.is_err(), "missing amount should be a source error");
}

#[test]
fn source_missing_amount_for_withdrawal_should_error() {
    let input = "type, client, tx, amount\n\
withdrawal, 1, 1,\n";

    let mut source = CsvSource::new(Cursor::new(input.as_bytes()));
    let first = source.next().expect("should have one row");
    assert!(first.is_err(), "missing amount should be a source error");
}

#[test]
fn source_unknown_type_should_error() {
    let input = "type, client, tx, amount\n\
weird, 1, 1, 1.0\n";

    let mut source = CsvSource::new(Cursor::new(input.as_bytes()));
    let first = source.next().expect("should have one row");
    assert!(first.is_err(), "unknown type should be a source error");
}

#[test]
fn negative_deposit_is_rejected_by_engine() {
    let input = "type, client, tx, amount\n\
deposit, 1, 1, -1.0000\n";

    let mut source = CsvSource::new(Cursor::new(input.as_bytes()));
    let tx = source
        .next()
        .unwrap()
        .expect("source should parse negative amount");

    let mut engine = Engine::new();

    assert!(
        engine.apply(&tx).is_err(),
        "engine should reject negative deposit amount"
    );

    let mut out = Vec::new();
    write_accounts_csv(&mut out, engine.accounts()).unwrap();
    let out_str = String::from_utf8(out).unwrap();
    let got = parse_output_csv_to_map(&out_str);

    assert!(
        !got.contains_key(&1),
        "rejected transaction should not create/modify account state"
    );
}

#[test]
fn negative_withdrawal_is_rejected_by_engine() {
    let input = "type, client, tx, amount\n\
withdrawal, 1, 1, -1.0000\n";

    let mut source = CsvSource::new(Cursor::new(input.as_bytes()));
    let tx = source
        .next()
        .unwrap()
        .expect("source should parse negative amount");

    let mut engine = Engine::new();

    assert!(
        engine.apply(&tx).is_err(),
        "engine should reject negative withdrawal amount"
    );
}

#[test]
fn zero_amount_deposit_is_rejected_by_engine() {
    let input = "type, client, tx, amount\n\
deposit, 1, 1, 0.0000\n";

    let mut source = CsvSource::new(Cursor::new(input.as_bytes()));
    let tx = source
        .next()
        .unwrap()
        .expect("source should parse zero amount");

    let mut engine = Engine::new();

    assert!(
        engine.apply(&tx).is_err(),
        "engine should reject zero deposit amount"
    );
}

#[test]
fn zero_amount_withdrawal_is_rejected_by_engine() {
    let input = "type, client, tx, amount\n\
withdrawal, 1, 1, 0.0000\n";

    let mut source = CsvSource::new(Cursor::new(input.as_bytes()));
    let tx = source
        .next()
        .unwrap()
        .expect("source should parse zero amount");

    let mut engine = Engine::new();

    assert!(
        engine.apply(&tx).is_err(),
        "engine should reject zero withdrawal amount"
    );
}

#[test]
fn duplicate_transaction_is_rejected_by_engine_and_does_not_double_apply() {
    let input = "type, client, tx, amount\n\
deposit, 1, 1, 1.0000\n\
deposit, 1, 1, 1.0000\n";

    let output = run_pipeline_ignore_engine_errors(input);
    let got = parse_output_csv_to_map(&output);
    let row = got.get(&1).unwrap();

    // Should be applied only once.
    assert_eq!(row.available, 10_000);
    assert_eq!(row.held, 0);
    assert_eq!(row.total, 10_000);
    assert!(!row.locked);
}

#[test]
fn withdrawal_dispute_then_resolve_restores_withdrawal_effect() {
    // deposit 100 -> withdrawal 100 -> dispute withdrawal -> resolve
    // Final should be same as after withdrawal: 0 balance, not locked.
    let input = "type, client, tx, amount\n\
deposit, 1, 1, 100.0000\n\
withdrawal, 1, 2, 100.0000\n\
dispute, 1, 2,\n\
resolve, 1, 2,\n";

    let got = run_and_parse_accounts(input);
    let row = got.get(&1).unwrap();

    assert_eq!(row.available, 0);
    assert_eq!(row.held, 0);
    assert_eq!(row.total, 0);
    assert!(!row.locked);
}

#[test]
fn withdrawal_dispute_then_chargeback_reverses_withdrawal_and_locks() {
    // deposit 100 -> withdrawal 100 -> dispute withdrawal -> chargeback
    // Final: withdrawal reversed, funds returned, account locked.
    let input = "type, client, tx, amount\n\
deposit, 1, 1, 100.0000\n\
withdrawal, 1, 2, 100.0000\n\
dispute, 1, 2,\n\
chargeback, 1, 2,\n";

    let got = run_and_parse_accounts(input);
    let row = got.get(&1).unwrap();

    assert_eq!(row.available, 1_000_000); // 100 * 10^4
    assert_eq!(row.held, 0);
    assert_eq!(row.total, 1_000_000);
    assert!(row.locked);
}

#[test]
fn deposit_dispute_then_chargeback_removes_funds_and_locks() {
    // deposit 100 -> dispute deposit -> chargeback
    // Deposit is reversed permanently: total becomes 0, account locked.
    let input = "type, client, tx, amount\n\
deposit, 1, 1, 100.0000\n\
dispute, 1, 1,\n\
chargeback, 1, 1,\n";

    let got = run_and_parse_accounts(input);
    let row = got.get(&1).unwrap();

    assert_eq!(row.available, 0);
    assert_eq!(row.held, 0);
    assert_eq!(row.total, 0);
    assert!(row.locked);
}

#[test]
fn deposit_dispute_then_resolve_restores_available_and_keeps_total() {
    // deposit 100 -> dispute deposit -> resolve
    // Final: back to normal (no lock): available=100, held=0, total=100
    let input = "type, client, tx, amount\n\
deposit, 1, 1, 100.0000\n\
dispute, 1, 1,\n\
resolve, 1, 1,\n";

    let got = run_and_parse_accounts(input);
    let row = got.get(&1).unwrap();

    assert_eq!(row.available, 1_000_000);
    assert_eq!(row.held, 0);
    assert_eq!(row.total, 1_000_000);
    assert!(!row.locked);
}

#[test]
fn io_error_mid_stream_is_reported_as_source_error() {
    // Create a reader that errors after N bytes.
    struct FailingRead {
        data: Vec<u8>,
        pos: usize,
        fail_after: usize,
    }

    impl Read for FailingRead {
        fn read(&mut self, buf: &mut [u8]) -> std::io::Result<usize> {
            // Fails after N bytes.
            if self.pos >= self.fail_after {
                return Err(std::io::Error::new(std::io::ErrorKind::Other, "boom"));
            }

            // Otherwise, read as normal until end of data.
            if self.pos >= self.data.len() {
                return Ok(0);
            }

            let n = (self.data.len() - self.pos).min(buf.len());
            let end = self.pos + n;
            buf[..n].copy_from_slice(&self.data[self.pos..end]);
            self.pos = end;
            Ok(n)
        }
    }

    let input = b"type, client, tx, amount\n\
deposit, 1, 1, 1.0\n\
deposit, 1, 2, 2.0\n";

    let reader = FailingRead {
        data: input.to_vec(),
        pos: 0,
        fail_after: 25,
    };

    let mut source = CsvSource::new(reader);

    let mut saw_error = false;
    for item in &mut source {
        if item.is_err() {
            saw_error = true;
            break;
        }
    }

    assert!(saw_error, "expected an IO/csv error during streaming read");
}
