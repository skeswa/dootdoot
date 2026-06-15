//! Tokenization support for the embedded `WordPiece` tokenizer.

use thiserror::Error;
use tokenizers::Tokenizer as HfTokenizer;

const EMBEDDED_TOKENIZER_JSON: &[u8] = include_bytes!("../../assets/tokenizer.json");
const CONTINUATION_PREFIX: &str = "##";
const CONTROL_TOKENS: [&str; 4] = ["[PAD]", "[CLS]", "[SEP]", "[MASK]"];
const UNKNOWN_TOKEN: &str = "[UNK]";

/// Wraps the embedded `WordPiece` tokenizer.
#[derive(Debug, Clone)]
pub struct Tokenizer {
    inner: HfTokenizer,
    control_token_ids: [u32; CONTROL_TOKENS.len()],
    unknown_token_id: u32,
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
}

/// Reports why the embedded tokenizer could not be loaded or run.
#[derive(Debug, Clone, Error, PartialEq, Eq)]
#[error("{message}")]
pub struct TokenizerError {
    message: String,
}

impl Tokenizer {
    /// Loads the embedded `tokenizer.json`.
    ///
    /// # Errors
    ///
    /// Returns an error if the committed tokenizer JSON is malformed or does
    /// not contain the frozen special-token IDs required by `VOICE_V1`.
    pub fn embedded() -> Result<Self, TokenizerError> {
        let inner = HfTokenizer::from_bytes(EMBEDDED_TOKENIZER_JSON).map_err(|error| {
            TokenizerError::new(format!("failed to load embedded tokenizer: {error}"))
        })?;
        let control_token_ids = [
            token_id(&inner, CONTROL_TOKENS[0])?,
            token_id(&inner, CONTROL_TOKENS[1])?,
            token_id(&inner, CONTROL_TOKENS[2])?,
            token_id(&inner, CONTROL_TOKENS[3])?,
        ];
        let unknown_token_id = token_id(&inner, UNKNOWN_TOKEN)?;

        Ok(Self {
            inner,
            control_token_ids,
            unknown_token_id,
        })
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
        let tokens = encoding
            .get_ids()
            .iter()
            .zip(encoding.get_tokens())
            .filter(|(id, _token)| !self.control_token_ids.contains(id))
            .map(|(id, token)| TokenizedToken {
                id: *id,
                text: token.clone(),
                continuation: token.starts_with(CONTINUATION_PREFIX),
            })
            .collect::<Vec<_>>();
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

fn token_id(tokenizer: &HfTokenizer, token: &str) -> Result<u32, TokenizerError> {
    tokenizer.token_to_id(token).ok_or_else(|| {
        TokenizerError::new(format!(
            "embedded tokenizer is missing required token {token}",
        ))
    })
}
