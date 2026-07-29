use std::{num::{IntErrorKind, ParseIntError}, sync::Arc};

use egui::{Widget, text::LayoutJob};
use egui_dnd::DragDropItem;
use indexmap::IndexMap;

use crate::{nbt::{Payload, RootTag, TagData}};

/// This is a trait, it is here to make my life easier
pub trait ToEguiLeaf {
    /// Basically render this type to egui, mainly for `NbtLeaf`, but maybe you will find more uses later
    fn to_egui_leaf(&mut self, id: egui::Id, ui: &mut egui::Ui, buffer: &mut String, name: &str, name_width: f32) -> egui::Response;
}

// my first ever macro, simple and kinda copied, but thats how we all start, right?
macro_rules! to_egui_leaf_int_impl {
    ($($t:ty)+) => {$(
        impl ToEguiLeaf for $t {
            fn to_egui_leaf(&mut self, id: egui::Id, ui: &mut egui::Ui, buffer: &mut String, mut name: &str, name_width: f32) -> egui::Response {
                ui.horizontal(|ui| {
                    let spacing_x = ui.spacing().item_spacing.x;
                    ui.spacing_mut().item_spacing.x = 0.0;
                    ui.add(egui::TextEdit::singleline(&mut name).desired_width(name_width).margin(egui::Margin { left: 4, right: 0, top: 2, bottom: 2 }));

                    let color = ui.visuals().override_text_color.unwrap_or_else(|| ui.visuals().widgets.inactive.text_color());
                    ui.spacing_mut().item_spacing.x = spacing_x;
                    ui.add(egui::Label::new(egui::RichText::new(":").color(color)).selectable(false));

                    let res = ui.add(egui::TextEdit::singleline(buffer).id(id));

                    if res.lost_focus() {
                        *self = buffer.parse().unwrap_or_else(|err: ParseIntError| match err.kind() {
                            IntErrorKind::PosOverflow => <$t>::MAX,
                            IntErrorKind::NegOverflow => <$t>::MIN,
                            IntErrorKind::Empty => <$t>::default(),
                            _ => *self,
                        });
                        *buffer = self.to_string();
                    }

                    res
                }).inner
            }
        }

        to_egui_leaf_vec_num_impl! { $t }
    )*}
}

// even simpler version of the above one
macro_rules! to_egui_leaf_float_impl {
    ($($t:ty)+) => {$(
        impl ToEguiLeaf for $t {
            fn to_egui_leaf(&mut self, id: egui::Id, ui: &mut egui::Ui, buffer: &mut String, mut name: &str, name_width: f32) -> egui::Response {
                ui.horizontal(|ui| {
                    let spacing_x = ui.spacing().item_spacing.x;
                    ui.spacing_mut().item_spacing.x = 0.0;
                    ui.add(egui::TextEdit::singleline(&mut name).desired_width(name_width).margin(egui::Margin { left: 4, right: 0, top: 2, bottom: 2 }));

                    let color = ui.visuals().override_text_color.unwrap_or_else(|| ui.visuals().widgets.inactive.text_color());
                    ui.spacing_mut().item_spacing.x = spacing_x;
                    ui.add(egui::Label::new(egui::RichText::new(":").color(color)).selectable(false));

                    let res = ui.add(egui::TextEdit::singleline(buffer).id(id));

                    if res.lost_focus() {
                        if let Ok(v) = buffer.parse() { *self = v; }
                        *buffer = self.to_string();
                    }

                    res
                }).inner
            }
        }

        to_egui_leaf_vec_num_impl! { $t }
    )*}
}

// am I good at naming stuff?
macro_rules! to_egui_leaf_vec_num_impl {
    ($t:ty) => {
        impl ToEguiLeaf for Vec<($t, String)> {
            fn to_egui_leaf(&mut self, id: egui::Id, ui: &mut egui::Ui, _buffer: &mut String, name: &str, name_width: f32) -> egui::Response {
                NbtLeaf::inspect_list(self, |item, id, ui| {
                    let name = format!("Item {}", item.index);
                    let name_width = NbtLeaf::calculate_name_width(name.clone(), ui);
                    item.item.0.to_egui_leaf(id.with(item.index), ui, &mut item.item.1, &name, name_width)
                }, id, name, name_width, ui)
            }
        }
    }
}

