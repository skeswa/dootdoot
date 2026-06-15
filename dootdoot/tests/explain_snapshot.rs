//! `--explain` snapshot tests.

use dootdoot::explain_table_for_text;

const EXPLAIN_SNAPSHOT: &str = include_str!("fixtures/explain_hello_there_question.txt");

#[test]
fn explain_table_matches_committed_snapshot() {
    let table = explain_table_for_text("hello there?").expect("explain table should render");

    assert_eq!(table, EXPLAIN_SNAPSHOT);
}
