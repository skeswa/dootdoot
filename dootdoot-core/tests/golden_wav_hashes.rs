//! Golden WAV hash contract tests.

use std::collections::BTreeMap;

use dootdoot_core::{render_text_canonical_buffer, wav_bytes};

const GOLDEN_CORPUS: &str = include_str!("fixtures/golden_corpus.tsv");
const GOLDEN_HASHES: &str = include_str!("fixtures/golden_wav_hashes.tsv");
const SHA256_BLOCK_BYTES: usize = 64;
const SHA256_WORD_COUNT: usize = 64;
const SHA256_LENGTH_OFFSET: usize = 56;
const SHA256_INITIAL_STATE: [u32; 8] = [
    0x6a09_e667,
    0xbb67_ae85,
    0x3c6e_f372,
    0xa54f_f53a,
    0x510e_527f,
    0x9b05_688c,
    0x1f83_d9ab,
    0x5be0_cd19,
];
const SHA256_ROUND_CONSTANTS: [u32; SHA256_WORD_COUNT] = [
    0x428a_2f98,
    0x7137_4491,
    0xb5c0_fbcf,
    0xe9b5_dba5,
    0x3956_c25b,
    0x59f1_11f1,
    0x923f_82a4,
    0xab1c_5ed5,
    0xd807_aa98,
    0x1283_5b01,
    0x2431_85be,
    0x550c_7dc3,
    0x72be_5d74,
    0x80de_b1fe,
    0x9bdc_06a7,
    0xc19b_f174,
    0xe49b_69c1,
    0xefbe_4786,
    0x0fc1_9dc6,
    0x240c_a1cc,
    0x2de9_2c6f,
    0x4a74_84aa,
    0x5cb0_a9dc,
    0x76f9_88da,
    0x983e_5152,
    0xa831_c66d,
    0xb003_27c8,
    0xbf59_7fc7,
    0xc6e0_0bf3,
    0xd5a7_9147,
    0x06ca_6351,
    0x1429_2967,
    0x27b7_0a85,
    0x2e1b_2138,
    0x4d2c_6dfc,
    0x5338_0d13,
    0x650a_7354,
    0x766a_0abb,
    0x81c2_c92e,
    0x9272_2c85,
    0xa2bf_e8a1,
    0xa81a_664b,
    0xc24b_8b70,
    0xc76c_51a3,
    0xd192_e819,
    0xd699_0624,
    0xf40e_3585,
    0x106a_a070,
    0x19a4_c116,
    0x1e37_6c08,
    0x2748_774c,
    0x34b0_bcb5,
    0x391c_0cb3,
    0x4ed8_aa4a,
    0x5b9c_ca4f,
    0x682e_6ff3,
    0x748f_82ee,
    0x78a5_636f,
    0x84c8_7814,
    0x8cc7_0208,
    0x90be_fffa,
    0xa450_6ceb,
    0xbef9_a3f7,
    0xc671_78f2,
];

#[test]
fn golden_corpus_wav_hashes_match_committed_fixture() {
    if std::env::var_os("DOOTDOOT_REGEN_GOLDEN").is_some() {
        regenerate_golden_hashes();
        return;
    }

    let hashes = golden_hashes();

    for case in golden_cases() {
        let buffer =
            render_text_canonical_buffer(case.text).expect("golden corpus case should render");
        let bytes = wav_bytes(&buffer).expect("golden corpus WAV should serialize");
        let actual = sha256_hex(&bytes);
        let expected = hashes
            .get(case.label)
            .expect("golden corpus label should have a committed hash");

        assert_eq!(
            &actual, expected,
            "golden WAV hash changed for {}",
            case.label
        );
    }
}

fn regenerate_golden_hashes() {
    use std::fmt::Write as _;

    let mut output = format!("# {}\tlabel\tsha256\n", dootdoot_core::ACTIVE_VOICE);

    for case in golden_cases() {
        let buffer =
            render_text_canonical_buffer(case.text).expect("golden corpus case should render");
        let bytes = wav_bytes(&buffer).expect("golden corpus WAV should serialize");

        writeln!(output, "{}\t{}", case.label, sha256_hex(&bytes))
            .expect("writing to a String cannot fail");
    }

    let path = concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/tests/fixtures/golden_wav_hashes.tsv",
    );

    std::fs::write(path, output).expect("golden hash fixture should be writable");
}

#[derive(Debug, Clone, Copy)]
struct GoldenCase<'a> {
    label: &'a str,
    text: &'a str,
}