to_egui_leaf_int_impl! { i8 i16 i32 i64 }
to_egui_leaf_float_impl! { f32 f64 }

impl ToEguiLeaf for String {
    fn to_egui_leaf(&mut self, id: egui::Id, ui: &mut egui::Ui, buffer: &mut String, mut name: &str, name_width: f32) -> egui::Response {
        ui.horizontal(|ui| {
            let spacing_x = ui.spacing().item_spacing.x;
            ui.spacing_mut().item_spacing.x = 0.0;
            ui.add(egui::TextEdit::singleline(&mut name).desired_width(name_width).margin(egui::Margin { left: 4, right: 0, top: 2, bottom: 2 }));

            let color = ui.visuals().override_text_color.unwrap_or_else(|| ui.visuals().widgets.inactive.text_color());
            ui.spacing_mut().item_spacing.x = spacing_x;
            ui.add(egui::Label::new(egui::RichText::new(":").color(color)).selectable(false));

            let res = ui.add(egui::TextEdit::singleline(buffer).id(id));

            if res.lost_focus() {
                *self = buffer.to_owned();
            }

            res
        }).inner
    }
}

impl ToEguiLeaf for NbtLeaf {
    fn to_egui_leaf(&mut self, id: egui::Id, ui: &mut egui::Ui, _buffer: &mut String, name: &str, name_width: f32) -> egui::Response {
        let id = if id == egui::Id::NULL { ui.next_auto_id() } else { id.with(&self.name) };

        if !name.is_empty() && self.name != name {
            self.name = name.to_owned();
            self.name_width = name_width;
        }

        if self.name_width.is_infinite() {
            self.update_name_width(ui);
        }

        ui.vertical(|ui| match &mut self.data {
            NbtLeafData::Byte(v) => v.to_egui_leaf(id, ui, &mut self.buffer, &self.name, self.name_width),
            NbtLeafData::Short(v) => v.to_egui_leaf(id, ui, &mut self.buffer, &self.name, self.name_width),
            NbtLeafData::Int(v) => v.to_egui_leaf(id, ui, &mut self.buffer, &self.name, self.name_width),
            NbtLeafData::Long(v) => v.to_egui_leaf(id, ui, &mut self.buffer, &self.name, self.name_width),
            NbtLeafData::Float(v) => v.to_egui_leaf(id, ui, &mut self.buffer, &self.name, self.name_width),
            NbtLeafData::Double(v) => v.to_egui_leaf(id, ui, &mut self.buffer, &self.name, self.name_width),
            NbtLeafData::String(v) => v.to_egui_leaf(id, ui, &mut self.buffer, &self.name, self.name_width),
            NbtLeafData::ByteArray(v) => v.to_egui_leaf(id, ui, &mut self.buffer, &self.name, self.name_width),
            NbtLeafData::IntArray(v) => v.to_egui_leaf(id, ui, &mut self.buffer, &self.name, self.name_width),
            NbtLeafData::LongArray(v) => v.to_egui_leaf(id, ui, &mut self.buffer, &self.name, self.name_width),
            NbtLeafData::List(v) => v.to_egui_leaf(id, ui, &mut self.buffer, &self.name, self.name_width),
            NbtLeafData::Compound(v) => Self::inspect_compound(v, id, &self.name, self.name_width, ui)
        }).inner
    }
}

impl ToEguiLeaf for Vec<NbtLeaf> {
    fn to_egui_leaf(&mut self, id: egui::Id, ui: &mut egui::Ui, _buffer: &mut String, name: &str, name_width: f32) -> egui::Response {
        NbtLeaf::inspect_list(self, |item, id, ui| {
            let name = format!("Item {}", item.index);
            let name_width = NbtLeaf::calculate_name_width(name.clone(), ui);
            item.item.to_egui_leaf(id, ui, &mut String::new(), &name, name_width)
        }, id, name, name_width, ui)
    }
}

