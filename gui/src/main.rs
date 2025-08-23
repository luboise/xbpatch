#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")] // hide console window on Windows in release
#![allow(rustdoc::missing_crate_level_docs)] // it's an example

use std::{env, path::PathBuf};

use eframe::egui::{self, Color32, Id, Modal, TextEdit};

use crate::file_handling::LoadedPatchSet;

mod file_handling;
mod modals;

fn main() -> eframe::Result {
    env_logger::init(); // Log to stderr (if you run with `RUST_LOG=debug`).
    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default().with_inner_size([1200.0, 800.0]),
        ..Default::default()
    };
    eframe::run_native(
        "XBPatch",
        options,
        Box::new(|cc| Ok(Box::<XBPatchApp>::default())),
    )
}

enum ISOStatus {
    Valid,
    Unknown,
    Invalid,
}

#[derive(PartialEq)]
enum XBPatchAppStatus {
    Startup,
    Normal,
    NeedReload,
    GettingNewPatchSetName,
    DeletionPrompt,
}

struct XBPatchApp {
    status: XBPatchAppStatus,
    iso_path: String,
    iso_status: ISOStatus,
    patch_sets_path: Option<PathBuf>,
    loaded_patches: Vec<LoadedPatchSet>,
    current_patch_set: u32,

    modal_input: String,
}

impl Default for XBPatchApp {
    fn default() -> Self {
        let patch_sets_path: Option<PathBuf> = match env::current_exe() {
            Ok(p) => {
                let mut data_folder = p.clone();
                data_folder.set_file_name("data");
                Some(data_folder)
            }
            Err(_) => None,
        };

        Self {
            status: XBPatchAppStatus::Startup,
            iso_path: String::new(),
            iso_status: ISOStatus::Unknown,
            patch_sets_path,
            loaded_patches: Vec::new(),
            current_patch_set: 0,
            modal_input: String::new(),
        }
    }
}

impl XBPatchApp {
    pub fn current_patch_set(&self) -> Option<&LoadedPatchSet> {
        let i = self.current_patch_set as usize;

        if i < self.loaded_patches.len() {
            return Some(&self.loaded_patches[i]);
        }
        None
    }
    pub fn reload_patch_sets(&mut self) -> Result<(), std::io::Error> {
        match &self.patch_sets_path {
            Some(p) => {
                if !p.exists() {
                    std::fs::create_dir_all(p)?;
                    return Ok(());
                }

                self.loaded_patches.clear();

                for entry in walkdir::WalkDir::new(p)
                    .into_iter()
                    .filter_map(Result::ok)
                    .filter(|e| e.path().is_file())
                {
                    if entry.path().is_dir() {
                        continue;
                    }

                    let path_buf: PathBuf = entry.path().into();

                    println!("Importing PatchSet from {}", path_buf.display());

                    let new_lps = match LoadedPatchSet::existing(&path_buf) {
                        Ok(l) => l,
                        Err(e) => {
                            println!(
                                "Unable to load PatchSet from {}\nDetails: {}",
                                path_buf.display(),
                                e
                            );
                            continue;
                        }
                    };

                    self.loaded_patches.push(new_lps);
                }
            }
            None => {
                eprintln!("Unable to reload patch sets when one hasn't been chosen.");
            }
        };

        Ok(())
    }
}

