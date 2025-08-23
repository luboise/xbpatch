use eframe::egui::{Color32, Context, Id, Modal};

pub fn ask_user<F>(ctx: &Context, id: &str, message: &String, on_answer: F)
where
    F: FnOnce(bool),
{
    Modal::new(Id::new(id))
        .backdrop_color(Color32::from_black_alpha(100))
        .show(ctx, |ui| {
            ui.label(message);

            ui.horizontal(|ui| {
                if ui.button("OK").clicked() {
                    on_answer(true);
                } else if ui.button("Cancel").clicked() {
                    on_answer(false);
                }
            });
        });
}
