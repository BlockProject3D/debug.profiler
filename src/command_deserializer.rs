// Copyright (c) 2022, BlockProject 3D
//
// All rights reserved.
//
// Redistribution and use in source and binary forms, with or without modification,
// are permitted provided that the following conditions are met:
//
//     * Redistributions of source code must retain the above copyright notice,
//       this list of conditions and the following disclaimer.
//     * Redistributions in binary form must reproduce the above copyright notice,
//       this list of conditions and the following disclaimer in the documentation
//       and/or other materials provided with the distribution.
//     * Neither the name of BlockProject 3D nor the names of its contributors
//       may be used to endorse or promote products derived from this software
//       without specific prior written permission.
//
// THIS SOFTWARE IS PROVIDED BY THE COPYRIGHT HOLDERS AND CONTRIBUTORS
// "AS IS" AND ANY EXPRESS OR IMPLIED WARRANTIES, INCLUDING, BUT NOT
// LIMITED TO, THE IMPLIED WARRANTIES OF MERCHANTABILITY AND FITNESS FOR
// A PARTICULAR PURPOSE ARE DISCLAIMED. IN NO EVENT SHALL THE COPYRIGHT OWNER OR
// CONTRIBUTORS BE LIABLE FOR ANY DIRECT, INDIRECT, INCIDENTAL, SPECIAL,
// EXEMPLARY, OR CONSEQUENTIAL DAMAGES (INCLUDING, BUT NOT LIMITED TO,
// PROCUREMENT OF SUBSTITUTE GOODS OR SERVICES; LOSS OF USE, DATA, OR
// PROFITS; OR BUSINESS INTERRUPTION) HOWEVER CAUSED AND ON ANY THEORY OF
// LIABILITY, WHETHER IN CONTRACT, STRICT LIABILITY, OR TORT (INCLUDING
// NEGLIGENCE OR OTHERWISE) ARISING IN ANY WAY OUT OF THE USE OF THIS
// SOFTWARE, EVEN IF ADVISED OF THE POSSIBILITY OF SUCH DAMAGE.

use serde::de::{DeserializeSeed, EnumAccess, MapAccess, SeqAccess, VariantAccess, Visitor};
use serde::Deserializer;
use std::fmt::{Debug, Display, Formatter};
use std::iter::Peekable;
use std::str::Split;

#[derive(Debug)]
pub enum Error<'a> {
    Unsupported,
    UnexpectedEof,
    ExpectedEof,
    SyntaxError(&'a str),
    Other(String),
}

impl<'a> serde::de::StdError for Error<'a> {}

impl<'a> Display for Error<'a> {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            Error::Unsupported => f.write_str("Unsupported parsing operation"),
            Error::UnexpectedEof => f.write_str("Unexpected EOF"),
            Error::ExpectedEof => f.write_str("Expected EOF"),
            Error::SyntaxError(v) => write!(f, "Syntax error in argument '{}'", v),
            Error::Other(v) => write!(f, "Foreign error: {}", v),
        }
    }
}

impl<'a> serde::de::Error for Error<'a> {
    fn custom<T>(msg: T) -> Self
    where
        T: Display,
    {
        Error::Other(msg.to_string())
    }
}

pub struct CommandDeserializer<'de, T: Iterator<Item = &'de str>> {
    args: Peekable<T>,
}

impl<'de, T: Iterator<Item = &'de str>> CommandDeserializer<'de, T> {
    pub fn new(iter: T) -> CommandDeserializer<'de, T> {
        CommandDeserializer {
            args: iter.peekable(),
        }
    }
}