fn golden_cases() -> Vec<GoldenCase<'static>> {
    data_lines(GOLDEN_CORPUS)
        .map(|line| {
            let (label, text) = line
                .split_once('\t')
                .expect("golden corpus rows should be tab-separated");

            GoldenCase { label, text }
        })
        .collect()
}

fn golden_hashes() -> BTreeMap<&'static str, &'static str> {
    data_lines(GOLDEN_HASHES)
        .map(|line| {
            line.split_once('\t')
                .expect("golden hash rows should be tab-separated")
        })
        .collect()
}

fn data_lines(tsv: &'static str) -> impl Iterator<Item = &'static str> {
    tsv.lines()
        .filter(|line| !line.trim().is_empty() && !line.starts_with('#'))
}

fn sha256_hex(bytes: &[u8]) -> String {
    let digest = sha256(bytes);
    let mut output = String::with_capacity(64);

    for byte in digest {
        output.push(nibble_to_hex(byte >> 4));
        output.push(nibble_to_hex(byte & 0x0f));
    }

    output
}

fn sha256(bytes: &[u8]) -> [u8; 32] {
    let mut state = SHA256_INITIAL_STATE;
    let mut message = bytes.to_vec();
    let bit_length = u64::try_from(bytes.len())
        .expect("golden WAV bytes should fit into u64")
        .checked_mul(8)
        .expect("golden WAV bit length should fit into u64");

    message.push(0x80);
    while message.len() % SHA256_BLOCK_BYTES != SHA256_LENGTH_OFFSET {
        message.push(0);
    }
    message.extend_from_slice(&bit_length.to_be_bytes());

    for block in message.chunks_exact(SHA256_BLOCK_BYTES) {
        compress_sha256_block(&mut state, block);
    }

    let mut digest = [0u8; 32];
    for (chunk, word) in digest.chunks_exact_mut(4).zip(state) {
        chunk.copy_from_slice(&word.to_be_bytes());
    }

    digest
}

fn compress_sha256_block(state: &mut [u32; 8], block: &[u8]) {
    let mut schedule = [0u32; SHA256_WORD_COUNT];
    for (word, chunk) in schedule.iter_mut().take(16).zip(block.chunks_exact(4)) {
        *word = u32::from_be_bytes([chunk[0], chunk[1], chunk[2], chunk[3]]);
    }
    for index in 16..SHA256_WORD_COUNT {
        let small_sigma_zero = schedule[index - 15].rotate_right(7)
            ^ schedule[index - 15].rotate_right(18)
            ^ (schedule[index - 15] >> 3);
        let small_sigma_one = schedule[index - 2].rotate_right(17)
            ^ schedule[index - 2].rotate_right(19)
            ^ (schedule[index - 2] >> 10);
        schedule[index] = schedule[index - 16]
            .wrapping_add(small_sigma_zero)
            .wrapping_add(schedule[index - 7])
            .wrapping_add(small_sigma_one);
    }

    let [
        mut state_a,
        mut state_b,
        mut state_c,
        mut state_d,
        mut state_e,
        mut state_f,
        mut state_g,
        mut state_h,
    ] = *state;
    for (round_constant, word) in SHA256_ROUND_CONSTANTS.iter().zip(schedule) {
        let big_sigma_one =
            state_e.rotate_right(6) ^ state_e.rotate_right(11) ^ state_e.rotate_right(25);
        let choice = (state_e & state_f) ^ (!state_e & state_g);
        let temp_one = state_h
            .wrapping_add(big_sigma_one)
            .wrapping_add(choice)
            .wrapping_add(*round_constant)
            .wrapping_add(word);
        let big_sigma_zero =
            state_a.rotate_right(2) ^ state_a.rotate_right(13) ^ state_a.rotate_right(22);
        let majority = (state_a & state_b) ^ (state_a & state_c) ^ (state_b & state_c);
        let temp_two = big_sigma_zero.wrapping_add(majority);

        state_h = state_g;
        state_g = state_f;
        state_f = state_e;
        state_e = state_d.wrapping_add(temp_one);
        state_d = state_c;
        state_c = state_b;
        state_b = state_a;
        state_a = temp_one.wrapping_add(temp_two);
    }

    for (state_word, compressed_word) in state.iter_mut().zip([
        state_a, state_b, state_c, state_d, state_e, state_f, state_g, state_h,
    ]) {
        *state_word = state_word.wrapping_add(compressed_word);
    }
}

fn nibble_to_hex(nibble: u8) -> char {
    let digits = b"0123456789abcdef";
    let index = usize::from(nibble);

    char::from(digits[index])
}
