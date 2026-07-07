use poll_promise::Promise;

use crate::decompress_file;

// https://github.com/c-git/egui_file_picker_poll_promise - example used for this, is also why the types are named this way
type SaveLoadReturn = Option<Vec<u8>>;
type SaveLoadPromise = Option<Promise<SaveLoadReturn>>;

/// We derive Deserialize/Serialize so we can persist app state on shutdown.
#[derive(serde::Deserialize, serde::Serialize)]
#[serde(default)] // if we add new fields, give them default values when deserializing old state
pub struct App {
    #[serde(skip)]
    gz_loaded: bool, // temp value will be removed very soon

    #[serde(skip)] // This how you opt-out of serialization of a field
    save_load_promise: SaveLoadPromise,
}

impl Default for App {
    fn default() -> Self {
        Self {
            gz_loaded: false,
            save_load_promise: None,
        }
    }
}

impl App {
    /// Called once before the first frame.
    pub fn new(cc: &eframe::CreationContext<'_>) -> Self {
        // This is also where you can customize the look and feel of egui using
        // `cc.egui_ctx.set_visuals` and `cc.egui_ctx.set_fonts`.

        // Load previous app state (if any).
        // Note that you must enable the `persistence` feature for this to work.
        if let Some(storage) = cc.storage {
            eframe::get_value(storage, eframe::APP_KEY).unwrap_or_default()
        } else {
            Default::default()
        }
    }
}

impl eframe::App for App {
    /// Called by the framework to save state before shutdown.
    fn save(&mut self, storage: &mut dyn eframe::Storage) {
        eframe::set_value(storage, eframe::APP_KEY, self);
    }

    fn logic(&mut self, ctx: &egui::Context, frame: &mut eframe::Frame) {
        if let Some(promise) = &self.save_load_promise {
            if promise.ready().is_some() {
                let mut temp = None;
                std::mem::swap(&mut temp, &mut self.save_load_promise);

                let maybe_data = temp.expect("Promise was in a state of ready and not ready at the same time.").block_and_take();

                if let Some(data) = maybe_data {
                    // TODO: put through nbt decoding or whatever
                    self.gz_loaded = decompress_file(data).is_ok();
                }
            }
        }
    }

    /// Called each time the UI needs repainting, which may be many times per second.
    fn ui(&mut self, ui: &mut egui::Ui, _frame: &mut eframe::Frame) {
        // Put your widgets into a `SidePanel`, `TopBottomPanel`, `CentralPanel`, `Window` or `Area`.
        // For inspiration and more examples, go to https://emilk.github.io/egui

        egui::Panel::top("top_panel").show_inside(ui, |ui| {
            // The top panel is often a good place for a menu bar:

            egui::MenuBar::new().ui(ui, |ui| {
                ui.menu_button("File", |ui| {
                    if ui.add_enabled(self.save_load_promise.is_none(), egui::Button::new("Open")).clicked() {
                        let ctx = ui.ctx().clone();
                        
                        self.save_load_promise = execute(async move {
                            let file = rfd::AsyncFileDialog::new().pick_file().await?;
                            let data = file.read().await;

                            ctx.request_repaint();

                            Some(data)
                        });
                    }
                    
                    // NOTE: no File->Quit on web pages!
                    if !cfg!(target_arch = "wasm32") {
                        if ui.button("Quit").clicked() {
                            ui.send_viewport_cmd(egui::ViewportCommand::Close);
                        }
                    }
                });

                ui.add_space(16.0);

                egui::widgets::global_theme_preference_buttons(ui);
            });
        });

        egui::CentralPanel::default().show_inside(ui, |ui| {
            // The central panel the region left after adding TopPanel's and SidePanel's
            ui.heading("MC NBT Viewer (WIP)");
            
            ui.label(self.gz_loaded.to_string());

            ui.add(egui::github_link_file!(
                "https://github.com/sheepdotcom/mc-nbt-viewer/blob/main/",
                "Source code."
            ));

            ui.with_layout(egui::Layout::bottom_up(egui::Align::LEFT), |ui| {
                powered_by_egui_and_eframe(ui);
                egui::warn_if_debug_build(ui);
            });
        });
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
    Some(Promise::spawn_async(f))
}

#[cfg(target_arch = "wasm32")]
fn execute<F: std::future::Future<Output = SaveLoadReturn> + 'static>(f: F) -> SaveLoadPromise {
    Some(Promise::spawn_local(f))
}