#[derive(Clone, Debug, PartialEq)]
pub enum NbtLeafData {
    Byte(i8),
    Short(i16),
    Int(i32),
    Long(i64),
    Float(f32),
    Double(f64),
    String(String),
    ByteArray(Vec<(i8, String)>), // the String is for the buffer, TextEdit needs a buffer, and this is the easiest and simplest way
    IntArray(Vec<(i32, String)>), // compared to Vec<NbtLeaf>, this is still smaller, as it doesn't store the name
    LongArray(Vec<(i64, String)>),
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
            Payload::ByteArray(v) => Self::ByteArray(v.iter().map(|&v| (v, v.to_string())).collect()),
            Payload::IntArray(v) => Self::IntArray(v.iter().map(|&v| (v, v.to_string())).collect()),
            Payload::LongArray(v) => Self::LongArray(v.iter().map(|&v| (v, v.to_string())).collect()),
            Payload::EmptyList => Self::List(Vec::new()),
            Payload::ByteList(v) => Self::List(NbtLeaf::easy_create_list(v)),
            Payload::ShortList(v) => Self::List(NbtLeaf::easy_create_list(v)),
            Payload::IntList(v) => Self::List(NbtLeaf::easy_create_list(v)),
            Payload::LongList(v) => Self::List(NbtLeaf::easy_create_list(v)),
            Payload::FloatList(v) => Self::List(NbtLeaf::easy_create_list(v)),
            Payload::DoubleList(v) => Self::List(NbtLeaf::easy_create_list(v)),
            Payload::StringList(v) => Self::List(NbtLeaf::easy_create_list(v)),
            Payload::ByteArrayList(v) => Self::List(NbtLeaf::easy_create_list_with_fn(v, |v| Self::ByteArray(v.iter().map(|&v| (v, v.to_string())).collect()))),
            Payload::IntArrayList(v) => Self::List(NbtLeaf::easy_create_list_with_fn(v, |v| Self::IntArray(v.iter().map(|&v| (v, v.to_string())).collect()))),
            Payload::LongArrayList(v) => Self::List(NbtLeaf::easy_create_list_with_fn(v, |v| Self::LongArray(v.iter().map(|&v| (v, v.to_string())).collect()))),
            Payload::ListList(v) => Self::List(NbtLeaf::easy_create_list(v)),
            Payload::CompoundList(v) => Self::List(NbtLeaf::easy_create_list(v)),
            Payload::Compound(v) => Self::from(v),
        }
    }
}

impl From<&IndexMap<String, TagData>> for NbtLeafData {
    fn from(value: &IndexMap<String, TagData>) -> Self {
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

        compounds.sort_by_key(|v| v.name.to_lowercase());
        lists.sort_by_key(|v| v.name.to_lowercase());
        others.sort_by_key(|v| v.name.to_lowercase());

        Self::Compound(compounds.into_iter().chain(lists).chain(others).collect())
    }
}

impl NbtLeafData {
    /// get a string version of the data, but specifically tailored for `NbtLeaf` and its buffer
    fn get_buffer_string(&self) -> Option<String> {
        match self {
            Self::Byte(v) => Some(v.to_string()),
            Self::Short(v) => Some(v.to_string()),
            Self::Int(v) => Some(v.to_string()),
            Self::Long(v) => Some(v.to_string()),
            Self::Float(v) => Some(v.to_string()),
            Self::Double(v) => Some(v.to_string()),
            Self::String(v) => Some(v.to_owned()),
            _ => None,
        }
    }
}

/// A copy of the private `egui_field_editor::EnumeratedItem` cuz I need it for thing
struct EnumeratedItem<T> {
    item: T,
    index: usize,
    salt_id: egui::Id,
}

