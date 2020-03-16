// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::error::{Error, Result};
use serde::de::{self, Deserialize, DeserializeSeed, IntoDeserializer, Visitor};
use std::convert::TryFrom;

/// Deserializes a `&[u8]` into a type.
///
/// This function will attempt to interpret `bytes` as the LCS serialized form of `T` and
/// deserialize `T` from `bytes`.
///
/// # Examples
///
/// ```
/// use libra_canonical_serialization::from_bytes;
/// use serde::Deserialize;
///
/// #[derive(Deserialize)]
/// struct Ip([u8; 4]);
///
/// #[derive(Deserialize)]
/// struct Port(u16);
///
/// #[derive(Deserialize)]
/// struct SocketAddr {
///     ip: Ip,
///     port: Port,
/// }
///
/// let bytes = vec![0x7f, 0x00, 0x00, 0x01, 0x41, 0x1f];
/// let socket_addr: SocketAddr = from_bytes(&bytes).unwrap();
///
/// assert_eq!(socket_addr.ip.0, [127, 0, 0, 1]);
/// assert_eq!(socket_addr.port.0, 8001);
/// ```
pub fn from_bytes<'a, T>(bytes: &'a [u8]) -> Result<T>
where
    T: Deserialize<'a>,
{
    let mut deserializer = Deserializer::new(bytes);
    let t = T::deserialize(&mut deserializer)?;
    deserializer.end().map(move |_| t)
}

/// Perform a stateful deserialization from a `&[u8]` using the provided `seed`.
pub fn from_bytes_seed<'a, T>(seed: T, bytes: &'a [u8]) -> Result<T::Value>
where
    T: DeserializeSeed<'a>,
{
    let mut deserializer = Deserializer::new(bytes);
    let t = seed.deserialize(&mut deserializer)?;
    deserializer.end().map(move |_| t)
}

/// Deserialization implementation for LCS
struct Deserializer<'de> {
    input: &'de [u8],
}

impl<'de> Deserializer<'de> {
    /// Creates a new `Deserializer` which will be deserializing the provided
    /// input.
    fn new(input: &'de [u8]) -> Self {
        Deserializer { input }
    }

    /// The `Deserializer::end` method should be called after a type has been
    /// fully deserialized. This allows the `Deserializer` to validate that
    /// the there are no more bytes remaining in the input stream.
    fn end(&mut self) -> Result<()> {
        if self.input.is_empty() {
            Ok(())
        } else {
            Err(Error::RemainingInput)
        }
    }
}

impl<'de> Deserializer<'de> {
    fn peek(&mut self) -> Result<u8> {
        self.input.first().copied().ok_or(Error::Eof)
    }

    fn next(&mut self) -> Result<u8> {
        let byte = self.peek()?;
        self.input = &self.input[1..];
        Ok(byte)
    }

    fn parse_bool(&mut self) -> Result<bool> {
        let byte = self.next()?;

        match byte {
            0 => Ok(false),
            1 => Ok(true),
            _ => Err(Error::ExpectedBoolean),
        }
    }

    fn fill_slice(&mut self, slice: &mut [u8]) -> Result<()> {
        for byte in slice {
            *byte = self.next()?;
        }
        Ok(())
    }

    fn parse_u8(&mut self) -> Result<u8> {
        self.next()
    }

    fn parse_u16(&mut self) -> Result<u16> {
        let mut le_bytes = [0; 2];
        self.fill_slice(&mut le_bytes)?;
        Ok(u16::from_le_bytes(le_bytes))
    }

    fn parse_u32(&mut self) -> Result<u32> {
        let mut le_bytes = [0; 4];
        self.fill_slice(&mut le_bytes)?;
        Ok(u32::from_le_bytes(le_bytes))
    }

    fn parse_u64(&mut self) -> Result<u64> {
        let mut le_bytes = [0; 8];
        self.fill_slice(&mut le_bytes)?;
        Ok(u64::from_le_bytes(le_bytes))
    }

    fn parse_u128(&mut self) -> Result<u128> {
        let mut le_bytes = [0; 16];
        self.fill_slice(&mut le_bytes)?;
        Ok(u128::from_le_bytes(le_bytes))
    }

    fn parse_u32_from_uleb128(&mut self) -> Result<u32> {
        let mut value: u64 = 0;
        for shift in (0..32).step_by(7) {
            let byte = self.next()?;
            let digit = byte & 0x7f;
            value |= u64::from(digit) << shift;
            // If the highest bit of `byte` is 0, return the final value.
            if digit == byte {
                if shift > 0 && digit == 0 {
                    // We only accept canonical ULEB128 encodings, therefore the
                    // heaviest (and last) base-128 digit must be non-zero.
                    return Err(Error::NonCanonicalUleb128Encoding);
                }
                // Decoded integer must not overflow.
                return u32::try_from(value)
                    .map_err(|_| Error::IntegerOverflowDuringUleb128Decoding);
            }
        }
        // Decoded integer must not overflow.
        Err(Error::IntegerOverflowDuringUleb128Decoding)
    }

