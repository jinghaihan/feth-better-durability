/// FE3H 1.2.0's four-byte inventory item. The held-item and convoy layouts
/// use the same ID and durability positions.
#[repr(C)]
pub struct Item {
  pub id: u16,
  pub durability: u8,
  pub amount: u8,
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn item_layout_matches_the_game() {
    assert_eq!(core::mem::size_of::<Item>(), 4);
    assert_eq!(core::mem::offset_of!(Item, id), 0);
    assert_eq!(core::mem::offset_of!(Item, durability), 2);
    assert_eq!(core::mem::offset_of!(Item, amount), 3);
  }
}
