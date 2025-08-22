#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")] // hide console window on Windows in release
#![allow(rustdoc::missing_crate_level_docs)] // it's an example

use eframe::egui;

fn main() -> eframe::Result {
    env_logger::init(); // Log to stderr (if you run with `RUST_LOG=debug`).
    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default().with_inner_size([320.0, 240.0]),
        ..Default::default()
    };
    eframe::run_native(
        "My egui App",
        options,
        Box::new(|cc| Ok(Box::<MyApp>::default())),
    )
}

struct MyApp {
    name: String,
    age: u32,
}

impl Default for MyApp {
    fn default() -> Self {
        Self {
            name: "Arthur".to_owned(),
            age: 42,
        }
    }
}

impl eframe::App for MyApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        egui::CentralPanel::default().show(ctx, |ui| {
            ui.heading("My egui Application");
            ui.horizontal(|ui| {
                let name_label = ui.label("Your name: ");
                ui.text_edit_singleline(&mut self.name)
                    .labelled_by(name_label.id);
            });
            ui.add(egui::Slider::new(&mut self.age, 0..=120).text("age"));
            if ui.button("Increment").clicked() {
                self.age += 1;
            }
            ui.label(format!("Hello '{}', age {}", self.name, self.age));
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