impl eframe::App for XBPatchApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        match self.status {
            XBPatchAppStatus::Startup => {
                println!(
                    "Importing patch sets from {}",
                    self.patch_sets_path
                        .as_ref()
                        .map(|p| p.display().to_string()) // map PathBuf -> String
                        .unwrap_or_else(|| "ERROR".to_string())
                );
                if self.loaded_patches.is_empty() {
                    // TODO: Handle failure here
                    self.reload_patch_sets();
                }

                self.status = XBPatchAppStatus::Normal
            }
            XBPatchAppStatus::NeedReload => {
                self.reload_patch_sets();
                self.status = XBPatchAppStatus::Normal;
            }
            XBPatchAppStatus::Normal => {}
            XBPatchAppStatus::GettingNewPatchSetName => {
                Modal::new(Id::new("Getting New Patch Set Name"))
                    .backdrop_color(Color32::from_black_alpha(100))
                    .show(ctx, |ui| {
                        ui.label("Please enter a name for the new patch set:");
                        ui.add(TextEdit::singleline(&mut self.modal_input));

                        ui.horizontal(|ui| {
                            if ui.button("OK").clicked() {
                                println!("User entered new patch set: {}", &self.modal_input);

                                if let Some(p) = &self.patch_sets_path {
                                    let new_path = p.join("name.json");

                                    match LoadedPatchSet::create_new(
                                        self.modal_input.clone(),
                                        &new_path,
                                    ) {
                                        Ok(_) => {
                                            match self.reload_patch_sets() {
                                                Ok(_) => (),
                                                Err(_) => {
                                                    eprintln!("Unable to reload patch sets.")
                                                }
                                            };
                                        }
                                        Err(_) => {
                                            eprintln!(
                                                "Unable to create new patch set at {}",
                                                new_path.display()
                                            );
                                        }
                                    };
                                };

                                self.status = XBPatchAppStatus::Normal;
                            }
                            if ui.button("Cancel").clicked() {
                                self.status = XBPatchAppStatus::Normal;
                                self.modal_input.clear();
                            }
                        });
                    });
            }
            XBPatchAppStatus::DeletionPrompt => {
                if let Some(cps) = self.current_patch_set() {
                    let text = format!(
                        "Are you sure that you would like to delete patch set {}\n{}",
                        cps.data().name,
                        cps.path().display()
                    );

                    let path = cps.path().clone();

                    modals::ask_user(ctx, "PatchSet Deletion", &text, |b| {
                        if b {
                            println!("Attempting to delete {}", path.display());
                            match std::fs::remove_file(&path) {
                                Ok(_) => {
                                    println!("Successfully deleted {}", &path.display());
                                    self.current_patch_set = 0;
                                }
                                Err(_) => {
                                    eprintln!("Unable to delete {}", path.display());
                                }
                            };
                        }

                        self.status = XBPatchAppStatus::NeedReload;
                    });
                }

                // eprintln!("In deletion prompt state but no patch set is currently selected.");
                // self.status = XBPatchAppStatus::Normal;
            }
        };

        egui::CentralPanel::default().show(ctx, |ui| {
            let width = ui.available_width();

            ui.vertical_centered(|ui| {
                ui.heading(format!("XBPatch v{}", env!("CARGO_PKG_VERSION")));
            });

            egui::SidePanel::right("scroll_test")
                .max_width(0.45 * width)
                .show_inside(ui, |ui| {
                    ui.label(
                        "The scroll area below has many labels with interactive tooltips. \
                 The purpose is to test that the tooltips close when you scroll.",
                    )
                    .on_hover_text("Try hovering a label below, then scroll!");
                    egui::ScrollArea::vertical()
                        .auto_shrink(false)
                        .show(ui, |ui| {
                            for i in 0..1000 {
                                ui.label(format!("This is line {i}")).on_hover_ui(|ui| {
                                    ui.style_mut().interaction.selectable_labels = true;
                                    ui.label(
                            "This tooltip is interactive, because the text in it is selectable.",
                        );
                                });
                            }
                        });
                });

            egui::CentralPanel::default().show_inside(ui, |ui| {
                ui.horizontal(|ui| {
                    ui.heading("ISO");

                    ui.group(|ui| {
                        ui.add(egui::TextEdit::singleline(&mut self.iso_path));
                        if ui.button("Choose").clicked() {
                            // TODO: Select file here
                        }
                    });

                    let color = match self.iso_status {
                        ISOStatus::Valid => egui::Color32::GREEN,
                        ISOStatus::Invalid => egui::Color32::RED,
                        ISOStatus::Unknown => egui::Color32::GRAY,
                    };

                    let text = match self.iso_status {
                        ISOStatus::Valid => "ISO located.",
                        ISOStatus::Unknown => "Unknown status.",
                        ISOStatus::Invalid => "Unable to locate ISO on disk.",
                    };

                    ui.colored_label(color, text);
                });

                ui.heading("Patch Sets");
                ui.group(|ui| {
                    ui.horizontal(|ui| {
                        ui.allocate_ui(
                            egui::Vec2::new(ui.available_width() * 0.8, ui.available_height()),
                            |ui| {
                                egui::ScrollArea::vertical().show(ui, |ui| {
                                    egui::Grid::new("patch_sets")
                                        .num_columns(2)
                                        .spacing([40.0, 4.0])
                                        .striped(false)
                                        .show(ui, |ui| {
                                            self.loaded_patches.iter().for_each(|lps| {
                                                let patch_set = lps.data();
                                                let label_text = format!("0/{}", patch_set.len());
                                                ui.label(label_text);
                                                ui.label(&patch_set.name);
                                                ui.end_row();
                                            });
                                        });
                                });
                            },
                        );

                        ui.vertical(|ui| {
                            let button_size = egui::vec2(30.0, 30.0);

                            if ui.add_sized(button_size, egui::Button::new("+")).clicked() {
                                if self.status == XBPatchAppStatus::Normal {
                                    self.status = XBPatchAppStatus::GettingNewPatchSetName;
                                    self.modal_input.clear();
                                }
                            };

                            if ui.add_sized(button_size, egui::Button::new("-")).clicked() {
                                if self.status == XBPatchAppStatus::Normal {
                                    match self.current_patch_set() {
                                        Some(cps) => self.status = XBPatchAppStatus::DeletionPrompt,
                                        None => {
                                            eprintln!("Unable to delete when no patch set has been selected.");
                                        }
                                    };
                                }
                            };
                        });
                    });
                });
            });
        });
    }
}
