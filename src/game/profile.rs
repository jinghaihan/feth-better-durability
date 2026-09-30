use crate::durability::CostPath;

pub const TITLE_ID: u64 = 0x0100_55D0_09F7_8000;
pub const DISPLAY_VERSION: &[u8] = b"1.2.0";

// These are text-relative offsets and original AArch64 instructions from the
// FE3H 1.2.0 main NSO (Build ID 89048449BA238C8CF565518B83BF02D3).
pub const BATTLE_CONSUME_OFFSET: usize = 0x000C_2E50;
pub const ATTACK_COST_OFFSET: usize = 0x000C_30A0;
pub const ORDINARY_ATTACK_CALLER: usize = 0x000D_D4F8;
pub const ATTACK_RECORD_CALLER: usize = 0x000C_3114;
pub const TEXT_SIGNATURES: [(usize, u32); 17] = [
  (0x000C_2E50, 0xF81D_0FF5),            // battle consume function prologue
  (0x000C_2E70, 0x940D_2608),            // classify spell ID
  (0x000C_2E78, 0x5400_02A0),            // branch to non-spell durability handling
  (0x000C_2EEC, 0x940D_26F5),            // non-spell durability decrement call
  (0x000C_30A0, 0xA9BD_57F6),            // attack-cost function prologue
  (0x000C_30B4, 0x3940_1EC8),            // attack record kind at +7
  (0x000C_30C4, 0x7100_5D1F),            // all-durability attack kind 0x17
  (0x000C_30DC, 0x7940_02C0),            // attack record action ID at +0
  (0x000C_30E8, 0x5400_00A0),            // record lookup sentinel, not a combat-art test
  (0x000C_30FC, 0x3940_2AC8),            // attack record base cost at +0xA
  (0x000C_3114, 0x97FF_FF4F),            // attack cost reaches battle consume
  (0x0040_CAC0, 0x3940_0808),            // load durability from item +2
  (0x0040_CACC, 0x3900_0808),            // store durability to item +2
  (0x000D_D4E4, 0x321F_03E2),            // ordinary attack requests two uses
  (0x000D_D4EC, 0x3200_03E2),            // ordinary attack requests one use
  (0x000D_D4F0, 0xF940_06C1),            // ordinary attack supplies its inventory item
  (ORDINARY_ATTACK_CALLER, 0x97FF_9656), // verified ordinary battle consume call
];

pub const fn cost_path(caller: usize) -> CostPath {
  match caller {
    ORDINARY_ATTACK_CALLER => CostPath::OrdinaryAttack,
    ATTACK_RECORD_CALLER => CostPath::AttackRecord,
    _ => CostPath::Unknown,
  }
}

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
  fn only_verified_callers_have_a_known_cost_path() {
    assert_eq!(cost_path(0xDD4F8), CostPath::OrdinaryAttack);
    assert_eq!(cost_path(0xC3114), CostPath::AttackRecord);
    for caller in [
      0,
      0xDD3F8,
      0xDD688,
      0xDD4F4,
      0xDD4FC,
      0xC3110,
      0xC3118,
      usize::MAX,
    ] {
      assert_eq!(cost_path(caller), CostPath::Unknown);
    }
  }

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
    for (modified_offset, _) in TEXT_SIGNATURES {
      assert!(!matches_text_signatures(|offset| {
        if offset == modified_offset {
          0xD503_201F
        } else {
          original(offset)
        }
      }));
    }
  }
}
