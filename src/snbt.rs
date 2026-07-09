use std::{collections::BTreeMap, fmt::{self, Write}};

use crate::nbt::{Payload, TAG_LIST, TagData};

/// Provides conversion to SNBT for certain types.
/// Can directly convert NBT into SNBT as well.
pub trait Snbt {
    /// Can be used on any type implementing `std::fmt::Write`, appending its data rather than overwriting.
    /// If you just want the SNBT data, consider creating a new `String` object and passing that in.
    /// Also takes a bool for whether it should print pretty text or not (where compounds will be spaced across multiple lines)
    fn to_snbt<W: Write>(&self, w: &mut W) -> fmt::Result;
}

impl Snbt for bool {
    fn to_snbt<W: Write>(&self, w: &mut W) -> fmt::Result {
        write!(w, "{self}")
    }
}

impl Snbt for i8 {
    fn to_snbt<W: Write>(&self, w: &mut W) -> fmt::Result {
        write!(w, "{self}b")
    }
}

impl Snbt for u8 {
    fn to_snbt<W: Write>(&self, w: &mut W) -> fmt::Result {
        write!(w, "{self}sB")
    }
}

impl Snbt for i16 {
    fn to_snbt<W: Write>(&self, w: &mut W) -> fmt::Result {
        write!(w, "{self}s")
    }
}

impl Snbt for u16 {
    fn to_snbt<W: Write>(&self, w: &mut W) -> fmt::Result {
        write!(w, "{self}sS")
    }
}

impl Snbt for i32 {
    fn to_snbt<W: Write>(&self, w: &mut W) -> fmt::Result {
        write!(w, "{self}")
    }
}

impl Snbt for u32 {
    fn to_snbt<W: Write>(&self, w: &mut W) -> fmt::Result {
        write!(w, "{self}sI")
    }
}

impl Snbt for i64 {
    fn to_snbt<W: Write>(&self, w: &mut W) -> fmt::Result {
        write!(w, "{self}L")
    }
}

impl Snbt for u64 {
    fn to_snbt<W: Write>(&self, w: &mut W) -> fmt::Result {
        write!(w, "{self}sL")
    }
}

impl Snbt for f32 {
    fn to_snbt<W: Write>(&self, w: &mut W) -> fmt::Result {
        write!(w, "{self}f")
    }
}

impl Snbt for f64 {
    fn to_snbt<W: Write>(&self, w: &mut W) -> fmt::Result {
        write!(w, "{self}d")
    }
}

impl Snbt for &str {
    fn to_snbt<W: Write>(&self, w: &mut W) -> fmt::Result {
        let quote = self.chars().find_map(|ch| match ch {
            '\'' => Some('"'),
            '"' => Some('\''),
            _ => None,
        }).unwrap_or('"');

        w.write_char(quote)?;

        // some crazy shit in here just so it can only do a single pass
        for ch in self.chars() {
            match ch {
                '\x08' => w.write_str("\\b"),
                '\x0C' => w.write_str("\\f"),
                '\n' => w.write_str("\\n"),
                '\r' => w.write_str("\\r"),
                '\t' => w.write_str("\\t"),
                '\\' => w.write_str("\\\\"),
                c if c == quote => write!(w, "\\{c}"),
                c => w.write_char(c),
            }?; // feels kinda wrong putting it here, but it works
        }

        w.write_char(quote)
    }
}

impl Snbt for String {
    fn to_snbt<W: Write>(&self, w: &mut W) -> fmt::Result {
        self.as_str().to_snbt(w)
    }
}

impl<T: Snbt> Snbt for &[T] {
    fn to_snbt<W: Write>(&self, w: &mut W) -> fmt::Result {
        w.write_char('[')?;
        write_array_inner(w, *self)?;
        w.write_char(']')
    }
}

impl<T: Snbt> Snbt for Vec<T> {
    fn to_snbt<W: Write>(&self, w: &mut W) -> fmt::Result {
        self.as_slice().to_snbt(w)
    }
}

impl<T: Snbt, U: Snbt> Snbt for BTreeMap<T, U> {
    fn to_snbt<W: Write>(&self, w: &mut W) -> fmt::Result {
        w.write_char('{')?;

        for (name, data) in self {
            name.to_snbt(w)?;
            w.write_str(":")?;
            data.to_snbt(w)?;
        }

        w.write_char('}')
    }
}

// gotta use this tuple instead of TagData so that GenericArray doesn't require a ton more code
impl Snbt for (u8, &Payload) {
    fn to_snbt<W: Write>(&self, w: &mut W) -> fmt::Result {
        match self.1 {
            Payload::Byte(v) => v.to_snbt(w),
            Payload::Short(v) => v.to_snbt(w),
            Payload::Int(v) => v.to_snbt(w),
            Payload::Long(v) => v.to_snbt(w),
            Payload::Float(v) => v.to_snbt(w),
            Payload::Double(v) => v.to_snbt(w),
            Payload::String(v) => v.to_snbt(w),
            Payload::EmptyArray => write!(w, "[]"),
            Payload::ByteArray(arr) => {
                if self.0 != TAG_LIST {
                    w.write_str("[B;")?;
                    write_array_inner(w, arr)?;
                    w.write_char(']')
                } else {
                    arr.to_snbt(w)
                }
            },
            Payload::ShortArray(arr) => arr.to_snbt(w),
            Payload::IntArray(arr) => {
                if self.0 != TAG_LIST {
                    w.write_str("[I;")?;
                    write_array_inner(w, arr)?;
                    w.write_char(']')
                } else {
                    arr.to_snbt(w)
                }
            },
            Payload::LongArray(arr) => {
                if self.0 != TAG_LIST {
                    w.write_str("[L;")?;
                    write_array_inner(w, arr)?;
                    w.write_char(']')
                } else {
                    arr.to_snbt(w)
                }
            },
            Payload::FloatArray(arr) => arr.to_snbt(w),
            Payload::DoubleArray(arr) => arr.to_snbt(w),
            Payload::StringArray(arr) => arr.to_snbt(w),
            Payload::GenericArray(arr, list_tag) => arr.iter().map(|v| (*list_tag, v) ).collect::<Vec<_>>().to_snbt(w),
            Payload::Compound(map) => map.to_snbt(w)
        }
    }
}

impl Snbt for TagData {
    fn to_snbt<W: Write>(&self, w: &mut W) -> fmt::Result {
        (self.tag, &self.payload).to_snbt(w)
    }
}

fn write_array_inner<'a, W: Write, T: Snbt + 'a, I: IntoIterator<Item = &'a T>>(w: &mut W, iter: I) -> std::fmt::Result {
    let mut it = iter.into_iter().peekable();
    while let Some(v) = it.next() {
        v.to_snbt(w)?;

        if it.peek().is_some() {
            w.write_char(',')?;
        }
    }
    Ok(())
}
