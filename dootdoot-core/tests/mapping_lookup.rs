//! Token mapping lookup tests.

use dootdoot_core::{
    DOOT_ASSET_AXIS_COUNT, DOOT_ASSET_TOKEN_RECORD_BYTES, DootAsset, TokenVector,
    embedded_doot_asset, embedded_mapping,
};

#[test]
fn embedded_mapping_dequantizes_token_records() {
    let asset = embedded_doot_asset().expect("asset should parse");
    let mapping = embedded_mapping().expect("mapping should load");
    let unknown = mapping.lookup(1).expect("[UNK] should have a mapping");
    let expected = expected_vector(&asset, 1);

    assert_eq!(mapping.token_count(), asset.token_count());
    assert_eq!(axis_bits(&unknown), axis_bits(&expected));
    assert_eq!(unknown.weight().to_bits(), expected.weight().to_bits());
}

#[test]
fn mapping_rejects_out_of_range_token_ids() {
    let mapping = embedded_mapping().expect("mapping should load");
    let out_of_range = u32::try_from(mapping.token_count()).expect("token count should fit u32");
    let error = mapping
        .lookup(out_of_range)
        .expect_err("token ID past the record table should fail");

    assert!(
        error.to_string().contains("outside mapping table"),
        "unexpected error: {error}",
    );
}

fn expected_vector(asset: &DootAsset, token_id: usize) -> TokenVector {
    let start = token_id * DOOT_ASSET_TOKEN_RECORD_BYTES;
    let record = &asset.record_bytes()[start..start + DOOT_ASSET_TOKEN_RECORD_BYTES];
    let axes = [
        f64::from(read_i16(record, 0)) * f64::from(asset.axis_scales()[0]),
        f64::from(read_i16(record, 2)) * f64::from(asset.axis_scales()[1]),
        f64::from(read_i16(record, 4)) * f64::from(asset.axis_scales()[2]),
        f64::from(read_i16(record, 6)) * f64::from(asset.axis_scales()[3]),
    ];
    let weight = f64::from(read_i16(record, 8)) * f64::from(asset.weight_scale());

    TokenVector::new(axes, weight)
}

fn axis_bits(vector: &TokenVector) -> [u64; DOOT_ASSET_AXIS_COUNT] {
    vector.axes().map(f64::to_bits)
}

fn read_i16(bytes: &[u8], offset: usize) -> i16 {
    i16::from_le_bytes(
        bytes[offset..offset + 2]
            .try_into()
            .expect("record field should be present"),
    )
}
