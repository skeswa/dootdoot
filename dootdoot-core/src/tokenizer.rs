//! Tokenization support for the embedded `WordPiece` tokenizer.

use thiserror::Error;
use tokenizers::Tokenizer as HfTokenizer;

use crate::{DootAsset, PosClass, PosTable, embedded_doot_asset, embedded_pos_table};

const CONTINUATION_PREFIX: &str = "##";
const CONTROL_TOKENS: [&str; 4] = ["[PAD]", "[CLS]", "[SEP]", "[MASK]"];
const UNKNOWN_TOKEN: &str = "[UNK]";

/// Wraps the embedded `WordPiece` tokenizer.
#[derive(Debug, Clone)]
pub struct Tokenizer {
    inner: HfTokenizer,
    control_token_ids: [u32; CONTROL_TOKENS.len()],
    unknown_token_id: u32,
    pos_table: PosTable,
}

/// Gives the tokenized form of user input after control-token filtering.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TokenizedInput {
    tokens: Vec<TokenizedToken>,
    empty_chirp: bool,
}

/// Gives one voiced tokenizer output token.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TokenizedToken {
    id: u32,
    text: String,
    continuation: bool,
    pos_class: PosClass,
}

/// Reports why the embedded tokenizer could not be loaded or run.
#[derive(Debug, Clone, Error, PartialEq, Eq)]
#[error("{message}")]
pub struct TokenizerError {
    message: String,
}

impl Tokenizer {
    /// Loads the tokenizer carried by a parsed dootdoot asset.
    ///
    /// # Errors
    ///
    /// Returns an error if the asset tokenizer JSON is malformed or does not
    /// contain the frozen special-token IDs required by `VOICE_V1`.
    pub fn from_asset(asset: &DootAsset) -> Result<Self, TokenizerError> {
        let inner = HfTokenizer::from_bytes(asset.tokenizer_json()).map_err(|error| {
            TokenizerError::new(format!(
                "failed to load tokenizer from dootdoot asset: {error}",
            ))
        })?;
        let control_token_ids = [
            token_id(&inner, CONTROL_TOKENS[0])?,
            token_id(&inner, CONTROL_TOKENS[1])?,
            token_id(&inner, CONTROL_TOKENS[2])?,
            token_id(&inner, CONTROL_TOKENS[3])?,
        ];
        let unknown_token_id = token_id(&inner, UNKNOWN_TOKEN)?;
        let pos_table = embedded_pos_table().map_err(|error| {
            TokenizerError::new(format!("failed to load embedded pos table: {error}"))
        })?;

        Ok(Self {
            inner,
            control_token_ids,
            unknown_token_id,
            pos_table,
        })
    }

    /// Loads the embedded dootdoot asset's tokenizer.
    ///
    /// # Errors
    ///
    /// Returns an error if the committed dootdoot asset or its tokenizer JSON
    /// cannot be loaded or validated.
    pub fn embedded() -> Result<Self, TokenizerError> {
        let asset = embedded_doot_asset().map_err(|error| {
            TokenizerError::new(format!("failed to load embedded dootdoot asset: {error}"))
        })?;

        Self::from_asset(&asset)
    }

    /// Tokenizes input with injected special tokens disabled.
    ///
    /// # Errors
    ///
    /// Returns an error when the embedded tokenizer cannot encode the provided
    /// input.
    pub fn tokenize(&self, input: &str) -> Result<TokenizedInput, TokenizerError> {
        let encoding = self.inner.encode(input, false).map_err(|error| {
            TokenizerError::new(format!(
                "failed to tokenize input with VOICE_V1 tokenizer: {error}"
            ))
        })?;
        let mut tokens = encoding
            .get_ids()
            .iter()
            .zip(encoding.get_tokens())
            .filter(|(id, _token)| !self.control_token_ids.contains(id))
            .map(|(id, token)| TokenizedToken {
                id: *id,
                text: token.clone(),
                continuation: token.starts_with(CONTINUATION_PREFIX),
                pos_class: PosClass::Other,
            })
            .collect::<Vec<_>>();

        assign_word_pos_classes(&mut tokens, &self.pos_table);

        let empty_chirp = tokens.is_empty();

        Ok(TokenizedInput {
            tokens,
            empty_chirp,
        })
    }