impl<'a, 'de, T: Iterator<Item = &'de str>> Deserializer<'de>
    for &'a mut CommandDeserializer<'de, T>
{
    type Error = Error<'de>;

    fn deserialize_any<V>(self, _: V) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        Err(Error::Unsupported)
    }

    fn deserialize_bool<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        let val = self.args.next().ok_or(Error::UnexpectedEof)?;
        if val == "true" || val == "1" || val == "on" {
            visitor.visit_bool(true)
        } else if val == "false" || val == "0" || val == "off" {
            visitor.visit_bool(false)
        } else {
            Err(Error::SyntaxError(val))
        }
    }

    fn deserialize_i8<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        let val = self.args.next().ok_or(Error::UnexpectedEof)?;
        visitor.visit_i8(val.parse().map_err(|_| Error::SyntaxError(val))?)
    }

    fn deserialize_i16<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        let val = self.args.next().ok_or(Error::UnexpectedEof)?;
        visitor.visit_i16(val.parse().map_err(|_| Error::SyntaxError(val))?)
    }

    fn deserialize_i32<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        let val = self.args.next().ok_or(Error::UnexpectedEof)?;
        visitor.visit_i32(val.parse().map_err(|_| Error::SyntaxError(val))?)
    }

    fn deserialize_i64<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        let val = self.args.next().ok_or(Error::UnexpectedEof)?;
        visitor.visit_i64(val.parse().map_err(|_| Error::SyntaxError(val))?)
    }

    fn deserialize_u8<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        let val = self.args.next().ok_or(Error::UnexpectedEof)?;
        visitor.visit_u8(val.parse().map_err(|_| Error::SyntaxError(val))?)
    }

    fn deserialize_u16<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        let val = self.args.next().ok_or(Error::UnexpectedEof)?;
        visitor.visit_u16(val.parse().map_err(|_| Error::SyntaxError(val))?)
    }

    fn deserialize_u32<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        let val = self.args.next().ok_or(Error::UnexpectedEof)?;
        visitor.visit_u32(val.parse().map_err(|_| Error::SyntaxError(val))?)
    }

    fn deserialize_u64<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        let val = self.args.next().ok_or(Error::UnexpectedEof)?;
        visitor.visit_u64(val.parse().map_err(|_| Error::SyntaxError(val))?)
    }

    fn deserialize_f32<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        let val = self.args.next().ok_or(Error::UnexpectedEof)?;
        visitor.visit_f32(val.parse().map_err(|_| Error::SyntaxError(val))?)
    }

    fn deserialize_f64<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        let val = self.args.next().ok_or(Error::UnexpectedEof)?;
        visitor.visit_f64(val.parse().map_err(|_| Error::SyntaxError(val))?)
    }

    fn deserialize_char<V>(self, _: V) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        Err(Error::Unsupported)
    }

    fn deserialize_str<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        let val = self.args.next().ok_or(Error::UnexpectedEof)?;
        visitor.visit_borrowed_str(val)
    }

    fn deserialize_string<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        let val = self.args.next().ok_or(Error::UnexpectedEof)?;
        visitor.visit_string(val.into())
    }

    fn deserialize_bytes<V>(self, _: V) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        Err(Error::Unsupported)
    }

    fn deserialize_byte_buf<V>(self, _: V) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        Err(Error::Unsupported)
    }

    fn deserialize_option<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        match self.args.peek() {
            None => visitor.visit_none(),
            Some(_) => visitor.visit_some(self),
        }
    }

    fn deserialize_unit<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        match self.args.next() {
            None => visitor.visit_unit(),
            Some(_) => Err(Error::ExpectedEof),
        }
    }

    fn deserialize_unit_struct<V>(
        self,
        _: &'static str,
        visitor: V,
    ) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        self.deserialize_unit(visitor)
    }

    fn deserialize_newtype_struct<V>(
        self,
        _: &'static str,
        visitor: V,
    ) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        visitor.visit_newtype_struct(self)
    }

    fn deserialize_seq<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        visitor.visit_seq(Sequence {
            de: self,
            size: usize::MAX,
        })
    }

    fn deserialize_tuple<V>(self, len: usize, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        visitor.visit_seq(Sequence {
            de: self,
            size: len,
        })
    }

    fn deserialize_tuple_struct<V>(
        self,
        _: &'static str,
        len: usize,
        visitor: V,
    ) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        self.deserialize_tuple(len, visitor)
    }

    fn deserialize_map<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        visitor.visit_map(Map {
            de: self,
            next: None,
            size: usize::MAX,
        })
    }

    fn deserialize_struct<V>(
        self,
        _: &'static str,
        fields: &'static [&'static str],
        visitor: V,
    ) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        visitor.visit_map(Map {
            de: self,
            next: None,
            size: fields.len(),
        })
    }

    fn deserialize_enum<V>(
        self,
        _: &'static str,
        _: &'static [&'static str],
        visitor: V,
    ) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        visitor.visit_enum(Enum { de: self })
    }

    fn deserialize_identifier<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        self.deserialize_str(visitor)
    }

    fn deserialize_ignored_any<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        self.args.next();
        visitor.visit_none()
    }
}

