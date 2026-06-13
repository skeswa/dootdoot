//! Public facade tests for dootdoot-core.

use dootdoot_core::{
    FORMAT_V1, Format, Mapping, MappingError, Mathx, Synth, TokenVector, TokenizedInput,
    TokenizedToken, Tokenizer, TokenizerError, WavWriter, embedded_mapping, embedded_tokenizer,
};

#[test]
fn public_api_exports_core_stubs() {
    assert_eq!(FORMAT_V1, "FORMAT_V1");
    let embedded: fn() -> Result<Tokenizer, TokenizerError> = embedded_tokenizer;
    let mapping: fn() -> Result<Mapping<'static>, MappingError> = embedded_mapping;
    assert!(std::mem::size_of_val(&embedded) > 0);
    assert!(std::mem::size_of_val(&mapping) > 0);

    let stubs = [
        format!("{Format:?}"),
        format!("{Mathx:?}"),
        format!("{Synth:?}"),
        format!("{WavWriter:?}"),
    ];

    assert_eq!(stubs, ["Format", "Mathx", "Synth", "WavWriter"],);
    assert!(std::any::type_name::<TokenVector>().ends_with("TokenVector"),);
    assert!(std::any::type_name::<TokenizedInput>().ends_with("TokenizedInput"),);
    assert!(std::any::type_name::<TokenizedToken>().ends_with("TokenizedToken"),);
}
