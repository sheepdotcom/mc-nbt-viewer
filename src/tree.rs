use std::collections::BTreeMap;

use egui::{CollapsingHeader, IdSalt, TextEdit, Widget};
use egui_field_editor::EguiInspect;
use either::Either;

use crate::{RootTag, nbt::{Payload, TagData}};

#[derive(Clone, Debug, PartialEq)]
pub enum NbtLeafData {
    Byte(i8),
    Short(i16),
    Int(i32),
    Long(i64),
    Float(f32),
    Double(f64),
    String(String),
    ByteArray(Vec<i8>),
    IntArray(Vec<i32>),
    LongArray(Vec<i64>),
    List(Vec<NbtLeaf>),
    Compound(Vec<NbtLeaf>),
}

impl From<i8> for NbtLeafData {
    fn from(value: i8) -> Self {
        Self::Byte(value)
    }
}

impl From<&i8> for NbtLeafData {
    fn from(value: &i8) -> Self {
        Self::Byte(*value)
    }
}

impl From<i16> for NbtLeafData {
    fn from(value: i16) -> Self {
        Self::Short(value)
    }
}

impl From<&i16> for NbtLeafData {
    fn from(value: &i16) -> Self {
        Self::Short(*value)
    }
}

impl From<i32> for NbtLeafData {
    fn from(value: i32) -> Self {
        Self::Int(value)
    }
}

impl From<&i32> for NbtLeafData {
    fn from(value: &i32) -> Self {
        Self::Int(*value)
    }
}

impl From<i64> for NbtLeafData {
    fn from(value: i64) -> Self {
        Self::Long(value)
    }
}

impl From<&i64> for NbtLeafData {
    fn from(value: &i64) -> Self {
        Self::Long(*value)
    }
}

impl From<f32> for NbtLeafData {
    fn from(value: f32) -> Self {
        Self::Float(value)
    }
}

impl From<&f32> for NbtLeafData {
    fn from(value: &f32) -> Self {
        Self::Float(*value)
    }
}

impl From<f64> for NbtLeafData {
    fn from(value: f64) -> Self {
        Self::Double(value)
    }
}

impl From<&f64> for NbtLeafData {
    fn from(value: &f64) -> Self {
        Self::Double(*value)
    }
}

impl From<String> for NbtLeafData {
    fn from(value: String) -> Self {
        Self::String(value)
    }
}

impl From<&String> for NbtLeafData{
    fn from(value: &String) -> Self {
        Self::String(value.to_owned())
    }
}

impl From<&str> for NbtLeafData {
    fn from(value: &str) -> Self {
        Self::String(value.to_owned())
    }
}

impl From<&TagData> for NbtLeafData {
    fn from(value: &TagData) -> Self {
        value.payload().into()
    }
}

impl From<&Payload> for NbtLeafData {
    fn from(value: &Payload) -> Self {
        match value {
            Payload::Byte(v) => v.into(),
            Payload::Short(v) => v.into(),
            Payload::Int(v) => v.into(),
            Payload::Long(v) => v.into(),
            Payload::Float(v) => v.into(),
            Payload::Double(v) => v.into(),
            Payload::String(v) => v.into(),
            Payload::ByteArray(v) => Self::ByteArray(v.clone()),
            Payload::IntArray(v) => Self::IntArray(v.clone()),
            Payload::LongArray(v) => Self::LongArray(v.clone()),
            Payload::EmptyList => Self::List(Vec::new()),
            Payload::ByteList(v) => Self::List(NbtLeaf::easy_create_list(v)),
            Payload::ShortList(v) => Self::List(NbtLeaf::easy_create_list(v)),
            Payload::IntList(v) => Self::List(NbtLeaf::easy_create_list(v)),
            Payload::LongList(v) => Self::List(NbtLeaf::easy_create_list(v)),
            Payload::FloatList(v) => Self::List(NbtLeaf::easy_create_list(v)),
            Payload::DoubleList(v) => Self::List(NbtLeaf::easy_create_list(v)),
            Payload::StringList(v) => Self::List(NbtLeaf::easy_create_list(v)),
            Payload::ByteArrayList(v) => Self::List(NbtLeaf::easy_create_list_alt(v, |v| Self::ByteArray(v.clone()))),
            Payload::IntArrayList(v) => Self::List(NbtLeaf::easy_create_list_alt(v, |v| Self::IntArray(v.clone()))),
            Payload::LongArrayList(v) => Self::List(NbtLeaf::easy_create_list_alt(v, |v| Self::LongArray(v.clone()))),
            Payload::ListList(v) => Self::List(NbtLeaf::easy_create_list(v)),
            Payload::CompoundList(v) => Self::List(NbtLeaf::easy_create_list(v)),
            Payload::Compound(v) => v.into(),
        }
    }
}

