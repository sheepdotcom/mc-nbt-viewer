use std::io::{self, Cursor};

use poll_promise::Promise;

use mc_nbt_viewer::{decompress_file, nbt::RootTag, parse_nbt_file, tree::NbtTree, world::World};

// https://github.com/c-git/egui_file_picker_poll_promise - example used for this, is also why the types are named this way
type SaveLoadReturn = Option<(Cursor<Vec<u8>>, String)>;
type SaveLoadPromise = Promise<SaveLoadReturn>;

#[derive(Default)]
pub struct App {
    root_tag: Option<RootTag>,
    nbt_tree: Option<NbtTree>,

    nbt_parsing_error_popup: bool,
    nbt_parsing_error: Option<io::Error>,

    save_load_promise: Option<SaveLoadPromise>,
}

impl App {
    /// Called once before the first frame.
    pub fn new(_cc: &eframe::CreationContext<'_>) -> Self {
        Self::default()
    }
}

impl eframe::App for App {
    fn logic(&mut self, _ctx: &egui::Context, _frame: &mut eframe::Frame) {
        if self.nbt_parsing_error.is_none() && self.nbt_parsing_error_popup {
            self.nbt_parsing_error_popup = false;
        }

        if let Some(promise) = &self.save_load_promise && promise.ready().is_some() {
            let mut temp = None;
            std::mem::swap(&mut temp, &mut self.save_load_promise);

            let maybe_data = temp.expect("Promise was in a state of ready and not ready at the same time.").block_and_take();

            if let Some((data, name)) = maybe_data {
                let root_tag = match decompress_file(data) {
                    Ok(mut v) => parse_nbt_file(&mut v, name),
                    Err(mut data) => parse_nbt_file(&mut data, name),
                };

                match root_tag {
                    Ok(v) => {
                        self.nbt_tree = Some(NbtTree::new(&v));
                        self.root_tag = Some(v);
                    },
                    Err(err) => {
                        self.nbt_parsing_error_popup = true;
                        self.nbt_parsing_error = Some(err);
                    },
                }
            }
        }
    }

    /// Called each time the UI needs repainting, which may be many times per second.
    fn ui(&mut self, ui: &mut egui::Ui, frame: &mut eframe::Frame) {
        // Put your widgets into a `SidePanel`, `TopBottomPanel`, `CentralPanel`, `Window` or `Area`.
        // For inspiration and more examples, go to https://emilk.github.io/egui

        egui::Panel::top("top_panel").show(ui, |ui| {
            // The top panel is often a good place for a menu bar:

            egui::MenuBar::new().ui(ui, |ui| {
                ui.menu_button("File", |ui| {
                    if ui.add_enabled(self.save_load_promise.is_none(), egui::Button::new("Open")).clicked() {
                        let ctx = ui.ctx().clone();
                        
                        self.save_load_promise = Some(execute(async move {
                            let file = rfd::AsyncFileDialog::new().pick_file().await?;
                            let data = Cursor::new(file.read().await);
                            let name = file.file_name();

                            ctx.request_repaint();

                            Some((data, name))
                        }));
                    }
                    
                    // NOTE: no File->Quit on web pages!
                    if !frame.is_web() && ui.button("Quit").clicked() {
                        ui.send_viewport_cmd(egui::ViewportCommand::Close);
                    }
                });

                ui.add_space(16.0);

                egui::widgets::global_theme_preference_buttons(ui);
            });
        });

        egui::Panel::bottom("bottom_menu").show(ui, |ui| {
            ui.horizontal(|ui| {
                powered_by_egui_and_eframe(ui);

                if cfg!(debug_assertions) {
                    ui.separator();
                    ui.label(egui::RichText::new("⚠ Debug build ⚠").small().color(ui.visuals().warn_fg_color)).on_hover_text("egui was compiled with debug assertions enabled.");
                }

                ui.with_layout(egui::Layout::right_to_left(egui::Align::TOP), |ui| {
                    ui.add(egui::github_link_file!(
                        "https://github.com/sheepdotcom/mc-nbt-viewer",
                        "Source code"
                    ));
                });
            });
        });

        let min_inspector_width = 250.0;

        // no longer resizable due to a bug with the panel's resizing mechanism
        // issue was the resize panel edge whatever has a min x as the width of the content, but the panel size rendering doesn't
        // which causes the panel to render smaller while the resize panel edge whatever would be farther to the right
        // TODO: custom panel resize so I can have proper resizing without that issue getting in my way
        egui::Panel::left("inspector_view").min_size(min_inspector_width + 16.0).resizable(false).show(ui, |ui| {
            ui.heading("Inspector View");
            
            if let Some(tree) = &mut self.nbt_tree {
                ui.add(tree);
            }
        });

        let min_world_width = 250.0;

        egui::Panel::right("world_view").min_size(min_world_width + 16.0).resizable(false).show(ui, |ui| {
            ui.heading("World View (?)");

            // I just learn't you can use else statements in a let statement on an enum, this is crazy
            let Some(render_state) = frame.wgpu_render_state() else {
                return;
            };

            let world = World::new(render_state, 100, 100);

            world.render();

            ui.add(egui::Image::from_texture(world.get_sized_texture()));

            // TODO: draw a triangle or a cube
        });

        egui::Window::new("NBT Parsing Error")
            .open(&mut self.nbt_parsing_error_popup)
            .auto_sized()
            .show(ui.ctx(), |ui| {
                if let Some(v) = &self.nbt_parsing_error {
                    ui.label(v.to_string());
                } else {
                    ui.label("Unknown");
                }
            });

        #[cfg(target_arch = "wasm32")]
        if crate::UPDATE_FLAG.load(std::sync::atomic::Ordering::Relaxed)
            && egui::Modal::new("update".into())
                .show(ui.ctx(), |ui| {
                    ui.heading("Update Available");
                    ui.label("An updated version has been found, refresh to get it(?).");
                    ui.label("// TODO: Test if refreshing works, and if you have to close ALL instances or only refresh the current one");
                    ui.vertical_centered_justified(|ui| ui.button("Close").clicked()).inner
                }).inner {
            crate::set_update_flag(false);
        }
    }
}

fn powered_by_egui_and_eframe(ui: &mut egui::Ui) {
    ui.horizontal(|ui| {
        ui.spacing_mut().item_spacing.x = 0.0;
        ui.label("Powered by ");
        ui.hyperlink_to("egui", "https://github.com/emilk/egui");
        ui.label(" and ");
        ui.hyperlink_to(
            "eframe",
            "https://github.com/emilk/egui/tree/master/crates/eframe",
        );
        ui.label(".");
    });
}

#[cfg(not(target_arch = "wasm32"))]
fn execute<F: std::future::Future<Output = SaveLoadReturn> + Send + 'static>(f: F) -> SaveLoadPromise {
    Promise::spawn_async(f)
}

#[cfg(target_arch = "wasm32")]
fn execute<F: std::future::Future<Output = SaveLoadReturn> + 'static>(f: F) -> SaveLoadPromise {
    Promise::spawn_local(f)
}