impl<T> DragDropItem for EnumeratedItem<&mut T> {
    fn id(&self) -> egui::Id {
        egui::Id::new(self.salt_id.with(self.index))
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct NbtLeaf {
    name: String,
    data: NbtLeafData,
    name_width: f32,
    buffer: String,
}

impl NbtLeaf {
    fn update_name_width(&mut self, ui: &egui::Ui) {
        self.name_width = Self::calculate_name_width(self.name.clone(), ui);
    }

    fn calculate_name_width(text: String, ui: &egui::Ui) -> f32 {
        Self::get_galley_from_string(text, ui).rect.width() + 4.0
    }

    fn get_galley_from_string(text: String, ui: &egui::Ui) -> Arc<egui::Galley> {
        let font_id = egui::FontSelection::Default.resolve(ui.style());
        let color = ui.visuals().override_text_color.unwrap_or_else(|| ui.visuals().widgets.inactive.text_color());
        let mut job = LayoutJob::simple_singleline(text, font_id, color);
        job.halign = egui::Align::LEFT;
        job.keep_trailing_whitespace = true;
        ui.fonts_mut(|f| f.layout_job(job))
    }

    // copied and slightly modified inspect_with_custom_id from the implementation on Vec<T>
    fn inspect_list<T, F: Fn(EnumeratedItem<&mut T>, egui::Id, &mut egui::Ui) -> egui::Response>(
        v: &mut [T],
        f: F,
        parent_id: egui::Id,
        label: &str,
        label_width: f32,
        ui: &mut egui::Ui,
    ) -> egui::Response {
        let id = if parent_id == egui::Id::NULL { ui.next_auto_id() } else { parent_id.with(label) };
        let parent_id_for_children = if parent_id == egui::Id::NULL { egui::Id::NULL } else { id };

        let mut changed = false;

        let data_len = v.len();

        let collapsing_resp = egui::collapsing_header::CollapsingState::load_with_default_open(
            ui.ctx(),
            id.with("collapse"),
            false
        ).show_header(ui, |ui| {
            Self::collection_header(label, label_width, data_len, ui)
        }).body(|ui| {
            let dnd_resp = egui_dnd::dnd(ui, id.with("dnd"))
                .with_animation_time(0.0)
                .show(
                    v.iter_mut().enumerate().map(|(i, item)| EnumeratedItem { item, index: i, salt_id: id }),
                    |ui, item, handle, state| {
                        ui.horizontal(|ui| {
                            handle.ui(ui, |ui| {
                                ui.label(if state.dragged { "≡" } else { "☰" });
                            });

                            let res = f(item, parent_id_for_children, ui);

                            if res.changed() {
                                changed = true;
                            }
                        });
                    },
                );

            if dnd_resp.is_drag_finished() {
                dnd_resp.update_vec(v);
                changed = true;
            }

            dnd_resp
        });

        let mut res = ui.response();
        if let Some(body_res) = collapsing_resp.2 {
            res = res.union(body_res.response);
        }

        if changed {
            res.mark_changed();
        }

        res
    }

    // copied and slightly modified inspect_with_custom_id from the implementation on HashMap<String, T>
    fn inspect_compound(v: &mut [Self], parent_id: egui::Id, label: &str, label_width: f32, ui: &mut egui::Ui) -> egui::Response {
        let id = if parent_id == egui::Id::NULL { ui.next_auto_id() } else { parent_id.with(label) };

        let data_len = v.len();

        let mut add_content = |ui: &mut egui::Ui| {
            let mut resp = ui.response();

            for value in v.iter_mut() {
                resp = resp.union(ui.horizontal_top(|ui| {
                    value.to_egui_leaf(id.with(&value.name), ui, &mut String::new(), "", f32::INFINITY)
                }).inner);
            }

            resp
        };

        let mut header_resp = None;

        let content_resp = if !label.is_empty() {
            let resp = egui::collapsing_header::CollapsingState::load_with_default_open(
                ui.ctx(),
                id.with("collapse"),
                false
            ).show_header(ui, |ui| {
                Self::collection_header(label, label_width, data_len, ui)
            }).body(add_content);

            header_resp = Some(resp.1.inner);
            resp.2.map(|v| v.inner)
        } else {
            Some(add_content(ui))
        };

        let mut res = ui.response();
        if let Some(body_res) = content_resp {
            res = res.union(body_res);
        }
        if let Some(head_res) = header_resp {
            res = res.union(head_res);
        }

        res
    }

    fn collection_header(mut label: &str, label_width: f32, data_len: usize, ui: &mut egui::Ui) -> egui::Response {
        let spacing_x = ui.spacing().item_spacing.x;
        ui.spacing_mut().item_spacing.x = 0.0;

        let res = ui.add(egui::TextEdit::singleline(&mut label).desired_width(label_width).margin(egui::Margin { left: 0, right: 4, top: 2, bottom: 2 }));
        
        ui.spacing_mut().item_spacing.x = spacing_x;

        if data_len == 1 {
            res.union(ui.add(egui::Label::new("1 entry").selectable(false)))
        } else {
            res.union(ui.add(egui::Label::new(format!("{data_len} entries")).selectable(false)))
        }
    }
}

impl From<&RootTag> for NbtLeaf {
    fn from(value: &RootTag) -> Self {
        Self::new(value.name(), value.payload())
    }
}

impl NbtLeaf {
    pub fn new(name: impl Into<String>, data: impl Into<NbtLeafData>) -> Self {
        let name = name.into();
        let data = data.into();
        let buffer = data.get_buffer_string().unwrap_or_default();
        Self {
            name,
            data,
            name_width: f32::INFINITY,
            buffer,
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
        if let Some(buf) = self.data.get_buffer_string() {
            self.buffer = buf;
        }
    }

    // stupid shit that works, sometimes, had to be changed because a previous version would cause the compiler to panic
    fn easy_create_list<T>(v: &[T]) -> Vec<Self> where for<'a> &'a T: Into<NbtLeafData> {
        v.iter().enumerate().map(|(i, v)| Self::new(format!("Item {i}"), v)).collect()
    }

    // i don't really care what these are called, cuz how tf am i supposed to describe what it does with just a function name
    fn easy_create_list_with_fn<T, U: Into<NbtLeafData>, F: Fn(&T) -> U>(v: &[T], f: F) -> Vec<Self> {
        v.iter().enumerate().map(|(i, v)| Self::new(format!("Item {i}"), f(v))).collect()
    }
}

/// Is an `NbtLeaf` with more stuff for like styling the whole tree
/// This is meant to be stored in a variable, you save it, its not to be created every single ui re-draw, as it is kinda expensive
#[derive(Clone, Debug)]
pub struct NbtTree {
    leaf: NbtLeaf, // top leaf, always only one, should be a compound, hopefully...
    enabled: bool,
}

impl NbtTree {
    pub fn new(leaf: impl Into<NbtLeaf>) -> Self {
        Self {
            leaf: leaf.into(),
            enabled: true,
        }
    }

    pub fn leaf(&self) -> &NbtLeaf {
        &self.leaf
    }

    pub fn set_leaf(&mut self, leaf: impl Into<NbtLeaf>) {
        self.leaf = leaf.into();
    }

    pub fn enabled(&self) -> bool {
        self.enabled
    }

    pub fn set_enabled(&mut self, enabled: bool) {
        self.enabled = enabled;
    }
}

impl Widget for NbtTree {
    fn ui(mut self, ui: &mut egui::Ui) -> egui::Response {
        (&mut self).ui(ui)
    }
}

impl Widget for &mut NbtTree {
    fn ui(self, ui: &mut egui::Ui) -> egui::Response {
        ui.set_min_width(250.0);

        let id = ui.next_auto_id();
        let available_width = ui.available_width();

        egui::ScrollArea::vertical().id_salt(id.with("scroll")).show(ui, |ui| {
            ui.set_min_width(available_width);

            self.leaf.to_egui_leaf(id, ui, &mut String::new(), "", f32::INFINITY)
        }).inner
    }
}
