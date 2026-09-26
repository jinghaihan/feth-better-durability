use core::{ffi::c_void, ptr::NonNull};

use crate::{
  durability::effective_cost,
  game::{layout::Item, profile},
};

pub fn install() {
  if !matches_supported_game() {
    println!("[feth-infinite-weapon-durability] unsupported or modified game; hook not installed");
    return;
  }

  skyline::install_hooks!(battle_consume_hook);
  println!("[feth-infinite-weapon-durability] weapon durability loss disabled");
}

fn matches_supported_game() -> bool {
  if skyline::info::get_program_id() != profile::TITLE_ID {
    return false;
  }

  let mut version = skyline::nn::oe::DisplayVersion { name: [0; 16] };
  // SAFETY: Nintendo's API initializes the valid DisplayVersion out-pointer.
  unsafe {
    skyline::nn::oe::GetDisplayVersion(&mut version);
  }
  if !profile::is_supported_display_version(&version.name) {
    return false;
  }

  // SAFETY: Skyline supplies the loaded main NSO text base for this process.
  let text = unsafe { skyline::hooks::getRegionAddress(skyline::hooks::Region::Text) };
  let Some(text) = NonNull::new(text.cast::<u8>()) else {
    return false;
  };

  profile::matches_text_signatures(|offset| {
    // SAFETY: Each profile offset is aligned and inside the mapped 1.2.0 text
    // segment. The title and display-version checks ran before this read.
    unsafe { text.as_ptr().add(offset).cast::<u32>().read_volatile() }
  })
}

#[skyline::hook(offset = profile::BATTLE_CONSUME_OFFSET)]
fn battle_consume_hook(unit: *mut c_void, item: *mut Item, requested_cost: i32) -> i32 {
  let Some(item) = NonNull::new(item) else {
    return call_original!(unit, item, requested_cost);
  };

  // SAFETY: The hooked game function receives a live four-byte Item pointer.
  // Reading its ID does not mutate save-backed memory. Unaligned access also
  // tolerates a caller-provided packed inventory position.
  let item_id = unsafe { core::ptr::addr_of!((*item.as_ptr()).id).read_unaligned() };
  call_original!(unit, item.as_ptr(), effective_cost(item_id, requested_cost))
}
