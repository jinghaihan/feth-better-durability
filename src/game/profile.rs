pub const TITLE_ID: u64 = 0x0100_55D0_09F7_8000;
pub const DISPLAY_VERSION: &[u8] = b"1.2.0";

// These are text-relative offsets and original AArch64 instructions from the
// FE3H 1.2.0 main NSO (Build ID 89048449BA238C8CF565518B83BF02D3).
pub const BATTLE_CONSUME_OFFSET: usize = 0x000C_2E50;
pub const ATTACK_COST_OFFSET: usize = 0x000C_30A0;
pub const TEXT_SIGNATURES: [(usize, u32); 13] = [
  (0x000C_2E50, 0xF81D_0FF5), // battle consume function prologue
  (0x000C_2E70, 0x940D_2608), // classify spell ID
  (0x000C_2E78, 0x5400_02A0), // branch to non-spell durability handling
  (0x000C_2EEC, 0x940D_26F5), // non-spell durability decrement call
  (0x000C_30A0, 0xA9BD_57F6), // attack-cost function prologue
  (0x000C_30B4, 0x3940_1EC8), // attack record kind at +7
  (0x000C_30C4, 0x7100_5D1F), // all-durability attack kind 0x17
  (0x000C_30DC, 0x7940_02C0), // attack record action ID at +0
  (0x000C_30E8, 0x5400_00A0), // ordinary-attack sentinel 0xFFFF
  (0x000C_30FC, 0x3940_2AC8), // attack record base cost at +0xA
  (0x000C_3114, 0x97FF_FF4F), // attack cost reaches battle consume
  (0x0040_CAC0, 0x3940_0808), // load durability from item +2
  (0x0040_CACC, 0x3900_0808), // store durability to item +2
];

pub fn is_supported_display_version(version: &[u8; 16]) -> bool {
  let length = version.iter().position(|byte| *byte == 0).unwrap_or(16);
  &version[..length] == DISPLAY_VERSION
}

pub fn matches_text_signatures(read_instruction: impl Fn(usize) -> u32) -> bool {
  TEXT_SIGNATURES
    .iter()
    .all(|(offset, expected)| read_instruction(*offset) == *expected)
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn rejects_other_display_versions() {
    let mut supported = [0; 16];
    supported[..DISPLAY_VERSION.len()].copy_from_slice(DISPLAY_VERSION);
    assert!(is_supported_display_version(&supported));
    assert!(!is_supported_display_version(
      b"1.1.1\0\0\0\0\0\0\0\0\0\0\0"
    ));
    assert!(!is_supported_display_version(b"1.2.0-extra\0\0\0\0\0"));
  }

  #[test]
  fn rejects_conflicting_instructions() {
    let original = |offset| {
      TEXT_SIGNATURES
        .iter()
        .find_map(|(candidate, instruction)| (*candidate == offset).then_some(*instruction))
        .unwrap()
    };
    assert!(matches_text_signatures(original));
    assert!(!matches_text_signatures(|offset| {
      if offset == BATTLE_CONSUME_OFFSET {
        0xD503_201F
      } else {
        original(offset)
      }
    }));
  }
}