impl From<&BTreeMap<String, TagData>> for NbtLeafData {
    fn from(value: &BTreeMap<String, TagData>) -> Self {
        let mut compounds = Vec::new();
        let mut lists = Vec::new();
        let mut others = Vec::new();

        for (name, data) in value {
            let leaf = NbtLeaf::new(name, data);
            match leaf.data {
                Self::Compound(..) => compounds.push(leaf),
                Self::List(..) => lists.push(leaf),
                _ => others.push(leaf),
            }
        }

        Self::Compound(compounds.into_iter().chain(lists).chain(others).collect())
    }
}

#[derive(Clone, Debug, EguiInspect, PartialEq)]
pub struct NbtLeaf {
    name: String,
    #[inspect(hidden)] // TODO: add a custom way to edit this, so that instead of it showing as an enum, it only shows the type it should be
    data: NbtLeafData,
    #[inspect(hidden)]
    open: bool, // only does something on lists and compounds, as those become CollapsingHeader
    #[inspect(hidden)]
    id_salt: IdSalt,
}

impl From<&RootTag> for NbtLeaf {
    fn from(value: &RootTag) -> Self {
        Self::new(value.name(), value.payload())
    }
}

impl NbtLeaf {
    pub fn new(name: impl Into<String>, data: impl Into<NbtLeafData>) -> Self {
        let name = name.into();
        let id_salt = IdSalt::new(&name);
        let data = data.into();
        Self {
            name,
            data,
            open: false,
            id_salt,
        }
    }

    pub fn name(&self) -> &str {
        &self.name
    }

    pub fn set_name(&mut self, name: impl Into<String>) {
        self.name = name.into();
    }

    pub fn data(&self) -> &NbtLeafData {
        &self.data
    }

    pub fn set_data(&mut self, data: impl Into<NbtLeafData>) {
        self.data = data.into();
    }

    pub fn open(&self) -> bool {
        self.open
    }

    pub fn set_open(&mut self, open: bool) {
        self.open = open;
    }

    pub fn id_salt(&self) -> IdSalt {
        self.id_salt
    }

    pub fn set_id_salt(&mut self, id_salt: IdSalt) {
        self.id_salt = id_salt;
    }

    // stupid shit that works, sometimes, had to be changed because a previous version would cause the compiler to panic
    fn easy_create_list<T>(v: &[T]) -> Vec<Self> where for<'a> &'a T: Into<NbtLeafData> {
        v.iter().enumerate().map(|(i, v)| Self::new(i.to_string(), v)).collect()
    }

    // i don't really care what these are called, cuz how tf am i supposed to describe what it does with just a function name
    fn easy_create_list_alt<T, U: Into<NbtLeafData>, F: Fn(&T) -> U>(v: &[T], f: F) -> Vec<Self> {
        v.iter().enumerate().map(|(i, v)| Self::new(i.to_string(), f(v))).collect()
    }

    // pub fn to_egui_widget(&self) -> Either<CollapsingHeader, TextEdit<'_>> {
    //     match self.data() {
    //         NbtLeafData::Byte(v) => TextEdit::singleline(text)
    //     }
    // }
}

/// Is an `NbtLeaf` with more stuff for like styling the whole tree
/// This is meant to be stored in a variable, you save it, its not to be created every single ui re-draw, as it is kinda expensive
#[derive(Clone, Debug, EguiInspect)]
pub struct NbtTree {
    leaf: NbtLeaf, // top leaf, always only one, should be a compound, hopefully...
    enabled: bool,
    selectable: bool,
    selected: bool,
    indented: bool,
}

impl NbtTree {
    pub fn new(leaf: impl Into<NbtLeaf>) -> Self {
        Self {
            leaf: leaf.into(),
            enabled: true,
            selectable: false,
            selected: false,
            indented: true,
        }
    }

    pub fn enabled(&self) -> bool {
        self.enabled
    }

    pub fn set_enabled(&mut self, enabled: bool) {
        self.enabled = enabled;
    }

    pub fn selectable(&self) -> bool {
        self.selectable
    }

    pub fn set_selectable(&mut self, selectable: bool) {
        self.selectable = selectable;
    }

    pub fn selected(&self) -> bool {
        self.selected
    }

    pub fn set_selected(&mut self, selected: bool) {
        self.selected = selected;
    }

    pub fn indented(&self) -> bool {
        self.indented
    }

    pub fn set_indented(&mut self, indented: bool) {
        self.indented = indented;
    }
}

// impl Widget for NbtTree {
//     fn ui(self, ui: &mut egui::Ui) -> egui::Response {
//         (&self).ui(ui)
//     }
// }

// impl Widget for &NbtTree {
//     fn ui(self, ui: &mut egui::Ui) -> egui::Response {
//         let top = self.leaf.ui(ui);
//
//         todo!()
//     }
// }
