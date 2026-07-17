use std::collections::BTreeMap;

use egui::Widget;
use egui_dnd::DragDropItem;
use egui_field_editor::{EguiInspect, EguiInspector};

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

/// A copy of the private `egui_field_editor::EnumeratedItem` cuz I need it for thing
struct EnumeratedItem<T> {
    item: T,
    index: usize,
    salt_id: egui::Id,
}

impl<T: EguiInspect> DragDropItem for EnumeratedItem<&mut T> {
    fn id(&self) -> egui::Id {
        egui::Id::new(self.salt_id.with(self.index))
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct NbtLeaf {
    name: String,
    data: NbtLeafData,
}

impl EguiInspect for NbtLeaf {
    fn inspect_with_custom_id(
        &mut self,
        parent_id: egui::Id,
        _label: &str,
        tooltip: &str,
        label_ratio: f32,
        read_only: bool,
        ui: &mut egui::Ui,
    ) -> egui::Response {
        self.to_egui_inspect_mut(parent_id, tooltip, label_ratio, read_only, true, true, true, ui)
    }
}

impl NbtLeaf {
    /// This function is so insane that it produces three different clippy errors that I gotta ignore otherwise I have to refactor this a bit,
    /// and right now I already don't like how much I have to mess with it, so I will just ignore the warnings because I have free will
    #[expect(clippy::too_many_arguments)]
    #[expect(clippy::too_many_lines)]
    #[expect(clippy::fn_params_excessive_bools)]
    pub fn to_egui_inspect_mut(
        &mut self,
        parent_id: egui::Id,
        tooltip: &str,
        label_ratio: f32,
        read_only: bool,
        allow_add_delete: bool,
        editable_name: bool,
        editable_keys: bool,
        ui: &mut egui::Ui,
    ) -> egui::Response {
        let id = if parent_id == egui::Id::NULL { ui.next_auto_id() } else { parent_id.with(&self.name) };

        ui.vertical(|ui| match &mut self.data {
            NbtLeafData::Byte(v) => v.inspect_with_custom_id(id, "", tooltip, 0.0, read_only, ui),
            NbtLeafData::Short(v) => v.inspect_with_custom_id(id, "", tooltip, 0.0, read_only, ui),
            NbtLeafData::Int(v) => v.inspect_with_custom_id(id, "", tooltip, 0.0, read_only, ui),
            NbtLeafData::Long(v) => v.inspect_with_custom_id(id, "", tooltip, 0.0, read_only, ui),
            NbtLeafData::Float(v) => v.inspect_with_custom_id(id, "", tooltip, 0.0, read_only, ui),
            NbtLeafData::Double(v) => v.inspect_with_custom_id(id, "", tooltip, 0.0, read_only, ui),
            NbtLeafData::String(v) => v.inspect_with_custom_id(id, "", tooltip, 0.0, read_only, ui),
            NbtLeafData::ByteArray(v) => v.inspect_with_custom_id(id, "", tooltip, 0.0, read_only, ui),
            NbtLeafData::IntArray(v) => v.inspect_with_custom_id(id, "", tooltip, 0.0, read_only, ui),
            NbtLeafData::LongArray(v) => v.inspect_with_custom_id(id, "", tooltip, 0.0, read_only, ui),
            NbtLeafData::List(v) => { // copied and slightly modified inspect_with_custom_id from the implementation on Vec<T>
                let id = if parent_id == egui::Id::NULL { ui.next_auto_id() } else { parent_id.with(&self.name) };
                let parent_id_for_children = if parent_id == egui::Id::NULL { egui::Id::NULL } else { id };

                let mut changed = false;

                let collapsing_resp = egui::collapsing_header::CollapsingState::load_with_default_open(
                    ui.ctx(),
                    id.with("collapse"),
                    false
                ).show_header(ui, |ui| {
                    if editable_name {
                        let res = ui.text_edit_singleline(&mut self.name);
                        res.union(ui.label(format!("[{}]", v.len())))
                    } else {
                        ui.label(format!("{} [{}]", self.name, v.len()))
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

                                    item.item.name = format!("Item {}", item.index);
                                    let res = item.item.to_egui_inspect_mut(
                                        parent_id_for_children,
                                        tooltip,
                                        label_ratio,
                                        read_only,
                                        true,
                                        false,
                                        true,
                                        ui,
                                    );

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
                ui.add_enabled_ui(!read_only, |ui| {
                    ui.add_space(ui.available_width() - 50.0);
                    if ui.add(egui::Button::new("+").min_size(egui::Vec2::new(20.0, 20.0))).clicked() {
                        let data = if let Some(inner) = v.first() {
                            match inner.data {
                                NbtLeafData::Byte(..) => NbtLeafData::Byte(0),
                                NbtLeafData::Short(..) => NbtLeafData::Short(0),
                                NbtLeafData::Int(..) => NbtLeafData::Int(0),
                                NbtLeafData::Long(..) => NbtLeafData::Long(0),
                                NbtLeafData::Float(..) => NbtLeafData::Float(0.0),
                                NbtLeafData::Double(..) => NbtLeafData::Double(0.0),
                                NbtLeafData::String(..) => NbtLeafData::String(String::default()),
                                NbtLeafData::ByteArray(..) => NbtLeafData::ByteArray(Vec::default()),
                                NbtLeafData::IntArray(..) => NbtLeafData::IntArray(Vec::default()),
                                NbtLeafData::LongArray(..) => NbtLeafData::LongArray(Vec::default()),
                                NbtLeafData::List(..) => NbtLeafData::List(Vec::default()),
                                NbtLeafData::Compound(..) => NbtLeafData::Compound(Vec::default()),
                            }
                        } else {
                            NbtLeafData::Int(0) // TODO: make it so you can pick what type to add, rather than forcing an integer on you
                        };
                        v.push(Self::new("name", data));
                        changed = true;
                    }
                    if ui.add(egui::Button::new("-").min_size(egui::Vec2::new(20.0, 20.0))).clicked() && v.pop().is_some() {
                        changed = true;
                    }
                });

                if changed {
                    res.mark_changed();
                }

                res
            },
            NbtLeafData::Compound(v) => { // copied and slightly modified inspect_with_custom_id from the implementation on HashMap<String, T>
                let id = if parent_id == egui::Id::NULL { ui.next_auto_id() } else { parent_id.with(&self.name) };
                let mut changed = false;
                let data_len = v.len();
                let mut add_content = |ui: &mut egui::Ui| {
                    let mut resp = ui.response();

                    for value in v.iter_mut() {
                        let inner_res = if editable_keys && !matches!(value.data, NbtLeafData::List(..) | NbtLeafData::Compound(..)) {
                            ui.horizontal_top(|ui| {
                                ui.add_enabled_ui(!read_only, |ui| {
                                    let mut te = value.name.clone();
                                    let res = ui.add_sized([ui.available_width() * label_ratio, 0.0], egui::TextEdit::singleline(&mut te));

                                    if res.changed() && te != value.name {
                                        value.name = te.clone();
                                        changed = true;
                                    }

                                    let value_res = ui.vertical(|ui| {
                                        value.to_egui_inspect_mut(
                                            id.with(&value.name),
                                            tooltip,
                                            0.0,
                                            read_only,
                                            allow_add_delete,
                                            true,
                                            editable_keys,
                                            ui
                                        )
                                    }).inner;

                                    res.union(value_res)
                                })
                            })
                        } else {
                            ui.horizontal_top(|ui| {
                                ui.add_enabled_ui(!read_only, |ui| {
                                    ui.vertical(|ui| {
                                        value.to_egui_inspect_mut(
                                            id.with(&value.name),
                                            tooltip,
                                            label_ratio,
                                            read_only,
                                            allow_add_delete,
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

                let content_resp = if !self.name.is_empty() {
                    let resp = egui::collapsing_header::CollapsingState::load_with_default_open(
                        ui.ctx(),
                        id.with("collapse"),
                        false
                    ).show_header(ui, |ui| {
                        if editable_name {
                            let res = ui.text_edit_singleline(&mut self.name);
                            res.union(ui.label(format!("[{data_len}]")))
                        } else {
                            ui.label(format!("{} [{data_len}]", self.name))
                        }
                    }).body(add_content);

                    header_resp = Some(resp.1.inner);
                    resp.2.map(|v| v.inner)
                } else {
                    Some(add_content(ui))
                };

                if allow_add_delete {
                    ui.add_enabled_ui(!read_only, |ui| {
                        ui.horizontal_top(|ui| {
                            ui.add_space(ui.available_width() - 50.0);

                            if ui.add(egui::Button::new("+").min_size(egui::Vec2::new(20.0, 20.0))).clicked() {
                                // TODO: make it so you can pick what type to add, rather than forcing an integer on you
                                v.push(Self::new("name", NbtLeafData::Int(0)));
                                changed = true;
                            }

                            if ui.add(egui::Button::new("-").min_size(egui::Vec2::new(20.0, 20.0))).clicked() && v.pop().is_some() {
                                changed = true;
                            }
                        });
                    });
                }

                let mut final_res = ui.response();
                if let Some(body_res) = content_resp {
                    final_res = final_res.union(body_res);
                }
                if let Some(head_res) = header_resp {
                    final_res = final_res.union(head_res);
                }

                if changed {
                    final_res.mark_changed();
                }

                final_res
            },
        }).inner
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
        Self {
            name,
            data,
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
    fn easy_create_list_alt<T, U: Into<NbtLeafData>, F: Fn(&T) -> U>(v: &[T], f: F) -> Vec<Self> {
        v.iter().enumerate().map(|(i, v)| Self::new(format!("Item {i}"), f(v))).collect()
    }
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
