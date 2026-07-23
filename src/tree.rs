use egui::{Widget, text::LayoutJob};
use egui_dnd::DragDropItem;
use egui_field_editor::{EguiInspect, EguiInspector};
use indexmap::IndexMap;

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
            Payload::ByteArrayList(v) => Self::List(NbtLeaf::easy_create_list_with_fn(v, |v| Self::ByteArray(v.clone()))),
            Payload::IntArrayList(v) => Self::List(NbtLeaf::easy_create_list_with_fn(v, |v| Self::IntArray(v.clone()))),
            Payload::LongArrayList(v) => Self::List(NbtLeaf::easy_create_list_with_fn(v, |v| Self::LongArray(v.clone()))),
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
    fn is_array_type(&self) -> bool {
        matches!(self, Self::ByteArray(..) | Self::IntArray(..) | Self::LongArray(..) | Self::List(..) | Self::Compound(..))
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
}

impl EguiInspect for NbtLeaf {
    fn inspect_with_custom_id(
        &mut self,
        parent_id: egui::Id,
        _label: &str,
        _tooltip: &str,
        _label_ratio: f32,
        _read_only: bool,
        ui: &mut egui::Ui,
    ) -> egui::Response {
        self.to_egui_inspect_mut(parent_id, true, true, ui)
    }
}

impl NbtLeaf {
    fn update_name_width(&mut self, ui: &egui::Ui) {
        self.name_width = Self::calculate_name_width(self.name.clone(), ui);
    }

    fn calculate_name_width(text: String, ui: &egui::Ui) -> f32 {
        let font_id = egui::FontSelection::Default.resolve(ui.style());
        let color = ui.visuals().override_text_color.unwrap_or_else(|| ui.visuals().widgets.inactive.text_color());
        let mut job = LayoutJob::simple_singleline(text, font_id, color);
        job.keep_trailing_whitespace = true;
        let galley = ui.fonts_mut(|f| f.layout_job(job));
        galley.rect.width() + 8.0 // add 8 cuz the default x margin is 4 per side
    }

    pub fn to_egui_inspect_mut(
        &mut self,
        parent_id: egui::Id,
        editable_name: bool,
        editable_keys: bool,
        ui: &mut egui::Ui,
    ) -> egui::Response {
        let id = if parent_id == egui::Id::NULL { ui.next_auto_id() } else { parent_id.with(&self.name) };

        if self.name_width.is_infinite() { self.update_name_width(ui); }

        ui.vertical(|ui| match &mut self.data {
            NbtLeafData::Byte(v) => v.inspect_with_custom_id(id, "", "", 0.0, false, ui),
            NbtLeafData::Short(v) => v.inspect_with_custom_id(id, "", "", 0.0, false, ui),
            NbtLeafData::Int(v) => v.inspect_with_custom_id(id, "", "", 0.0, false, ui),
            NbtLeafData::Long(v) => v.inspect_with_custom_id(id, "", "", 0.0, false, ui),
            NbtLeafData::Float(v) => v.inspect_with_custom_id(id, "", "", 0.0, false, ui),
            NbtLeafData::Double(v) => v.inspect_with_custom_id(id, "", "", 0.0, false, ui),
            NbtLeafData::String(v) => v.inspect_with_custom_id(id, "", "", 0.0, false, ui),
            NbtLeafData::ByteArray(v) => Self::inspect_list(v, |item, id, ui| { // TODO: repetition will probably get replaced in future code change
                item.item.inspect_with_custom_id(id, &format!("Item {}", item.index), "", 0.0, false, ui)
            }, parent_id, &mut self.name, self.name_width, editable_name, ui),
            NbtLeafData::IntArray(v) => Self::inspect_list(v, |item, id, ui| {
                item.item.inspect_with_custom_id(id, &format!("Item {}", item.index), "", 0.0, false, ui)
            }, parent_id, &mut self.name, self.name_width, editable_name, ui),
            NbtLeafData::LongArray(v) => Self::inspect_list(v, |item, id, ui| {
                item.item.inspect_with_custom_id(id, &format!("Item {}", item.index), "", 0.0, false, ui)
            }, parent_id, &mut self.name, self.name_width, editable_name, ui),
            NbtLeafData::List(v) => Self::inspect_list(v, |item, id, ui| {
                item.item.name = format!("Item {}", item.index);
                item.item.to_egui_inspect_mut(id, false, true, ui)
            }, parent_id, &mut self.name, self.name_width, editable_name, ui),
            NbtLeafData::Compound(v) => Self::inspect_compound(v, parent_id, &mut self.name, self.name_width, editable_name, editable_keys, ui)
        }).inner
    }
    
