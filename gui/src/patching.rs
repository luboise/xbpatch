use std::{
    ffi::OsStr,
    path::PathBuf,
    sync::{Arc, RwLock},
};

use walkdir::WalkDir;
use xbpatch_core::{
    iso_handling::{self, backup_file, restore_backup},
    patching::PatchEntry,
    xbe::{PatchReport, XBEWriter},
};

use crate::XBPatchApp;

#[derive(Debug, Clone)]
pub struct PatchSpecification {
    in_file: PathBuf,
    out_file: PathBuf,
    temp_folder: PathBuf,

    extract_xiso_path: PathBuf,

    entries: Vec<PatchEntry>,
    force_reextract: bool,
}

impl PatchSpecification {
    pub fn from_xbpatchapp(app: &XBPatchApp) -> Result<Self, Box<dyn std::error::Error>> {
        let mut spec = PatchSpecification {
            in_file: Default::default(),
            out_file: Default::default(),
            temp_folder: Default::default(),
            entries: Vec::new(),
            force_reextract: app.force_reextract,
            extract_xiso_path: Default::default(),
        };

        for loaded_set in app.loaded_patch_sets() {
            let enabled_entries = loaded_set.enabled_entries();

            for (i, _) in loaded_set
                .patch_set
                .entries
                .iter()
                .enumerate()
                .filter(|(i, _v)| enabled_entries.get(*i).cloned().unwrap_or(false))
            {
                spec.entries.push(loaded_set.patch_set.entries[i].clone());
            }
        }

        spec.in_file = PathBuf::from(&app.input_iso_path);
        spec.out_file = PathBuf::from(&app.output_iso_path);

        spec.extract_xiso_path = PathBuf::from(&app.extract_xiso_path);

        spec.temp_folder = match &app.temp_path {
            Some(t) => t.clone(),
            None => app.cwd_path.join("xbpatch_temp"),
        };

        Ok(spec)
    }

    pub fn in_file(&self) -> &PathBuf {
        &self.in_file
    }

    pub fn out_file(&self) -> &PathBuf {
        &self.out_file
    }

    pub fn temp_dir(&self) -> &PathBuf {
        &self.temp_folder
    }

    pub fn entries(&self) -> &[PatchEntry] {
        &self.entries
    }

    pub fn extract_xiso_path(&self) -> &PathBuf {
        &self.extract_xiso_path
    }
}

#[derive(Debug, Default)]
pub struct ThreadContext {
    completed: bool,
    error: bool,
    log: String,
}

impl ThreadContext {
    pub fn log(&self) -> &str {
        &self.log
    }

    pub fn completed(&self) -> bool {
        self.completed
    }

    pub fn error(&self) -> bool {
        self.error
    }
}

