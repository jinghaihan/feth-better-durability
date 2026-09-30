import struct
import unittest

from verify_nro import verify_caller_capture


class CallerCaptureTests(unittest.TestCase):
  def test_accepts_capture_before_calls(self):
    for register in range(31):
      with self.subTest(register=register):
        code = struct.pack("<III", 0xA9BF7BFD, 0xAA1E03E0 | register, 0x94000000)
        verify_caller_capture(code)

  def test_rejects_control_flow_before_capture(self):
    for branch in (
      0x94000000, 0x14000000, 0xD63F0000, 0xD61F0000, 0xD65F03C0,
      0x54000000, 0x34000000, 0x35000000, 0xB4000000, 0xB5000000,
      0x36000000, 0x37000000,
    ):
      with self.subTest(branch=hex(branch)):
        with self.assertRaises(ValueError):
          verify_caller_capture(struct.pack("<II", branch, 0xAA1E03F3))

  def test_rejects_missing_capture(self):
    with self.assertRaises(ValueError):
      verify_caller_capture(struct.pack("<I", 0xD503201F))

  def test_rejects_discarding_the_return_address(self):
    with self.assertRaises(ValueError):
      verify_caller_capture(struct.pack("<I", 0xAA1E03FF))


if __name__ == "__main__":
  unittest.main()
