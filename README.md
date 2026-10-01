# FETH Better Durability

Prevents ordinary attacks from consuming weapon durability in Fire Emblem:
Three Houses. Combat arts still consume their original durability cost.

[![build](https://github.com/jinghaihan/feth-better-durability/actions/workflows/build.yml/badge.svg)](https://github.com/jinghaihan/feth-better-durability/actions/workflows/build.yml)
[![license](https://img.shields.io/badge/license-MIT-blue.svg)](LICENSE)

## Requirements

- Fire Emblem: Three Houses 1.2.0 (Build ID `89048449BA238C8CF565518B83BF02D3`)
- A Fire Emblem: Three Houses Skyline loader, such as the one provided by
  [Aldebaran](https://github.com/three-houses-research-team/aldebaran-rs)

## Install

Back up your saves before installing the plugin.

### Nintendo Switch (Atmosphere)

Install the game-specific Skyline loader, then download the NRO or installable
ZIP from [Releases](https://github.com/jinghaihan/feth-better-durability/releases).
Copy `feth-better-durability.nro` to:

```text
sdmc:/atmosphere/contents/010055D009F78000/romfs/skyline/plugins/feth-better-durability.nro
```

Fully restart the game. The NRO can coexist with other Skyline plugins that
do not hook the same battle-consumption function. Remove the NRO to uninstall
it; the plugin does not write metadata into save files.

When upgrading from the previous project name, remove the old NRO from the
plugins directory before installing this one. Rename an existing diagnostic
configuration file to `feth-better-durability.cfg`.

### Emulators (Eden or Ryubing)

Install the same Skyline loader in the emulator's
Atmosphere mod directory, then merge the release ZIP into its emulated SD card.
The ZIP contains this plugin only, not the Skyline loader. Restart the emulator
completely before testing.

## Behavior

- Ordinary weapon attacks, including follow-ups and counters, do not consume
  durability. Combat arts keep their original durability cost, including arts
  that exhaust the entire weapon. Each weapon keeps its own original maximum;
  no weapon is rewritten to `100`.
- Spell uses, consumable items, and other non-weapon costs are unchanged.
- Only the verified ordinary-attack call site receives free weapon uses.
  Action-record settlement and unrecognized callers keep the game's original
  cost. The plugin does not infer combat arts from action IDs or cost amounts.
- A weapon that is already damaged stays damaged. This plugin does not repair
  old saves or alter the game's combat-art availability checks.
- The plugin checks the title, displayed version, and original code signatures
  before installing its hook. Unsupported or conflicting executable patches
  are left unchanged.

## Diagnostic log

Logging is off by default and does not affect normal gameplay. To enable it,
create this UTF-8 text file in the SD card root (or the emulator's virtual SD
card root):

```text
sdmc:/feth-better-durability.cfg
```

Its contents should be:

```ini
diagnostic_log=true
```

Fully restart the game after changing the file. When enabled, the plugin
appends to `sdmc:/feth-better-durability.log`. It records the game
version and hook checks, action-record fields and item durability, plus the
call path, requested cost, forwarded cost, and underlying durability changes.
Action-record fields are diagnostic only and do not control the cost policy.
It does not change the durability behavior of any newly observed call path.
Diagnostic logging can slow the game during the first 2,048 item-decrement
calls; after testing, remove the configuration file or set `diagnostic_log=false` and
restart. An absent or invalid configuration leaves logging disabled and does
not install the action-record or low-level diagnostic hooks.

## Build

```sh
cargo fmt --check
cargo test --locked
python3 -m unittest discover -s tools -p 'test_*.py'
cargo skyline check
cargo skyline build --release
python3 tools/verify_nro.py \
  target/aarch64-skyline-switch/release/libfeth_better_durability.nro \
  --elf target/aarch64-skyline-switch/release/libfeth_better_durability.so
```

The NRO is written to
`target/aarch64-skyline-switch/release/libfeth_better_durability.nro`.

## Credits

This is an independent implementation. These projects provided public
technical references or runtime tooling; their source and assets are not
included in this repository.

- [Aldebaran](https://github.com/three-houses-research-team/aldebaran-rs) —
  Fire Emblem: Three Houses Skyline loader.
- [FETH Overlays](https://github.com/3096/feth-overlays) — item layout and
  Fire Emblem: Three Houses 1.2.0 Build ID reference.
- [Progenitor](https://github.com/three-houses-research-team/Progenitor) —
  weapon data and combat-art durability-cost references.
- [skyline-rs](https://github.com/ultimate-research/skyline-rs) — plugin runtime.

Fire Emblem and related names are trademarks of Nintendo and Intelligent
Systems. This unofficial fan project is not affiliated with or endorsed by
them.

## License

[MIT](./LICENSE) License © [jinghaihan](https://github.com/jinghaihan)
