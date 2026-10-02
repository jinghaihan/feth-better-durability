use core::{cell::Cell, ffi::c_void, ptr::NonNull};

use crate::{
  diagnostics,
  durability::effective_cost,
  game::{layout::Item, profile},
};

std::thread_local! {
  static ATTACK_CONTEXT: Cell<Option<(u16, u8, u8)>> = const { Cell::new(None) };
  static IN_BATTLE_CONSUME: Cell<bool> = const { Cell::new(false) };
}

pub fn install() {
  let diagnostic = diagnostics::init();
  diagnostics::append(concat!("plugin_version=", env!("CARGO_PKG_VERSION")));
  if !matches_supported_game() {
    diagnostics::append("hooks=not_installed");
    println!("[feth-better-durability] unsupported or modified game; hook not installed");
    return;
  }

  skyline::install_hooks!(battle_consume_hook);
  if diagnostic {
    skyline::install_hooks!(attack_cost_hook, raw_durability_hook);
    diagnostics::append(
      "hooks=installed battle_consume=0xC2E50 attack_cost=0xC30A0 raw_durability=0x40CAC0",
    );
    diagnostics::append(
      "cost_policy=verified_ordinary_callsite_only ordinary_caller=0xDD4F8 record_caller=0xC3114",
    );
  }
  println!("[feth-better-durability] ordinary weapon attacks do not consume durability");
}

fn matches_supported_game() -> bool {
  let program_id = skyline::info::get_program_id();
  diagnostics::append(&format!(
    "program_id={program_id:#018x} expected={:#018x}",
    profile::TITLE_ID
  ));
  if program_id != profile::TITLE_ID {
    return false;
  }

  let mut version = skyline::nn::oe::DisplayVersion { name: [0; 16] };
  // SAFETY: Nintendo's API initializes the valid DisplayVersion out-pointer.
  unsafe {
    skyline::nn::oe::GetDisplayVersion(&mut version);
  }
  diagnostics::append(&format!("display_version={:?}", version.name));
  if !profile::is_supported_display_version(&version.name) {
    return false;
  }

  // SAFETY: Skyline supplies the loaded main NSO text base for this process.
  let text = unsafe { skyline::hooks::getRegionAddress(skyline::hooks::Region::Text) };
  let Some(text) = NonNull::new(text.cast::<u8>()) else {
    diagnostics::append("text_region=null");
    return false;
  };

  for (offset, expected) in profile::TEXT_SIGNATURES {
    // SAFETY: Each profile offset is aligned and inside the mapped 1.2.0 text
    // segment. The title and display-version checks ran before this read.
    let actual = unsafe { text.as_ptr().add(offset).cast::<u32>().read_volatile() };
    if actual != expected {
      diagnostics::append(&format!(
        "signature_mismatch offset={offset:#x} expected={expected:#010x} actual={actual:#010x}"
      ));
      return false;
    }
  }
  diagnostics::append("signatures=matched");
  true
}

#[skyline::hook(offset = profile::ATTACK_COST_OFFSET)]
fn attack_cost_hook(
  unit: *mut c_void,
  item: *mut Item,
  attack_record: *const u8,
  doubled_cost: i32,
) -> i32 {
  let return_address: usize;
  // SAFETY: The hook receives the game's return address in X30. The compiler
  // output is checked so it is captured before any calls can replace it.
  unsafe {
    core::arch::asm!("mov {}, x30", out(reg) return_address, options(nomem, nostack, preserves_flags));
  }
  let log = diagnostics::enabled();
  let Some(record) = NonNull::new(attack_record.cast_mut()) else {
    if log {
      diagnostics::append(&format!(
        "attack_cost record=null item={item:p} doubled_cost={doubled_cost}"
      ));
    }
    return call_original!(unit, item, attack_record, doubled_cost);
  };

  // SAFETY: The original function reads these fields from the live record.
  // They are diagnostic data only: 0xFFFF also occurs for combat arts.
  let action_id = unsafe { record.as_ptr().cast::<u16>().read_unaligned() };
  let kind = unsafe { record.as_ptr().add(7).read() };
  let base_cost = unsafe { record.as_ptr().add(0xA).read() };
  let before = if log && !item.is_null() {
    Some(unsafe { core::ptr::addr_of!((*item).durability).read_unaligned() })
  } else {
    None
  };

  if log {
    let caller = caller_offset(return_address);
    diagnostics::append(&format!(
      "attack_cost enter caller={caller:#x} item={item:p} action_id={action_id:#06x} kind={kind:#04x} base_cost={base_cost} doubled_cost={doubled_cost}"
    ));
  }

  ATTACK_CONTEXT.with(|context| {
    let previous_context = context.replace(Some((action_id, kind, base_cost)));
    let result = call_original!(unit, item, attack_record, doubled_cost);
    context.set(previous_context);
    if log {
      let after =
        before.map(|_| unsafe { core::ptr::addr_of!((*item).durability).read_unaligned() });
      diagnostics::append(&format!(
        "attack_cost exit item={item:p} durability={before:?}->{after:?} result={result}"
      ));
    }
    result
  })
}

