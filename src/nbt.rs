use std::{collections::BTreeMap, fmt::Display, io::{self, Read}};

use byteorder_lite::{BigEndian, ReadBytesExt};

use crate::snbt::Snbt as _;

// the nbt crates i find don't feel like they fit me, so im writing my own nbt parser and you can't stop me
// raw nbt -> nbt enum -> json or any other type maybe

// just for readability, so i can easily know what tag without having to use a lookup table
pub const TAG_END: u8 = 0x00;
pub const TAG_BYTE: u8 = 0x01;
pub const TAG_SHORT: u8 = 0x02;
pub const TAG_INT: u8 = 0x03;
pub const TAG_LONG: u8 = 0x04;
pub const TAG_FLOAT: u8 = 0x05;
pub const TAG_DOUBLE: u8 = 0x06;
pub const TAG_BYTE_ARRAY: u8 = 0x07;
pub const TAG_STRING: u8 = 0x08;
pub const TAG_LIST: u8 = 0x09;
pub const TAG_COMPOUND: u8 = 0x0A;
pub const TAG_INT_ARRAY: u8 = 0x0B;
pub const TAG_LONG_ARRAY: u8 = 0x0C;

// mainly used for the root tag, as it can have a name
#[derive(Clone, Debug)]
pub struct RootTag {
    name: String,
    data: TagData,
}

impl Display for RootTag {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        if self.name.is_empty() {
            self.data.to_snbt(f)
        } else {
            self.name.to_snbt(f)?;
            write!(f, ": {}", self.data)
        }
    }
}

impl RootTag {
    /// Takes any type that implements `std::io::Read` and tries to parse the nbt object.
    ///
    /// # Errors
    ///
    /// This function will return an error if the data is not valid nbt.
    pub fn from_raw<R: Read>(data: &mut R) -> io::Result<Self> {
        if let Some((name, data)) = Self::read_tag(data)? {
            Ok(Self { name, data })
        } else {
            Err(io::Error::other("Invalid tag type TAG_END found"))
        }
    }

    pub fn name(&self) -> &str {
        &self.name
    }

    pub fn tag(&self) -> u8 {
        self.data.tag()
    }

    pub fn tag_name(&self) -> &str {
        self.data.tag_name()
    }

    pub fn payload(&self) -> &Payload {
        self.data.payload()
    }

    /// The `Option` is only there for something else, all it means is that the tag type was `TAG_END`
    fn read_tag<R: Read>(data: &mut R) -> io::Result<Option<(String, TagData)>> {
        let tag = data.read_u8()?;

        if tag == TAG_END {
            return Ok(None);
        }

        let name = Self::read_string(data)?;

        let payload = Self::read_payload(data, tag)?;

        Ok(Some((name, (tag, payload).into())))
    }

    fn read_payload<R: Read>(data: &mut R, tag: u8) -> io::Result<Payload> {
        Ok(match tag {
            TAG_BYTE => Payload::Byte(data.read_i8()?),
            TAG_SHORT => Payload::Short(data.read_i16::<BigEndian>()?),
            TAG_INT => Payload::Int(data.read_i32::<BigEndian>()?),
            TAG_LONG => Payload::Long(data.read_i64::<BigEndian>()?),
            TAG_FLOAT => Payload::Float(data.read_f32::<BigEndian>()?),
            TAG_DOUBLE => Payload::Double(data.read_f64::<BigEndian>()?),
            TAG_BYTE_ARRAY => Payload::ByteArray(Self::read_array(data, ReadBytesExt::read_i8)?),
            TAG_STRING => Payload::String(Self::read_string(data)?),
            TAG_LIST => { // this shit is such a mess (update: even more of a mess)
                match data.read_u8()? { // tag ID of the list's contents
                    TAG_END => Payload::EmptyList,
                    TAG_BYTE => Payload::ByteList(Self::read_array(data, ReadBytesExt::read_i8)?),
                    TAG_SHORT => Payload::ShortList(Self::read_array(data, ReadBytesExt::read_i16::<BigEndian>)?),
                    TAG_INT => Payload::IntList(Self::read_array(data, ReadBytesExt::read_i32::<BigEndian>)?),
                    TAG_LONG => Payload::LongList(Self::read_array(data, ReadBytesExt::read_i64::<BigEndian>)?),
                    TAG_FLOAT => Payload::FloatList(Self::read_array(data, ReadBytesExt::read_f32::<BigEndian>)?),
                    TAG_DOUBLE => Payload::DoubleList(Self::read_array(data, ReadBytesExt::read_f64::<BigEndian>)?),
                    TAG_BYTE_ARRAY => Payload::ByteArrayList(Self::read_array(data, |r| Self::read_array(r, ReadBytesExt::read_i8))?),
                    TAG_STRING => Payload::StringList(Self::read_array(data, Self::read_string)?),
                    TAG_LIST => Payload::ListList(Self::read_array(data, |r| Self::read_payload(r, TAG_LIST))?),
                    TAG_COMPOUND => Payload::CompoundList(Self::read_array(data, Self::read_compound)?),
                    TAG_INT_ARRAY => Payload::IntArrayList(Self::read_array(data, |r| Self::read_array(r, ReadBytesExt::read_i32::<BigEndian>))?),
                    TAG_LONG_ARRAY => Payload::LongArrayList(Self::read_array(data, |r| Self::read_array(r, ReadBytesExt::read_i64::<BigEndian>))?),
                    v => return Err(io::Error::other(format!("Invalid list tag type {v:#X} found"))),
                }
            },
            TAG_COMPOUND => Payload::Compound(Self::read_compound(data)?),
            TAG_INT_ARRAY => Payload::IntArray(Self::read_array(data, ReadBytesExt::read_i32::<BigEndian>)?),
            TAG_LONG_ARRAY => Payload::LongArray(Self::read_array(data, ReadBytesExt::read_i64::<BigEndian>)?),
            v => return Err(io::Error::other(format!("Invalid tag type {v:#X} found"))),
        })
    }