    /// Returns the IDs filtered out as non-voiced control tokens.
    pub fn control_token_ids(&self) -> [u32; CONTROL_TOKENS.len()] {
        self.control_token_ids
    }

    /// Returns the `[UNK]` token ID, which remains voiced.
    pub fn unknown_token_id(&self) -> u32 {
        self.unknown_token_id
    }
}

impl TokenizedInput {
    /// Returns the voiced tokens after control-token filtering.
    pub fn tokens(&self) -> &[TokenizedToken] {
        &self.tokens
    }

    /// Returns true when tokenization should route to the empty-input chirp.
    pub fn is_empty_chirp(&self) -> bool {
        self.empty_chirp
    }
}

impl TokenizedToken {
    /// Returns the tokenizer ID.
    pub fn id(&self) -> u32 {
        self.id
    }

    /// Returns the tokenizer text for this token.
    pub fn text(&self) -> &str {
        &self.text
    }

    /// Returns true when this token starts with `WordPiece`'s `##` continuation
    /// marker.
    pub fn is_continuation(&self) -> bool {
        self.continuation
    }

    /// Returns the word-level POS class this token carries (`VOICE_V12`).
    ///
    /// The word-initial token establishes the class for its whole word and
    /// continuation tokens inherit it. Without the `spike-noun-verb` gate this
    /// is always [`PosClass::Other`].
    pub fn pos_class(&self) -> PosClass {
        self.pos_class
    }
}

impl TokenizerError {
    fn new(message: impl Into<String>) -> Self {
        Self {
            message: message.into(),
        }
    }
}

/// Loads the embedded `VOICE_V1` tokenizer.
///
/// # Errors
///
/// Returns an error if the embedded tokenizer cannot be loaded or validated.
pub fn embedded_tokenizer() -> Result<Tokenizer, TokenizerError> {
    Tokenizer::embedded()
}

/// Stamps each token with its word-level POS class.
///
/// A word spans one word-initial token plus its following continuation tokens;
/// the whole word's text (continuation prefixes stripped) is looked up once in
/// the baked class table and every token in the span carries the result, so
/// the word-initial token establishes the class and continuations inherit it.
///
/// With the `spike-noun-verb` gate off every word classifies
/// [`PosClass::Other`], keeping every rendered path byte-identical until the
/// `VOICE_V12` freeze; the `cfg!` branch constant-folds.
fn assign_word_pos_classes(tokens: &mut [TokenizedToken], pos_table: &PosTable) {
    let mut index = 0;

    while index < tokens.len() {
        if tokens[index].continuation {
            // A leading continuation has no word-initial token to inherit
            // from; it stays `Other`.
            index += 1;
            continue;
        }

        let start = index;
        let mut word = tokens[index].text.clone();

        index += 1;
        while index < tokens.len() && tokens[index].continuation {
            word.push_str(
                tokens[index]
                    .text
                    .strip_prefix(CONTINUATION_PREFIX)
                    .unwrap_or(&tokens[index].text),
            );
            index += 1;
        }

        let pos_class = if cfg!(feature = "spike-noun-verb") {
            pos_table.class_of(&word)
        } else {
            PosClass::Other
        };

        for token in &mut tokens[start..index] {
            token.pos_class = pos_class;
        }
    }
}

fn token_id(tokenizer: &HfTokenizer, token: &str) -> Result<u32, TokenizerError> {
    tokenizer.token_to_id(token).ok_or_else(|| {
        TokenizerError::new(format!(
            "embedded tokenizer is missing required token {token}",
        ))
    })
}