struct Map<'a, 'de, T: Iterator<Item = &'de str>> {
    de: &'a mut CommandDeserializer<'de, T>,
    next: Option<CommandDeserializer<'de, Split<'de, &'de str>>>,
    size: usize,
}

impl<'a, 'de: 'a, T: Iterator<Item = &'de str>> Map<'a, 'de, T> {
    fn get_next(&mut self) -> Option<&mut CommandDeserializer<'de, Split<'de, &'de str>>> {
        if (self.size == 0 || self.de.args.peek().is_none()) && self.next.is_none() {
            return None;
        }
        if self.next.is_some() {
            return self.next.as_mut();
        }
        let val = self.de.args.next()?;
        self.next = Some(CommandDeserializer::new(val.split("=")));
        self.size -= 1;
        self.next.as_mut()
    }
}

impl<'a, 'de: 'a, T: Iterator<Item = &'de str>> MapAccess<'de> for Map<'a, 'de, T> {
    type Error = Error<'de>;

    fn next_key_seed<K>(&mut self, seed: K) -> Result<Option<K::Value>, Self::Error>
    where
        K: DeserializeSeed<'de>,
    {
        match self.get_next() {
            None => Ok(None),
            Some(v) => {
                let val = seed.deserialize(v)?;
                Ok(Some(val))
            }
        }
    }

    fn next_value_seed<V>(&mut self, seed: V) -> Result<V::Value, Self::Error>
    where
        V: DeserializeSeed<'de>,
    {
        let val = seed.deserialize(self.get_next().ok_or(Error::UnexpectedEof)?);
        self.next = None;
        val
    }
}

struct Sequence<'a, 'de, T: Iterator<Item = &'de str>> {
    de: &'a mut CommandDeserializer<'de, T>,
    size: usize,
}

impl<'a, 'de: 'a, T: Iterator<Item = &'de str>> SeqAccess<'de> for Sequence<'a, 'de, T> {
    type Error = Error<'de>;

    fn next_element_seed<V>(&mut self, seed: V) -> Result<Option<V::Value>, Self::Error>
    where
        V: DeserializeSeed<'de>,
    {
        if self.size == 0 || self.de.args.peek().is_none() {
            return Ok(None);
        }
        self.size -= 1;
        seed.deserialize(&mut *self.de).map(Some)
    }
}

struct Enum<'a, 'de, T: Iterator<Item = &'de str>> {
    de: &'a mut CommandDeserializer<'de, T>,
}

impl<'a, 'de: 'a, T: Iterator<Item = &'de str>> EnumAccess<'de> for Enum<'a, 'de, T> {
    type Error = Error<'de>;
    type Variant = Self;

    fn variant_seed<V>(self, seed: V) -> Result<(V::Value, Self::Variant), Self::Error>
    where
        V: DeserializeSeed<'de>,
    {
        let val = seed.deserialize(&mut *self.de)?;
        Ok((val, self))
    }
}

impl<'a, 'de: 'a, T: Iterator<Item = &'de str>> VariantAccess<'de> for Enum<'a, 'de, T> {
    type Error = Error<'de>;

    fn unit_variant(self) -> Result<(), Self::Error> {
        Ok(())
    }

    fn newtype_variant_seed<V>(self, seed: V) -> Result<V::Value, Self::Error>
    where
        V: DeserializeSeed<'de>,
    {
        seed.deserialize(self.de)
    }

    fn tuple_variant<V>(self, len: usize, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        self.de.deserialize_tuple(len, visitor)
    }

    fn struct_variant<V>(
        self,
        fields: &'static [&'static str],
        visitor: V,
    ) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        self.de.deserialize_struct("", fields, visitor)
    }
}
