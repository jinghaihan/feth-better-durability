/// The 1.2.0 executable indexes its weapon table with item IDs 10 through 509.
pub const FIRST_WEAPON_ID: u16 = 0x000A;
pub const LAST_WEAPON_ID: u16 = 0x01FD;

pub const fn is_weapon(item_id: u16) -> bool {
  item_id >= FIRST_WEAPON_ID && item_id <= LAST_WEAPON_ID
}

/// Preserve all non-weapon costs, including spell uses and consumable items.
pub const fn effective_cost(item_id: u16, requested_cost: i32) -> i32 {
  if is_weapon(item_id) {
    0
  } else {
    requested_cost
  }
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn weapon_cost_is_zero_at_both_bounds() {
    for id in [FIRST_WEAPON_ID, FIRST_WEAPON_ID + 1, LAST_WEAPON_ID] {
      assert!(is_weapon(id));
      assert_eq!(effective_cost(id, 1), 0);
      assert_eq!(effective_cost(id, 2), 0);
      assert_eq!(effective_cost(id, 5), 0);
    }
  }

  #[test]
  fn other_item_costs_are_untouched() {
    for id in [0, FIRST_WEAPON_ID - 1, LAST_WEAPON_ID + 1, 0x03E8, 0x0FA0] {
      assert!(!is_weapon(id));
      assert_eq!(effective_cost(id, 1), 1);
      assert_eq!(effective_cost(id, 5), 5);
    }
  }

  #[test]
  fn zero_and_unusual_non_weapon_costs_are_forwarded() {
    assert_eq!(effective_cost(0x03E8, 0), 0);
    assert_eq!(effective_cost(0x03E8, -1), -1);
  }
}
