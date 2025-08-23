#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")] // hide console window on Windows in release
#![allow(rustdoc::missing_crate_level_docs)] // it's an example

use eframe::egui;

use crate::file_handling::PatchSet;

mod file_handling;
mod types;

fn main() -> eframe::Result {
    env_logger::init(); // Log to stderr (if you run with `RUST_LOG=debug`).
    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default().with_inner_size([320.0, 240.0]),
        ..Default::default()
    };
    eframe::run_native(
        "XBPatch",
        options,
        Box::new(|cc| Ok(Box::<MyApp>::default())),
    )
}

enum ISOStatus {
    Valid,
    Unknown,
    Invalid,
}

struct MyApp {
    iso_path: String,
    iso_status: ISOStatus,
    patch_sets: Vec<PatchSet>,
    current_patch_set: u32,
}

impl Default for MyApp {
    fn default() -> Self {
        Self {
            iso_path: String::from(""),
            iso_status: ISOStatus::Unknown,
            patch_sets: Vec::new(),
            current_patch_set: 0,
        }
    }
}

impl eframe::App for MyApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
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
                                            ui.label("0/15");
                                            ui.label("ghoulies_main_patches.json");
                                            ui.end_row();

                                            ui.label("2/15");
                                            ui.label("ghoulies_graphics_patches.json");
                                            ui.end_row();
                                        });
                                });
                            },
                        );

                        ui.vertical(|ui| {
                            let button_size = egui::vec2(30.0, 30.0);

                            if ui.add_sized(button_size, egui::Button::new("+")).clicked() {
                                // TODO: Prompt for patch set
                            };

                            if ui.add_sized(button_size, egui::Button::new("-")).clicked() {
                                // TODO: Prompt for patch set
                            };
                        });
                    });
                });
            });
        });
    }
}

/*
fn main() {
    // Parse args
    let mut args = match parse_args(env::args()) {
        Ok(a) => a,
        Err(e) => match e {
            ArgError::InvalidArgState => {
                error_exit("Unable to proceed: Invalid arguments.");
            }
            ArgError::IsoSpecifiedMultipleTimes => {
                error_exit("Unable to proceed: Iso has been specified multiple times.");
            }
        },
    };

    // Extract the ISO
    let iso: PathBuf = args.iso_path.take().expect("Expected an iso path.");

    if !iso.exists() {
        error_exit("The iso provided does not exist.");
    } else if !iso.is_file() {
        error_exit("The iso provided is not a file.");
    }

    let extraction_dir: PathBuf = "./xbpatch_temp/isos/isoname".into();

    // If the folder exists and the user wants to delete it, then delete it
    if extraction_dir.exists() && extraction_dir.is_dir() {
        let skip_extraction = prompt_user_bool(format!(
            "Extraction dir {} already exists.\nWould you like to skip extraction and use this folder instead?",
            extraction_dir.to_str().unwrap()
        ));
        if !skip_extraction {
            std::fs::remove_dir_all(&extraction_dir).expect("Unable to delete existing folder.");
        }
    }

    // If the folder doesn't exist (or the user just deleted it), then extract the game
    if !extraction_dir.exists() {
        std::fs::create_dir_all(&extraction_dir).expect("Unable to create xbpatch dir.");

        xiso::extract_iso(&iso, &extraction_dir);
    }

    // Grab default.xbe out of it
    let xbe_path = match find_file_in_folder("default.xbe".into(), PathBuf::from(&extraction_dir)) {
        Some(x) => x,
        None => error_exit("Unable to find default.xbe in the .iso files."),
    };

    match backup_file(&xbe_path) {
        Ok(b) => b,
        Err(_) => {
            error_exit("Unable to backup default.xbe");
        }
    };

    // Parse the config
    //
    let mut xbe_writer =
        XBEWriter::new(&xbe_path).expect("Unable to create new XBE writer to apply patches.");

    let patches: Vec<GamePatch> = vec![
        GamePatch {
            name: String::from("Uncap frame rate 1"),
            offset: 0x154919,
            offset_type: GamePatchOffsetType::Virtual,
            replacement_bytes: vec![0xeb, 0x21],
            original_bytes: None,
        },
        GamePatch {
            name: String::from("Remove Relic store RNG"),
            offset: 0x81454,
            offset_type: GamePatchOffsetType::Virtual,
            replacement_bytes: vec![0x90, 0x90, 0x90, 0x90, 0x90, 0x90, 0x90, 0x90],
            original_bytes: None,
        },
    ];

    let mut successes = 0;
    let mut failures = 0;

    patches.iter().for_each(|p| {
        // TODO: Remove these unwraps
        print!("Applying patch \"{}\"...  ", &p.name);
        std::io::stdout().flush().expect("Unable to flush stdout");

        match xbe_writer.apply_patch(p) {
            Ok(_) => {
                successes += 1;
                println!("DONE!")
            }
            Err(_) => {
                failures += 1;
                println!("FAILED!\n        Failed to apply {}.", &p.name);
            }
        };
    });

    if failures == 0 {
        println!("All patches applied successfully.");
    } else {
        if successes == 0 {
            println!("Failed to apply all patches.");
            println!("Exiting now.");
            std::process::exit(0);
        }

        let should_continue = prompt_user_bool(format!(
            "Failed to apply {} patches. Would you like to write the iso anyway?",
            failures
        ));

        if !should_continue {
            println!("Exiting now.");
            std::process::exit(0);
        }
    }

    if let Some(parent_dir) = extraction_dir.parent() {
        let iso_path = parent_dir.join("isoname.iso");
        println!(
            "Writing new iso at {}",
            &iso_path.as_os_str().to_str().unwrap()
        );

        match xiso::create_iso(&iso_path, &extraction_dir) {
            Ok(_) => (),
            Err(e) => error_exit_with_details("Unable to create iso", e.to_string()),
        };
        println!(
            "\nSuccessfully wrote new iso to {}.",
            &iso_path.to_str().unwrap_or("(Error fetching ISO path)")
        );
    } else {
        error_exit("Unable to get parent dir of iso temp folder.");
    }

    println!("Exiting now.");
}
*/
