#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")] // hide console window on Windows in release
#![allow(rustdoc::missing_crate_level_docs)] // it's an example

use std::{
    env,
    ffi::OsStr,
    path::{Path, PathBuf},
};

use eframe::egui::{self, Color32, Id, Modal, TextEdit};
use egui_file::FileDialog;

mod file_handling;
use crate::{file_handling::LoadedPatchSet, patching::PatchSpecification};

mod modals;
mod patching;

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
    UnrecognisedFiletype,
    FileDoesntExist,
    Unknown,
}

#[derive(PartialEq)]
enum XBPatchAppStatus {
    Startup,
    Patching,
    Normal,
    NeedReload,
    GettingNewPatchSetName,
    DeletionPrompt,
    SelectedInputISO,
    ConfirmingPatch,
}

struct XBPatchApp {
    status: XBPatchAppStatus,

    iso_finder_dialog: Option<FileDialog>,
    iso_finder: Option<PathBuf>,

    cwd_path: PathBuf,

    temp_path: Option<PathBuf>,
    temp_path_valid: bool,
    temp_input_str: String,

    input_iso_path: String,
    input_iso_status: ISOStatus,
    output_iso_path: String,
    output_iso_status: ISOStatus,

    patch_specification: Option<PatchSpecification>,

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
            // Paths
            cwd_path: std::env::current_dir().unwrap_or_default(),

            temp_path: Default::default(),
            temp_input_str: Default::default(),
            temp_path_valid: false,

            input_iso_path: Default::default(),
            input_iso_status: ISOStatus::Unknown,

            output_iso_path: Default::default(),
            output_iso_status: ISOStatus::Unknown,

            // Patch set folder
            patch_sets_path,
            loaded_patches: Vec::new(),
            current_patch_set: 0,
            modal_input: String::new(),
            iso_finder_dialog: None,
            iso_finder: None,
            patch_specification: None,
        }
    }
}

impl XBPatchApp {
    pub fn set_temp_directory(&mut self, str: &str) {
        let path: PathBuf = str.into();

        if path.is_dir() {
            self.temp_path = Some(path);
            self.temp_path_valid = true;
        } else {
            self.temp_path = None;
            self.temp_path_valid = false;
        }
    }

