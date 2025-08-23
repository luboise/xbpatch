use eframe::egui::{Color32, Context, Id, Modal};

pub fn ask_user(ctx: &Context, id: &str, message: &String) -> Option<bool> {
    Modal::new(Id::new(id))
        .backdrop_color(Color32::from_black_alpha(100))
        .show(ctx, |ui| {
            ui.label(message);

            ui.horizontal(|ui| {
                if ui.button("OK").clicked() {
                    return Some(true);
                } else if ui.button("Cancel").clicked() {
                    return Some(false);
                } else {
                    return None;
                }
            });
        });

    None
}