    fn read_string<R: Read>(data: &mut R) -> io::Result<String> {
        let len = data.read_u16::<BigEndian>()?;

        if len > 0 {
            let mut buf = vec![0u8; len as usize];
            data.read_exact(&mut buf)?;
            Ok(String::from_utf8_lossy(&buf).to_string())
        } else {
            Ok(String::new())
        }
    }

    /// This method is absolutely stupid, but at least it allows for arrays of any number and string to be parsed,
    /// without me having to make like 7 different functions for each one with almost the exact same code in each.
    ///
    /// Doing it this way also feels better because the other way I can think of is a macro and I don't wanna make a macro.
    fn read_array<R: Read, F: Fn(&mut R) -> io::Result<T>, T>(data: &mut R, f: F) -> io::Result<Vec<T>> {
        let len = data.read_u32::<BigEndian>()? as usize;

        let mut array = Vec::with_capacity(len);

        for _ in 0..len {
            array.push(f(data)?);
        }

        Ok(array)
    }

    fn read_compound<R: Read>(data: &mut R) -> io::Result<BTreeMap<String, TagData>> {
        let mut map = BTreeMap::new();

        while let Some((name, data)) = Self::read_tag(data)? {
            map.insert(name, data);
        }

        Ok(map)
    }
}

#[derive(Clone, Debug)]
pub struct TagData {
    tag: u8, // store the original tag it was (mainly so byte array and list of byte can be differentiated)
    payload: Payload,
}

impl Display for TagData {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.to_snbt(f)
    }
}

impl From<(u8, Payload)> for TagData {
    fn from(value: (u8, Payload)) -> Self {
        Self { tag: value.0, payload: value.1 }
    }
}

impl TagData {
    pub fn tag(&self) -> u8 {
        self.tag
    }
    
    pub fn tag_name(&self) -> &str {
        match self.tag {
            TAG_END => "End",
            TAG_BYTE => "Byte",
            TAG_SHORT => "Short",
            TAG_INT => "Int",
            TAG_LONG => "Long",
            TAG_FLOAT => "Float",
            TAG_DOUBLE => "Double",
            TAG_BYTE_ARRAY => "Byte Array",
            TAG_STRING => "String",
            TAG_LIST => "List",
            TAG_COMPOUND => "Compound",
            TAG_INT_ARRAY => "Int Array",
            TAG_LONG_ARRAY => "Long Array",
            _ => "Unknown"
        }
    }

    pub fn payload(&self) -> &Payload {
        &self.payload
    }
}

/// `TAG_List` has been split to where there is an enum variant per tag type it can hold
/// while also being called an array because that just sounds better than calling it a list.
/// This is mainly there to save on memory, as a `Vec<i8>` is way less than a `Vec<Payload>`,
/// but there is still a `Vec<Payload>`, mainly for when a list holds more lists or compounds
#[derive(Clone, Debug)]
pub enum Payload {
    Byte(i8),
    Short(i16),
    Int(i32),
    Long(i64),
    Float(f32),
    Double(f64),
    String(String),
    Compound(BTreeMap<String, TagData>),

    // the basic array types nbt has, whatever
    ByteArray(Vec<i8>),
    IntArray(Vec<i32>),
    LongArray(Vec<i64>),

    // here is list, split into many different things so that i can save memory
    EmptyList, // mainly for when TAG_List holds TAG_End (which it can do for some reason)
    ByteList(Vec<i8>),
    ShortList(Vec<i16>),
    IntList(Vec<i32>),
    LongList(Vec<i64>),
    FloatList(Vec<f32>),
    DoubleList(Vec<f64>),
    StringList(Vec<String>),
    ByteArrayList(Vec<Vec<i8>>),
    IntArrayList(Vec<Vec<i32>>),
    LongArrayList(Vec<Vec<i64>>),
    ListList(Vec<Self>), // stupidest name but its a list of lists, it holds Self because otherwise there could be infinite enum variants
    CompoundList(Vec<BTreeMap<String, TagData>>),
}