    fn parse_bytes(&mut self) -> Result<&'de [u8]> {
        let len = self.parse_u32_from_uleb128()? as usize;
        if len > crate::MAX_SEQUENCE_LENGTH {
            return Err(Error::ExceededMaxLen(len));
        }

        let slice = self.input.get(..len).ok_or(Error::Eof)?;
        self.input = &self.input[len..];
        Ok(slice)
    }

    fn parse_string(&mut self) -> Result<&'de str> {
        let slice = self.parse_bytes()?;
        std::str::from_utf8(slice).map_err(|_| Error::Utf8)
    }
}

impl<'de, 'a> de::Deserializer<'de> for &'a mut Deserializer<'de> {
    type Error = Error;

    // LCS is not a self-describing format so we can't implement `deserialize_any`
    fn deserialize_any<V>(self, _visitor: V) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        Err(Error::NotSupported("deserialize_any"))
    }

    fn deserialize_bool<V>(self, visitor: V) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        visitor.visit_bool(self.parse_bool()?)
    }

    fn deserialize_i8<V>(self, visitor: V) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        visitor.visit_i8(self.parse_u8()? as i8)
    }

    fn deserialize_i16<V>(self, visitor: V) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        visitor.visit_i16(self.parse_u16()? as i16)
    }

    fn deserialize_i32<V>(self, visitor: V) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        visitor.visit_i32(self.parse_u32()? as i32)
    }

    fn deserialize_i64<V>(self, visitor: V) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        visitor.visit_i64(self.parse_u64()? as i64)
    }

    fn deserialize_i128<V>(self, visitor: V) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        visitor.visit_i128(self.parse_u128()? as i128)
    }

    fn deserialize_u8<V>(self, visitor: V) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        visitor.visit_u8(self.parse_u8()?)
    }

    fn deserialize_u16<V>(self, visitor: V) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        visitor.visit_u16(self.parse_u16()?)
    }

    fn deserialize_u32<V>(self, visitor: V) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        visitor.visit_u32(self.parse_u32()?)
    }

    fn deserialize_u64<V>(self, visitor: V) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        visitor.visit_u64(self.parse_u64()?)
    }

    fn deserialize_u128<V>(self, visitor: V) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        visitor.visit_u128(self.parse_u128()?)
    }

    fn deserialize_f32<V>(self, _visitor: V) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        Err(Error::NotSupported("deserialize_f32"))
    }

    fn deserialize_f64<V>(self, _visitor: V) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        Err(Error::NotSupported("deserialize_f64"))
    }

    fn deserialize_char<V>(self, _visitor: V) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        Err(Error::NotSupported("deserialize_char"))
    }

    fn deserialize_str<V>(self, visitor: V) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        visitor.visit_borrowed_str(self.parse_string()?)
    }

    fn deserialize_string<V>(self, visitor: V) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        self.deserialize_str(visitor)
    }

    fn deserialize_bytes<V>(self, visitor: V) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        visitor.visit_borrowed_bytes(self.parse_bytes()?)
    }

    fn deserialize_byte_buf<V>(self, visitor: V) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        self.deserialize_bytes(visitor)
    }

    fn deserialize_option<V>(self, visitor: V) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        let byte = self.next()?;

        match byte {
            0 => visitor.visit_none(),
            1 => visitor.visit_some(self),
            _ => Err(Error::ExpectedOption),
        }
    }

    fn deserialize_unit<V>(self, visitor: V) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        visitor.visit_unit()
    }

    fn deserialize_unit_struct<V>(self, _name: &'static str, visitor: V) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        self.deserialize_unit(visitor)
    }

    fn deserialize_newtype_struct<V>(self, _name: &'static str, visitor: V) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        visitor.visit_newtype_struct(self)
    }

    fn deserialize_seq<V>(mut self, visitor: V) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        let len = self.parse_u32_from_uleb128()? as usize;
        if len > crate::MAX_SEQUENCE_LENGTH {
            return Err(Error::ExceededMaxLen(len));
        }

        visitor.visit_seq(SeqDeserializer::new(&mut self, len))
    }

    fn deserialize_tuple<V>(mut self, len: usize, visitor: V) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        visitor.visit_seq(SeqDeserializer::new(&mut self, len))
    }

    fn deserialize_tuple_struct<V>(
        mut self,
        _name: &'static str,
        len: usize,
        visitor: V,
    ) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        visitor.visit_seq(SeqDeserializer::new(&mut self, len))
    }

    fn deserialize_map<V>(mut self, visitor: V) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        let len = self.parse_u32_from_uleb128()? as usize;
        if len > crate::MAX_SEQUENCE_LENGTH {
            return Err(Error::ExceededMaxLen(len));
        }

        visitor.visit_map(MapDeserializer::new(&mut self, len))
    }

    fn deserialize_struct<V>(
        mut self,
        _name: &'static str,
        fields: &'static [&'static str],
        visitor: V,
    ) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        visitor.visit_seq(SeqDeserializer::new(&mut self, fields.len()))
    }

    fn deserialize_enum<V>(
        self,
        _name: &'static str,
        _variants: &'static [&'static str],
        visitor: V,
    ) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        visitor.visit_enum(self)
    }

    // LCS does not utilize identifiers, so throw them away
    fn deserialize_identifier<V>(self, _visitor: V) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        self.deserialize_bytes(_visitor)
    }

    // LCS is not a self-describing format so we can't implement `deserialize_ignored_any`
    fn deserialize_ignored_any<V>(self, _visitor: V) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        Err(Error::NotSupported("deserialize_ignored_any"))
    }

    // LCS is not a human readable format
    fn is_human_readable(&self) -> bool {
        false
    }
}

