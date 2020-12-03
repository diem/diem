# Copyright (c) The Diem Core Contributors
# SPDX-License-Identifier: Apache-2.0

LENGTH = 16  # type: int

def to_bytes(self) -> bytes:
    """Convert account address to bytes."""
    return bytes(typing.cast(typing.Iterable[int], self.value))

@staticmethod
def from_bytes(addr: bytes) -> "AccountAddress":
    """Create an account address from bytes."""
    if len(addr) != AccountAddress.LENGTH:
        raise ValueError("Incorrect length for an account address")
    return AccountAddress(value=tuple(st.uint8(x) for x in addr))  # pyre-ignore

def to_hex(self) -> str:
    """Convert account address to an hexadecimal string."""
    return self.to_bytes().hex()

@staticmethod
def from_hex(addr: str) -> "AccountAddress":
    """Create an account address from an hexadecimal string."""
    return AccountAddress.from_bytes(bytes.fromhex(addr))
