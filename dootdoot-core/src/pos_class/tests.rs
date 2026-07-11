use super::PosClass;

#[test]
fn content_classes_report_content() {
    assert!(PosClass::Noun.is_content());
    assert!(PosClass::Verb.is_content());
    assert!(!PosClass::Other.is_content());
}
