/// The 1.2.0 executable indexes its weapon table with item IDs 10 through 509.
pub const FIRST_WEAPON_ID: u16 = 0x000A;
pub const LAST_WEAPON_ID: u16 = 0x01FD;

pub const fn is_weapon(item_id: u16) -> bool {
  item_id >= FIRST_WEAPON_ID && item_id <= LAST_WEAPON_ID
}

/// The attack record's first word identifies a selected combat action;
/// 0xFFFF is the game's sentinel for an ordinary attack. Kind 0x17 is the
/// special all-durability action, which must never be made free.
pub const fn is_combat_art(action_id: u16, kind: u8) -> bool {
  action_id != 0xFFFF || kind == 0x17
}

/// Leave combat arts, spells, and non-weapon items on the original path.
pub const fn effective_cost(item_id: u16, combat_art: bool, requested_cost: i32) -> i32 {
  if is_weapon(item_id) && !combat_art {
    0
  } else {
    requested_cost
  }
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn ordinary_weapon_attacks_are_free_at_both_bounds() {
    for id in [FIRST_WEAPON_ID, FIRST_WEAPON_ID + 1, LAST_WEAPON_ID] {
      assert!(is_weapon(id));
      assert_eq!(effective_cost(id, false, 1), 0);
      assert_eq!(effective_cost(id, false, 2), 0);
    }
  }

  #[test]
  fn combat_arts_keep_their_original_cost() {
    assert!(!is_combat_art(0xFFFF, 0));
    assert!(is_combat_art(0x03E8, 0));
    assert!(is_combat_art(0xFFFF, 0x17));
    assert!(is_combat_art(0x03E8, 0x17));
    assert_eq!(effective_cost(FIRST_WEAPON_ID, true, 1), 1);
    assert_eq!(effective_cost(FIRST_WEAPON_ID, true, 3), 3);
    assert_eq!(effective_cost(FIRST_WEAPON_ID, true, 9), 9);
  }

  #[test]
  fn other_item_costs_are_untouched() {
    for id in [0, FIRST_WEAPON_ID - 1, LAST_WEAPON_ID + 1, 0x03E8, 0x0FA0] {
      assert!(!is_weapon(id));
      assert_eq!(effective_cost(id, false, 1), 1);
      assert_eq!(effective_cost(id, true, 5), 5);
    }
  }

  #[test]
  fn zero_and_unusual_non_weapon_costs_are_forwarded() {
    assert_eq!(effective_cost(0x03E8, false, 0), 0);
    assert_eq!(effective_cost(0x03E8, false, -1), -1);
  }
}