    pub fn update_state(&mut self, ctx: &egui::Context) {
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

                match env::current_dir() {
                    Ok(dir) => {
                        self.cwd_path = dir;
                        self.temp_input_str = self
                            .cwd_path
                            .join("xbpatch_temp")
                            .as_os_str()
                            .to_str()
                            .unwrap_or_default()
                            .to_string();

                        let new_temp = &self.temp_input_str.clone();
                        self.set_temp_directory(new_temp);
                    }
                    Err(_) => {
                        eprintln!("Unable to fetch current working directory.");
                    }
                };

                self.status = XBPatchAppStatus::Normal
            }
            XBPatchAppStatus::NeedReload => {
                // TODO: Handle failure here
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

                        let new_filename = LoadedPatchSet::filename_from_name(&self.modal_input);

                        ui.label(format!("The new filename will be\n{}", new_filename));

                        ui.horizontal(|ui| {
                            if ui.button("OK").clicked() {
                                println!("User entered new patch set: {}", &self.modal_input);

                                if let Some(p) = &self.patch_sets_path {
                                    let new_path = p.join(new_filename);

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
                if let Some(cps) = self.current_loaded_patch_set() {
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
                } else {
                    eprintln!(
                        "In deletion prompt state but no patch set is currently selected. Returning to normal state."
                    );
                    self.status = XBPatchAppStatus::Normal;
                }
            }
            XBPatchAppStatus::SelectedInputISO => {
                if let Ok(b) = std::fs::exists(&self.input_iso_path) {
                    if !b {
                        self.input_iso_status = ISOStatus::FileDoesntExist
                    } else if !self.input_iso_path.ends_with(".iso") {
                        self.input_iso_status = ISOStatus::UnrecognisedFiletype;
                    } else {
                        self.input_iso_status = ISOStatus::Valid;

                        let path_buf = PathBuf::from(&self.input_iso_path);

                        let stem = path_buf
                            .file_stem()
                            .unwrap_or_else(|| std::ffi::OsStr::new("file"));

                        self.output_iso_path = path_buf
                            .with_file_name(format!(
                                "{}_patched.iso",
                                stem.to_str().unwrap_or("file")
                            ))
                            .to_str()
                            .unwrap_or("error")
                            .into();
                    }
                } else {
                    self.input_iso_status = ISOStatus::Unknown;
                }

                self.status = XBPatchAppStatus::Normal;
            }
            XBPatchAppStatus::ConfirmingPatch => {
                let text =
                    "Are you sure you would like to patch the iso with these options?".into();
                modals::ask_user(ctx, "confirm_patch", &text, |b| {
                    if b {
                        self.status = XBPatchAppStatus::Patching
                    }
                });
            }
            XBPatchAppStatus::Patching => {
                println!("Do the patch here.");
            }
        };
    }

    pub fn has_current_patch_set(&self) -> bool {
        self.current_loaded_patch_set().is_some()
    }

    pub fn current_loaded_patch_set(&self) -> Option<&LoadedPatchSet> {
        let i = self.current_patch_set as usize;

        if i < self.loaded_patches.len() {
            return Some(&self.loaded_patches[i]);
        }

        None
    }

    pub fn current_loaded_patch_set_mut(&mut self) -> Option<&mut LoadedPatchSet> {
        let i = self.current_patch_set as usize;

        if i < self.loaded_patches.len() {
            return Some(&mut self.loaded_patches[i]);
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

    pub fn create_patch_spec(&self) -> Result<PatchSpecification, Box<dyn std::error::Error>> {
        let spec = PatchSpecification::from_xbpatchapp(self)?;
        Ok(spec)
    }

    fn loaded_patch_sets(&self) -> &[LoadedPatchSet] {
        &self.loaded_patches
    }

    fn status(&self) -> &XBPatchAppStatus {
        &self.status
    }

    fn status_mut(&mut self) -> &mut XBPatchAppStatus {
        &mut self.status
    }
}

impl eframe::App for XBPatchApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        self.update_state(ctx);

        egui::CentralPanel::default().show(ctx, |ui| {
            let width = ui.available_width();

            ui.vertical_centered(|ui| {
                ui.heading(format!("XBPatch v{}", env!("CARGO_PKG_VERSION")));
            });

            egui::SidePanel::right("patching_panel")
                .max_width(0.45 * width)
                .show_inside(ui, |ui| {
                    if let Some(mut_lps) = self.current_loaded_patch_set_mut() {
                        /*
                        if lps.data().len() == 0 {
                            ui.heading(format!("No patches in {}.", lps.data().name));
                            return;
                        }
                        */

                        // Make sure the number of bools matches the number of patches

                        // let enabled = self.enabled_entries();

                        // debug_assert_eq!(enabled.len(), lps.data().len());

                        egui::ScrollArea::vertical()
                            .auto_shrink(false)
                            .show(ui, |ui| {
                                egui::Grid::new("patches_viewer")
                                    .num_columns(1)
                                    .spacing([10.0, 4.0])
                                    .striped(false)
                                    .show(ui, |ui| {
                                        let len = mut_lps.enabled_entries_mut().len();

                                        for i in 0..len {
                                            if let Some(patch_entry) = mut_lps.get_patch_entry(i) {
                                                let mut_bool =
                                                    &mut mut_lps.enabled_entries_mut()[i];
                                                ui.checkbox(mut_bool, patch_entry.name())
                                                    .on_hover_text(patch_entry.description());

                                                ui.end_row();
                                            }
                                        }
                                    });
                            });
                    } else {
                        ui.heading("Current patch set is unavailable.");
                    }
                });

            egui::SidePanel::right("patch_sets_panel")
                .max_width(0.45 * width)
                .show_inside(ui, |ui| {
                    ui.heading("Patch Sets");

                    egui::ScrollArea::vertical()
                        .auto_shrink(true)
                        .show(ui, |ui| {
                            ui.horizontal(|ui| {
                                egui::Grid::new("patch_sets")
                                    .num_columns(2)
                                    .spacing([40.0, 4.0])
                                    .striped(false)
                                    .show(ui, |ui| {
                                        for (i, lps) in self.loaded_patches.iter().enumerate() {
                                            let patch_set = lps.data();

                                            // TODO: Implement highlight when mouse hovering

                                            if i as u32 == self.current_patch_set {
                                                let row_dims: egui::Rect =
                                                    ui.available_rect_before_wrap();
                                                ui.painter().rect_filled(
                                                    row_dims,
                                                    0.0,
                                                    ui.visuals().selection.bg_fill,
                                                );
                                            }

                                            let label_text = format!(
                                                "{}/{}",
                                                lps.enabled_entries()
                                                    .iter()
                                                    .filter(|v| { **v })
                                                    .count(),
                                                patch_set.len()
                                            );
                                            ui.label(label_text);
                                            if ui.label(&patch_set.name).clicked() {
                                                self.current_patch_set = i as u32;
                                            };
                                            ui.end_row();
                                        }
                                    });

                                ui.group(|ui| {
                                    ui.vertical(|ui| {
                                let button_size = egui::vec2(30.0, 30.0);

                                if ui
                                    .add_sized(button_size, egui::Button::new("+"))
                                    .on_hover_text("Create new patch set.")
                                    .clicked()
                                {
                                    if self.status == XBPatchAppStatus::Normal {
                                        self.modal_input.clear();
                                        self.status = XBPatchAppStatus::GettingNewPatchSetName;
                                    }
                                };

                                if ui.add_sized(button_size, egui::Button::new("-"))
                                    .on_hover_text("Delete selected patch set.")
                                    .clicked() {
                                    if self.status == XBPatchAppStatus::Normal {
                                        match self.current_loaded_patch_set() {
                                            Some(cps) => {
                                                self.status = XBPatchAppStatus::DeletionPrompt
                                            }
                                            None => {
                                                eprintln!(
                                        "Unable to delete when no patch set has been selected."
                                    );
                                            }
                                        };
                                    }
                                };

                                if ui.add_sized(button_size, egui::Button::new("⟳"))
                                    .on_hover_text("Refresh patches.")
                                    .clicked() {
                                    if self.status == XBPatchAppStatus::Normal {
                                        self.status = XBPatchAppStatus::NeedReload;
                                    }
                                };
                            });
                                })
                            });
                        });
                });

            ui.horizontal(|ui| {
                ui.heading("Input ISO");
                ui.group(|ui| {
                    ui.add(egui::TextEdit::singleline(&mut self.input_iso_path));
                    if ui.button("Choose").clicked() {
                        let filter = Box::new({
                            let ext = [OsStr::new("iso"), OsStr::new("xbe")];
                            move |path: &Path| -> bool {
                                ext.contains(
                                    &path
                                        .extension()
                                        .unwrap_or(OsStr::new("BAD_FILE_DONT_ACCEPT")),
                                )
                            }
                        });
                        let mut dialog = FileDialog::open_file(self.iso_finder.clone())
                            .show_files_filter(filter)
                            .title("Select an XBOX ISO");

                        dialog.set_path(
                            dirs_next::home_dir()
                                .unwrap_or_default()
                                .to_str()
                                .unwrap_or(""),
                        );

                        dialog.open();
                        self.iso_finder_dialog = Some(dialog);
                    }

                    if let Some(dialog) = &mut self.iso_finder_dialog {
                        if dialog.show(ctx).selected() {
                            if let Some(file) = dialog.path() {
                                self.input_iso_path = String::from(file.to_str().unwrap_or(""));
                            }

                            self.input_iso_status = ISOStatus::Unknown;
                            self.status = XBPatchAppStatus::SelectedInputISO;
                        }
                    }
                });
            });

            ui.horizontal(|ui| {
                ui.heading("Output ISO");
                ui.group(|ui| {
                    ui.add(egui::TextEdit::singleline(&mut self.output_iso_path));
                });

                let color = match self.input_iso_status {
                    ISOStatus::Valid => egui::Color32::GREEN,
                    ISOStatus::Unknown => egui::Color32::GRAY,
                    _ => egui::Color32::RED,
                };

                let text = match self.input_iso_status {
                    ISOStatus::Valid => "ISO located.",
                    ISOStatus::Unknown => "Unknown status.",
                    ISOStatus::UnrecognisedFiletype => "Unknown file type.",
                    ISOStatus::FileDoesntExist => "File could not be found.",
                };

                ui.colored_label(color, text);
            });

            ui.horizontal(|ui| {
                ui.heading("Temp Directory");
                ui.group(|ui| {
                    if ui
                        .add(egui::TextEdit::singleline(&mut self.temp_input_str))
                        .changed()
                    {
                        let str = self.temp_input_str.clone();
                        self.set_temp_directory(&str);
                    }
                });
            });
            ui.horizontal(|ui| {
                match self.temp_path_valid {
                    true => {
                        ui.colored_label(egui::Color32::GREEN, "Temp directory valid.");
                    }
                    false => {
                        ui.colored_label(egui::Color32::RED, "Unable to validate temp directory");
                        if ui.button("Create Temp Dir").clicked() {
                            let path: PathBuf = self.temp_input_str.clone().into();
                            match std::fs::create_dir_all(&path) {
                                Ok(_) => {
                                    let str = self.temp_input_str.clone();

                                    self.set_temp_directory(&str);
                                }

                                Err(e) => {
                                    eprintln!(
                                        "Unable to create temp directory at {}\nError: {}.",
                                        &path.display(),
                                        e
                                    );
                                }
                            };
                        }
                    }
                };
            });

            if ui.button("Patch").clicked() {
                if self.status == XBPatchAppStatus::Normal {
                    let patch_spec = self.create_patch_spec();
                }
            };
        });
    }
}
