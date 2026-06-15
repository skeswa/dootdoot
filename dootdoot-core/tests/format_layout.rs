//! Format layout tests.

use dootdoot_core::{
    FORMAT_AXIS_COUNT, FORMAT_HASH_BYTES, FORMAT_HEADER_BYTES, FORMAT_MAGIC, FORMAT_SCALE_COUNT,
    FORMAT_SQUASH_STATS_PER_AXIS, FORMAT_TOKEN_RECORD_BYTES, FORMAT_VERSION_NUMBER,
};

const FORMAT_LAYOUT: &str = include_str!("../../docs/reference/format_v1.md");

#[test]
fn format_v1_layout_sizes_are_pinned() {
    assert_eq!(FORMAT_MAGIC, *b"DOOTV1\0\0");
    assert_eq!(FORMAT_VERSION_NUMBER, 1);
    assert_eq!(FORMAT_AXIS_COUNT, 4);
    assert_eq!(FORMAT_SCALE_COUNT, 5);
    assert_eq!(FORMAT_HASH_BYTES, 32);
    assert_eq!(FORMAT_SQUASH_STATS_PER_AXIS, 2);
    assert_eq!(FORMAT_HEADER_BYTES, 208);
    assert_eq!(FORMAT_TOKEN_RECORD_BYTES, 10);

    assert!(FORMAT_LAYOUT.contains("little-endian"));
    assert!(FORMAT_LAYOUT.contains("Per-token record"));
}
