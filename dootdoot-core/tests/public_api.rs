//! Public facade tests for dootdoot-core.

use dootdoot_core::{
    ACTIVE_FORMAT, FORMAT_V1, FORMAT_V2, Format, KNOB_BOUNDS, KNOB_MODULATION_DEPTHS, KnobBounds,
    KnobSet, Mapping, MappingError, Mathx, SquashedVector, Synth, TokenVector, TokenizedInput,
    TokenizedToken, Tokenizer, TokenizerError, WavWriter, assemble_knob_sequence, assemble_knobs,
    embedded_mapping, embedded_tokenizer, pool_sequence,
};

#[test]
fn public_api_exports_core_stubs() {
    assert_eq!(FORMAT_V1, "FORMAT_V1");
    assert_eq!(FORMAT_V2, "FORMAT_V2");
    assert_eq!(ACTIVE_FORMAT, FORMAT_V2);
    let embedded: fn() -> Result<Tokenizer, TokenizerError> = embedded_tokenizer;
    let mapping: fn() -> Result<Mapping<'static>, MappingError> = embedded_mapping;
    let pool: fn(&[TokenVector]) -> Result<dootdoot_core::PooledVector, MappingError> =
        pool_sequence;
    let assemble: fn(SquashedVector, SquashedVector) -> KnobSet = assemble_knobs;
    let assemble_sequence: fn(SquashedVector, &[SquashedVector]) -> Vec<KnobSet> =
        assemble_knob_sequence;
    assert!(std::mem::size_of_val(&embedded) > 0);
    assert!(std::mem::size_of_val(&mapping) > 0);
    assert!(std::mem::size_of_val(&pool) > 0);
    assert!(std::mem::size_of_val(&assemble) > 0);
    assert!(std::mem::size_of_val(&assemble_sequence) > 0);
    assert_eq!(KNOB_MODULATION_DEPTHS.len(), KNOB_BOUNDS.len());

    let stubs = [
        format!("{Format:?}"),
        format!("{Mathx:?}"),
        format!("{Synth:?}"),
        format!("{WavWriter:?}"),
    ];

    assert_eq!(stubs, ["Format", "Mathx", "Synth", "WavWriter"],);
    assert!(std::any::type_name::<KnobBounds>().ends_with("KnobBounds"),);
    assert!(std::any::type_name::<KnobSet>().ends_with("KnobSet"),);
    assert!(std::any::type_name::<SquashedVector>().ends_with("SquashedVector"),);
    assert!(std::any::type_name::<dootdoot_core::PooledVector>().ends_with("PooledVector"),);
    assert!(std::any::type_name::<TokenVector>().ends_with("TokenVector"),);
    assert!(std::any::type_name::<TokenizedInput>().ends_with("TokenizedInput"),);
    assert!(std::any::type_name::<TokenizedToken>().ends_with("TokenizedToken"),);
}
