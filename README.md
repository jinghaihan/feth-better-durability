# FETH Infinite Weapon Durability

Prevents weapons from losing durability in Fire Emblem: Three Houses.

> [!WARNING]
> This plugin builds successfully but has not been tested in-game. Back up your
> saves before trying it.

## Requirements

- Fire Emblem: Three Houses 1.2.0 (Build ID `89048449BA238C8CF565518B83BF02D3`)
- A Fire Emblem: Three Houses Skyline loader, such as the one provided by
  [Aldebaran](https://github.com/three-houses-research-team/aldebaran-rs)

## Install

Install the game-specific Skyline loader, then copy
`feth-infinite-weapon-durability.nro` to:

```text
sdmc:/atmosphere/contents/010055D009F78000/romfs/skyline/plugins/feth-infinite-weapon-durability.nro
```

Fully restart the game. The NRO can coexist with other Skyline plugins that
do not hook the same battle-consumption function. Remove the NRO to uninstall
it; the plugin does not write metadata into save files.

## Behavior

- Normal attacks and combat actions pass zero durability cost for weapons.
  Each weapon keeps its existing durability value and its own original maximum;
  no weapon is rewritten to `100`.
- Spell uses, consumable items, and other non-weapon costs are unchanged.
- A weapon that is already damaged stays damaged. This plugin does not repair
  old saves or alter the game's combat-art availability checks.
- The plugin checks the title, displayed version, and original code signatures
  before installing its hook. Unsupported or conflicting executable patches
  are left unchanged.

## Build

```sh
cargo fmt --check
cargo test --locked
cargo skyline check
cargo skyline build --release
```

The NRO is written to
`target/aarch64-skyline-switch/release/libfeth_infinite_weapon_durability.nro`.

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