#[skyline::hook(offset = profile::BATTLE_CONSUME_OFFSET)]
fn battle_consume_hook(unit: *mut c_void, item: *mut Item, requested_cost: i32) -> i32 {
  let return_address: usize;
  unsafe {
    core::arch::asm!("mov {}, x30", out(reg) return_address, options(nomem, nostack, preserves_flags));
  }
  let log = diagnostics::enabled();
  let Some(item) = NonNull::new(item) else {
    if log {
      diagnostics::append(&format!(
        "battle_consume item=null requested_cost={requested_cost}"
      ));
    }
    return call_original!(unit, item, requested_cost);
  };

  // SAFETY: The hooked game function receives a live four-byte Item pointer.
  // Reading its ID does not mutate save-backed memory. Unaligned access also
  // tolerates a caller-provided packed inventory position.
  let item_id = unsafe { core::ptr::addr_of!((*item.as_ptr()).id).read_unaligned() };
  let before = if log {
    Some(unsafe { core::ptr::addr_of!((*item.as_ptr()).durability).read_unaligned() })
  } else {
    None
  };
  let caller = caller_offset(return_address);
  let path = profile::cost_path(caller);
  let forwarded_cost = effective_cost(item_id, path, requested_cost);
  if log {
    diagnostics::append(&format!(
      "battle_consume enter caller={caller:#x} item={:p} id={item_id} durability={before:?} path={path:?} requested_cost={requested_cost} forwarded_cost={forwarded_cost}",
      item.as_ptr()
    ));
  }
  let result = if diagnostics::enabled() {
    IN_BATTLE_CONSUME.with(|context| {
      let previous = context.replace(true);
      let result = call_original!(unit, item.as_ptr(), forwarded_cost);
      context.set(previous);
      result
    })
  } else {
    call_original!(unit, item.as_ptr(), forwarded_cost)
  };
  if log {
    let after = unsafe { core::ptr::addr_of!((*item.as_ptr()).durability).read_unaligned() };
    diagnostics::append(&format!(
      "battle_consume exit item={:p} id={item_id} durability={before:?}->{after} result={result}",
      item.as_ptr()
    ));
  }
  result
}

/// The game's four-byte item durability decrement. Other direct callers
/// bypass battle_consume_hook; trace them without changing their behavior.
#[skyline::hook(offset = 0x40CAC0)]
fn raw_durability_hook(item: *mut Item, cost: i32) -> *mut Item {
  let return_address: usize;
  unsafe {
    core::arch::asm!("mov {}, x30", out(reg) return_address, options(nomem, nostack, preserves_flags));
  }
  let Some(item) = NonNull::new(item) else {
    return call_original!(item, cost);
  };
  // Deliberately do not filter IDs; the assumed weapon ID range is one of
  // the things this diagnostic should be able to disprove.
  let item_id = unsafe { core::ptr::addr_of!((*item.as_ptr()).id).read_unaligned() };
  let log = diagnostics::enabled();
  let before = if log {
    unsafe { core::ptr::addr_of!((*item.as_ptr()).durability).read_unaligned() }
  } else {
    0
  };
  let attack_context = if log {
    ATTACK_CONTEXT.with(Cell::get)
  } else {
    None
  };
  let battle_consume_context = if log {
    IN_BATTLE_CONSUME.with(Cell::get)
  } else {
    false
  };
  let result = call_original!(item.as_ptr(), cost);
  if log {
    let after = unsafe { core::ptr::addr_of!((*item.as_ptr()).durability).read_unaligned() };
    let caller = caller_offset(return_address);
    diagnostics::append(&format!(
      "raw_decrement caller={caller:#x} lr={return_address:#x} item={:p} id={item_id} durability={before}->{after} cost={cost} attack_context={attack_context:?} battle_consume_context={battle_consume_context}",
      item.as_ptr()
    ));
  }
  result
}

fn caller_offset(return_address: usize) -> usize {
  let text = unsafe { skyline::hooks::getRegionAddress(skyline::hooks::Region::Text) } as usize;
  return_address.wrapping_sub(text).wrapping_sub(4)
}
