/// The 1.2.0 executable indexes its weapon table with item IDs 10 through 509.
pub const FIRST_WEAPON_ID: u16 = 0x000A;
pub const LAST_WEAPON_ID: u16 = 0x01FD;

pub const fn is_weapon(item_id: u16) -> bool {
  item_id >= FIRST_WEAPON_ID && item_id <= LAST_WEAPON_ID
}

/// Only the verified ordinary-attack call site is eligible for free costs.
/// Action-record settlement and unknown callers retain the game's result.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum CostPath {
  OrdinaryAttack,
  AttackRecord,
  Unknown,
}

/// The verified ordinary path requests one or two uses. Do not suppress an
/// unexpected cost, even from that caller. Combat-art costs are not guessed
/// from action IDs or amounts, and are never recalculated by the plugin.
pub const fn effective_cost(item_id: u16, path: CostPath, requested_cost: i32) -> i32 {
  if is_weapon(item_id)
    && matches!(path, CostPath::OrdinaryAttack)
    && matches!(requested_cost, 1 | 2)
  {
    0
  } else {
    requested_cost
  }
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn logged_tempest_lance_cost_is_preserved() {
    // The 0.1.2 trace requested five uses from 0xC3114 for item 23, despite
    // action_id=0xFFFF and kind=0. Those fields cannot identify a normal attack.
    let path = crate::game::profile::cost_path(0xC3114);
    assert_eq!(path, CostPath::AttackRecord);
    assert_eq!(effective_cost(23, path, 5), 5);
  }

  #[test]
  fn ordinary_weapon_attacks_are_free_at_both_bounds() {
    for id in [FIRST_WEAPON_ID, FIRST_WEAPON_ID + 1, LAST_WEAPON_ID] {
      assert!(is_weapon(id));
      assert_eq!(effective_cost(id, CostPath::OrdinaryAttack, 1), 0);
      assert_eq!(effective_cost(id, CostPath::OrdinaryAttack, 2), 0);
    }
  }

  #[test]
  fn combat_arts_keep_their_original_cost() {
    for cost in [0, 1, 2, 3, 5, 9, 10, 40, 99, 100, -1] {
      assert_eq!(
        effective_cost(FIRST_WEAPON_ID, CostPath::AttackRecord, cost),
        cost
      );
    }
  }

  #[test]
  fn other_item_costs_are_untouched() {
    for id in [0, FIRST_WEAPON_ID - 1, LAST_WEAPON_ID + 1, 0x03E8, 0x0FA0] {
      assert!(!is_weapon(id));
      for path in [
        CostPath::OrdinaryAttack,
        CostPath::AttackRecord,
        CostPath::Unknown,
      ] {
        assert_eq!(effective_cost(id, path, 1), 1);
        assert_eq!(effective_cost(id, path, 5), 5);
      }
    }
  }

  #[test]
  fn unknown_weapon_callers_keep_even_one_use_costs() {
    for cost in [0, 1, 2, 5, 40, 99, -1] {
      assert_eq!(
        effective_cost(FIRST_WEAPON_ID, CostPath::Unknown, cost),
        cost
      );
    }
  }

  #[test]
  fn unexpected_ordinary_path_costs_are_forwarded() {
    for cost in [0, 3, 5, 40, 99, -1] {
      assert_eq!(
        effective_cost(FIRST_WEAPON_ID, CostPath::OrdinaryAttack, cost),
        cost
      );
    }
  }

  #[test]
  fn logged_ordinary_attack_cost_is_suppressed() {
    let path = crate::game::profile::cost_path(0xDD4F8);
    assert_eq!(effective_cost(16, path, 1), 0);
    assert_eq!(effective_cost(23, path, 1), 0);
    assert_eq!(effective_cost(23, path, 2), 0);
  }

  #[test]
  fn spell_uses_on_the_logged_ordinary_path_are_preserved() {
    let path = crate::game::profile::cost_path(0xDD4F8);
    assert_eq!(effective_cost(4003, path, 1), 1);
  }

  #[test]
  fn consecutive_paths_do_not_leak_attack_state() {
    for (caller, requested, expected) in [
      (0xDD4F8, 1, 0),
      (0xC3114, 5, 5),
      (0xDD4F8, 1, 0),
      (0xC3114, 1, 1),
      (0xDD3F8, 1, 1),
      (0xDD4F8, 2, 0),
    ] {
      assert_eq!(
        effective_cost(23, crate::game::profile::cost_path(caller), requested),
        expected
      );
    }
  }
}
