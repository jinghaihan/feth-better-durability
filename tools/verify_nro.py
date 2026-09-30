#!/usr/bin/env python3

import argparse
import struct
from pathlib import Path


def verify_caller_capture(code: bytes) -> None:
  for offset in range(0, len(code) - 3, 4):
    word = read_u32(code, offset)
    if word & 0xFFFFFFE0 == 0xAA1E03E0 and word & 0x1F != 0x1F:
      return  # MOV Xn, X30 captures the incoming return address.
    if (
      word & 0x7C000000 == 0x14000000
      or word & 0xFFFFFC1F in (0xD61F0000, 0xD63F0000, 0xD65F0000)
      or word & 0xFF000010 == 0x54000000
      or word & 0x7E000000 in (0x34000000, 0x36000000)
    ):
      raise ValueError("hook branches before capturing the incoming X30")
  raise ValueError("hook does not capture the incoming X30")


def verify_hook_entries(path: Path) -> None:
  data = path.read_bytes()
  if data[:6] != b"\x7fELF\x02\x01" or struct.unpack_from("<H", data, 0x12)[0] != 183:
    raise ValueError("expected a little-endian AArch64 ELF")
  section_offset = struct.unpack_from("<Q", data, 0x28)[0]
  entry_size, section_count = struct.unpack_from("<HH", data, 0x3A)
  sections = [
    struct.unpack_from("<IIQQQQIIQQ", data, section_offset + index * entry_size)
    for index in range(section_count)
  ]
  required = {"battle_consume_hook", "attack_cost_hook", "raw_durability_hook"}
  checked = set()
  for section in sections:
    if section[1] != 2:
      continue
    strings = sections[section[6]]
    names = data[strings[4]:strings[4] + strings[5]]
    for offset in range(section[4], section[4] + section[5], section[9]):
      name_offset, _, _, index, address, size = struct.unpack_from("<IBBHQQ", data, offset)
      name = names[name_offset:].split(b"\0", 1)[0].decode("utf-8")
      if name not in required or index == 0:
        continue
      text = sections[index]
      start = text[4] + address - text[3]
      if size == 0 or start < text[4] or start + size > text[4] + text[5]:
        raise ValueError(f"invalid hook code range: {name}")
      verify_caller_capture(data[start:start + size])
      checked.add(name)
  if checked != required:
    raise ValueError(f"missing hook symbols: {sorted(required - checked)}")
  print(f"verified incoming caller capture in {path}")


def read_u32(data: bytes, offset: int) -> int:
  return struct.unpack_from("<I", data, offset)[0]


def verify_nro(path: Path) -> None:
  data = path.read_bytes()
  if len(data) < 0x80 or data[0x10:0x14] != b"NRO0":
    raise ValueError("NRO0 header is missing")
  if read_u32(data, 0x18) != len(data):
    raise ValueError("declared NRO size does not match the file")

  previous_end = 0
  for name, header_offset in (("text", 0x20), ("rodata", 0x28), ("data", 0x30)):
    offset, size = struct.unpack_from("<II", data, header_offset)
    if size == 0 or offset < previous_end or offset + size > len(data):
      raise ValueError(f"invalid {name} segment")
    previous_end = offset + size

  mod_offset = read_u32(data, 0x04)
  if mod_offset + 4 > len(data) or data[mod_offset:mod_offset + 4] != b"MOD0":
    raise ValueError("MOD0 header is missing")
  if not any(data[0x40:0x60]):
    raise ValueError("NRO Build ID is empty")

  print(f"verified {path} ({len(data)} bytes)")


def main() -> None:
  parser = argparse.ArgumentParser(description="Verify a Skyline NRO artifact")
  parser.add_argument("nro", type=Path)
  parser.add_argument("--elf", type=Path, help="also verify hook entry caller capture")
  args = parser.parse_args()
  verify_nro(args.nro)
  if args.elf is not None:
    verify_hook_entries(args.elf)


if __name__ == "__main__":
  main()