struct SeqDeserializer<'a, 'de: 'a> {
    de: &'a mut Deserializer<'de>,
    remaining: usize,
}

impl<'a, 'de> SeqDeserializer<'a, 'de> {
    fn new(de: &'a mut Deserializer<'de>, remaining: usize) -> Self {
        Self { de, remaining }
    }
}

impl<'de, 'a> de::SeqAccess<'de> for SeqDeserializer<'a, 'de> {
    type Error = Error;

    fn next_element_seed<T>(&mut self, seed: T) -> Result<Option<T::Value>>
    where
        T: DeserializeSeed<'de>,
    {
        if self.remaining == 0 {
            Ok(None)
        } else {
            self.remaining -= 1;
            seed.deserialize(&mut *self.de).map(Some)
        }
    }

    fn size_hint(&self) -> Option<usize> {
        Some(self.remaining)
    }
}

struct MapDeserializer<'a, 'de: 'a> {
    de: &'a mut Deserializer<'de>,
    remaining: usize,
    previous_key_bytes: Option<&'a [u8]>,
}

impl<'a, 'de> MapDeserializer<'a, 'de> {
    fn new(de: &'a mut Deserializer<'de>, remaining: usize) -> Self {
        Self {
            de,
            remaining,
            previous_key_bytes: None,
        }
    }
}

impl<'de, 'a> de::MapAccess<'de> for MapDeserializer<'a, 'de> {
    type Error = Error;

    fn next_key_seed<K>(&mut self, seed: K) -> Result<Option<K::Value>>
    where
        K: DeserializeSeed<'de>,
    {
        if self.remaining == 0 {
            Ok(None)
        } else {
            let previous_input_slice = &self.de.input[..];
            let key_value = seed.deserialize(&mut *self.de)?;
            let key_len = previous_input_slice.len() - self.de.input.len();
            let key_bytes = &previous_input_slice[0..key_len];
            if let Some(previous_key_bytes) = self.previous_key_bytes {
                if previous_key_bytes >= key_bytes {
                    return Err(Error::NonCanonicalMap);
                }
            }
            self.remaining -= 1;
            self.previous_key_bytes = Some(key_bytes);
            Ok(Some(key_value))
        }
    }

    fn next_value_seed<V>(&mut self, seed: V) -> Result<V::Value>
    where
        V: DeserializeSeed<'de>,
    {
        seed.deserialize(&mut *self.de)
    }

    fn size_hint(&self) -> Option<usize> {
        Some(self.remaining)
    }
}

impl<'de, 'a> de::EnumAccess<'de> for &'a mut Deserializer<'de> {
    type Error = Error;
    type Variant = Self;

    fn variant_seed<V>(self, seed: V) -> Result<(V::Value, Self::Variant)>
    where
        V: DeserializeSeed<'de>,
    {
        let variant_index = self.parse_u32_from_uleb128()?;
        let value = seed.deserialize(variant_index.into_deserializer())?;
        Ok((value, self))
    }
}

impl<'de, 'a> de::VariantAccess<'de> for &'a mut Deserializer<'de> {
    type Error = Error;

    fn unit_variant(self) -> Result<()> {
        Ok(())
    }

    fn newtype_variant_seed<T>(self, seed: T) -> Result<T::Value>
    where
        T: DeserializeSeed<'de>,
    {
        seed.deserialize(self)
    }

    fn tuple_variant<V>(self, len: usize, visitor: V) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        de::Deserializer::deserialize_tuple(self, len, visitor)
    }

    fn struct_variant<V>(self, fields: &'static [&'static str], visitor: V) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        de::Deserializer::deserialize_tuple(self, fields.len(), visitor)
    }
}
