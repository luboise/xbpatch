use std::path::PathBuf;

use xbpatch_core::patching::PatchEntry;

use crate::XBPatchApp;

#[derive(Debug)]
pub struct PatchSpecification {
    in_file: PathBuf,
    out_file: PathBuf,
    temp_folder: PathBuf,
    entries: Vec<PatchEntry>,
}

impl PatchSpecification {
    pub fn from_xbpatchapp(app: &XBPatchApp) -> Result<Self, Box<dyn std::error::Error>> {
        let mut spec = PatchSpecification {
            in_file: Default::default(),
            out_file: Default::default(),
            temp_folder: Default::default(),
            entries: Vec::new(),
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

        spec.temp_folder = match &app.temp_path {
            Some(t) => t.clone(),
            None => app.cwd_path.join("xbpatch_temp"),
        };

        spec.temp_folder =
            std::env::current_dir().unwrap_or(dirs_next::home_dir().unwrap().join("xbpatch_temp"));

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
}
