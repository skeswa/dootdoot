//! Mood rows in `--explain` tests.

use dootdoot::explain_table_for_text;

const MOOD_SNAPSHOT: &str = include_str!("fixtures/explain_mood_scenarios.txt");

#[test]
fn explain_table_includes_mood_row() {
    let table = explain_table_for_text("VERY HAPPY!!!").expect("explain table should render");

    assert!(
        table
            .lines()
            .any(|line| line.starts_with("mood") && line.contains("valence:"))
    );
    assert!(table.contains("arousal:"));
}

#[test]
fn explain_mood_rows_match_scenario_snapshot() {
    let scenarios = [
        ("calm", "happy day"),
        ("sad", "terrible bad"),
        ("excited", "VERY HAPPY!!!"),
        ("alarm", "DANGER!!! terrible?!"),
    ];
    let mut actual = String::new();

    for (label, text) in scenarios {
        let table = explain_table_for_text(text).expect("explain table should render");
        let mood = table
            .lines()
            .find(|line| line.starts_with("mood"))
            .expect("explain table should include mood row");

        actual.push_str(label);
        actual.push('\t');
        actual.push_str(mood);
        actual.push('\n');
    }

    assert_eq!(actual, MOOD_SNAPSHOT);
}