pub fn patch_iso_thread(ctx_lock: Arc<RwLock<ThreadContext>>, spec: PatchSpecification) {
    {
        let mut ctx = ctx_lock.write().unwrap();

        ctx.log.clear();
        ctx.completed = false;
        ctx.error = false;
    }

    // Helper functions for printing
    let ctx_print = |ctx_lock: &Arc<RwLock<ThreadContext>>, message: String| {
        let mut ctx = ctx_lock.write().unwrap();

        ctx.log.push('\n');
        ctx.log.push_str(message.as_ref());
    };

    let ctx_error = |ctx_lock: &Arc<RwLock<ThreadContext>>, msg: String| {
        ctx_print(ctx_lock, msg);

        let mut ctx = ctx_lock.write().unwrap();
        ctx.log.push_str("\n\nPatching failed.");
        ctx.completed = false;
        ctx.error = true;
    };

    ctx_print(
        &ctx_lock,
        format!(
            "Extracting {} to {}.",
            &spec.in_file.display(),
            &spec.temp_folder.display()
        ),
    );

    let extraction_path: PathBuf = spec
        .temp_folder
        .join("isos")
        .join(spec.in_file().file_stem().unwrap_or(OsStr::new("temp")));

    if extraction_path.exists() && extraction_path.is_dir() && spec.force_reextract {
        ctx_print(
            &ctx_lock,
            format!("Deleting existing directory {}", extraction_path.display()),
        );
        std::fs::remove_dir_all(&extraction_path).expect("Unable to delete existing folder.");
    }

    // Only extract if the folder doesn't exist
    if !extraction_path.exists() {
        ctx_print(&ctx_lock, "Extracting the iso...".to_string());

        match iso_handling::extract_iso(
            spec.extract_xiso_path.as_path(),
            &spec.in_file,
            &extraction_path,
        ) {
            Ok(_) => (),
            Err(e) => {
                ctx_error(&ctx_lock, format!("\nError during ISO extraction.\n{}", e));
                return;
            }
        }
    }

    ctx_print(&ctx_lock, String::from("Locating default.xbe..."));

    let xbe_path = match |file: PathBuf, path: &PathBuf| -> Option<PathBuf> {
        for entry in WalkDir::new(path) {
            if let Ok(entry) = entry {
                if entry.path().file_name() == file.file_name() {
                    return Some(entry.path().into());
                }
            }
        }
        None
    }("default.xbe".into(), &extraction_path)
    {
        Some(p) => p,
        None => {
            ctx_error(&ctx_lock, String::from("\nUnable to find default.xbe."));

            return;
        }
    };

    ctx_print(
        &ctx_lock,
        format!(
            "\ndefault.xbe located at {}\nBeginning patching...",
            xbe_path.display()
        ),
    );

    if restore_backup(&xbe_path)
        .expect("Failed to restore backup.")
        .is_none()
    {
        match backup_file(&xbe_path) {
            Ok(_) => (),
            Err(e) => ctx_error(&ctx_lock, format!("Failed to backup file. Error: {}", e)),
        };
    }

    // Parse the config
    let mut xbe_writer = match XBEWriter::new(&xbe_path) {
        Ok(w) => w,
        Err(_) => {
            ctx_error(
                &ctx_lock,
                format!(
                    "Unable to open {} for writing.",
                    &xbe_path.to_str().unwrap()
                ),
            );

            return;
        }
    };

    let mut report = PatchReport::default();

    for entry in spec.entries {
        ctx_print(
            &ctx_lock,
            format!("Applying patch \"{}\"...  ", entry.name()),
        );

        match xbe_writer.apply_patches(&entry) {
            Ok(patch_report) => {
                if patch_report.patch_successful() {
                    report.add_success();
                } else {
                    report.add_failure();
                    ctx_print(&ctx_lock, format!("FAILED to apply {}", entry.name()));
                }
            }
            Err(e) => {
                ctx_error(
                    &ctx_lock,
                    format!(
                        "A critical error occurred applying patches. Unable to continue. Error: {}",
                        e,
                    ),
                );
                return;
            }
        }
    }

    if report.failures() == 0 {
        println!("All patches applied successfully.");
    } else if report.successes() == 0 {
        ctx_error(&ctx_lock, String::from("Failed to apply all patches."));
        return;
    } else {
        ctx_print(
            &ctx_lock,
            String::from("Some patches failed to apply. Read above for details."),
        );
    }

    ctx_print(
        &ctx_lock,
        format!("Constructing new iso at {}", spec.out_file.display()),
    );

    match iso_handling::create_iso(
        spec.extract_xiso_path.as_path(),
        &spec.out_file,
        &extraction_path,
    ) {
        Ok(_) => {
            ctx_print(
                &ctx_lock,
                format!(
                    "Successfully wrote new iso to {}.",
                    &spec
                        .out_file
                        .to_str()
                        .unwrap_or("(Error fetching ISO path)")
                ),
            );
        }
        Err(e) => {
            ctx_error(&ctx_lock, format!("Failed to create iso. Error: {}", e));
            return;
        }
    };

    let mut ctx = ctx_lock.write().unwrap();
    ctx.log.push_str("\n\nPatching completed successfully.");
    ctx.error = false;
    ctx.completed = true;
}