    // copied and slightly modified inspect_with_custom_id from the implementation on Vec<T>
    fn inspect_list<T, F: Fn(EnumeratedItem<&mut T>, egui::Id, &mut egui::Ui) -> egui::Response>(
        v: &mut [T],
        f: F,
        parent_id: egui::Id,
        label: &mut String,
        label_width: f32,
        editable_name: bool,
        ui: &mut egui::Ui,
    ) -> egui::Response {
        let id = if parent_id == egui::Id::NULL { ui.next_auto_id() } else { parent_id.with(&label) };
        let parent_id_for_children = if parent_id == egui::Id::NULL { egui::Id::NULL } else { id };

        let mut changed = false;

        let collapsing_resp = egui::collapsing_header::CollapsingState::load_with_default_open(
            ui.ctx(),
            id.with("collapse"),
            false
        ).show_header(ui, |ui| {
            if editable_name {
                let res = ui.add(egui::TextEdit::singleline(label).desired_width(label_width).clip_text(true).horizontal_align(egui::Align::RIGHT));
                res.union(ui.label(format!("[{}]", v.len())))
            } else {
                ui.label(format!("{} [{}]", label, v.len()))
            }
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
    fn inspect_compound(v: &mut [Self], parent_id: egui::Id, label: &mut String, label_width: f32, editable_name: bool, editable_keys: bool, ui: &mut egui::Ui) -> egui::Response {
        let id = if parent_id == egui::Id::NULL { ui.next_auto_id() } else { parent_id.with(&label) };

        let mut changed = false;

        let data_len = v.len();

        let mut add_content = |ui: &mut egui::Ui| {
            let mut resp = ui.response();

            for value in v.iter_mut() {
                let inner_res = if editable_keys && !value.data.is_array_type() {
                    ui.horizontal_top(|ui| {
                        ui.add_enabled_ui(true, |ui| {
                            if value.name_width.is_infinite() { value.update_name_width(ui); }

                            let mut te = value.name.clone();

                            // TextEdit already does this but we need to know it before-hand so we do it ourselves
                            let child_id = ui.next_auto_id();
                            ui.skip_ahead_auto_ids(1);

                            // not really the most accurate solution but its better than basically copying (almost) all of TextEdit's event updating code
                            let width = if ui.memory(|mem| mem.has_focus(child_id)) { value.name_width + 8.0 } else { value.name_width };

                            let text_res = egui::TextEdit::singleline(&mut te).desired_width(width).id(child_id).show(ui);
                            let res = text_res.response.response;

                            if res.changed() && te != value.name {
                                value.name = te.clone();
                                value.name_width = text_res.galley.rect.width() + 8.0;
                                ui.request_repaint(); // need to repaint so no artifacts appear when typing stuff
                                changed = true;
                            }

                            let value_res = ui.vertical(|ui| {
                                value.to_egui_inspect_mut(id.with(&value.name), true, editable_keys, ui)
                            }).inner;

                            res.union(value_res)
                        })
                    })
                } else {
                    ui.horizontal_top(|ui| {
                        ui.add_enabled_ui(true, |ui| {
                            ui.vertical(|ui| {
                                value.to_egui_inspect_mut(
                                    id.with(&value.name),
                                    true,
                                    editable_keys,
                                    ui
                                )
                            }).inner
                        })
                    })
                };

                resp = resp.union(inner_res.inner.inner);
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
                if editable_name {
                    let res = ui.add(egui::TextEdit::singleline(label).desired_width(label_width).horizontal_align(egui::Align::RIGHT));
                    res.union(ui.label(format!("[{data_len}]")))
                } else {
                    ui.label(format!("{label} [{data_len}]"))
                }
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

        if changed {
            res.mark_changed();
        }

        res
    }
}

impl From<&RootTag> for NbtLeaf {
    fn from(value: &RootTag) -> Self {
        Self::new(value.name(), value.payload())
    }
}

impl NbtLeaf {
    pub fn new(name: impl Into<String>, data: impl Into<NbtLeafData>) -> Self {
        Self {
            name: name.into(),
            data: data.into(),
            name_width: f32::INFINITY,
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
#[derive(Clone, Debug, EguiInspect)]
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
        ui.add(EguiInspector::new(&mut self.leaf))
    }
}
