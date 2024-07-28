use std::io::{Cursor, Write, Read};

use byteorder::{LittleEndian, WriteBytesExt, ReadBytesExt};
use thiserror::Error;

#[derive(Error, Debug)]
pub enum SerializationError {
    #[error("IO error: {0}")]
    IoError(#[from] std::io::Error),
    #[error("Buffer overflow")]
    BufferOverflow,
}

pub struct Serializer {
    buffer: Vec<u8>,
}

impl Serializer {
    pub fn new() -> Self {
        Serializer { buffer: Vec::new() }
    }

    pub fn write<T: Serialize>(&mut self, value: &T) -> Result<(), SerializationError> {
        value.serialize(self)
    }

    pub fn data(&self) -> &[u8] {
        &self.buffer
    }
}

pub struct Deserializer<'a> {
    data: Cursor<&'a [u8]>,
}

impl<'a> Deserializer<'a> {
    pub fn new(data: &'a [u8]) -> Self {
        Deserializer { data: Cursor::new(data) }
    }

    pub fn read<T: Deserialize>(&mut self) -> Result<T, SerializationError> {
        T::deserialize(self)
    }

    pub fn has_more(&self) -> bool {
        self.data.position() < self.data.get_ref().len() as u64
    }
}

pub trait Serialize {
    fn serialize(&self, serializer: &mut Serializer) -> Result<(), SerializationError>;
}

pub trait Deserialize: Sized {
    fn deserialize(deserializer: &mut Deserializer) -> Result<Self, SerializationError>;
}

macro_rules! impl_serialize_primitive {
    ($t:ty, $writer:ident, $reader:ident) => {
        impl Serialize for $t {
            fn serialize(&self, serializer: &mut Serializer) -> Result<(), SerializationError> {
                serializer.buffer.$writer::<LittleEndian>(*self)?;
                Ok(())
            }
        }

        impl Deserialize for $t {
            fn deserialize(deserializer: &mut Deserializer) -> Result<Self, SerializationError> {
                Ok(deserializer.data.$reader::<LittleEndian>()?)
            }
        }
    };
}

impl_serialize_primitive!(i32, write_i32, read_i32);
impl_serialize_primitive!(i64, write_i64, read_i64);
impl_serialize_primitive!(f32, write_f32, read_f32);
impl_serialize_primitive!(f64, write_f64, read_f64);

// 为 bool 类型单独实现
impl Serialize for bool {
    fn serialize(&self, serializer: &mut Serializer) -> Result<(), SerializationError> {
        serializer.buffer.write_u8(*self as u8)?;
        Ok(())
    }
}

impl Deserialize for bool {
    fn deserialize(deserializer: &mut Deserializer) -> Result<Self, SerializationError> {
        Ok(deserializer.data.read_u8()? != 0)
    }
}

impl Serialize for String {
    fn serialize(&self, serializer: &mut Serializer) -> Result<(), SerializationError> {
        (self.len() as i32).serialize(serializer)?;
        serializer.buffer.write_all(self.as_bytes())?;
        Ok(())
    }
}

impl Deserialize for String {
    fn deserialize(deserializer: &mut Deserializer) -> Result<Self, SerializationError> {
        let length = i32::deserialize(deserializer)? as usize;
        let mut buffer = vec![0u8; length];
        deserializer.data.read_exact(&mut buffer)?;
        Ok(String::from_utf8_lossy(&buffer).into_owned())
    }
}