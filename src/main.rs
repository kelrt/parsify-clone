// TODO: UI changes to be a nicer experience
// TODO: exit + enter used to quit the application?
//

mod eval;

use eframe::egui;
use eval::{eval_line, format_num, Env};
use std::path::PathBuf;

fn save_path() -> PathBuf {
    let base = std::env::var_os("XDG_DATA_HOME")
        .map(PathBuf::from)
        .or_else(|| std::env::var_os("HOME").map(|h| PathBuf::from(h).join(".local/share")))
        .unwrap_or_else(|| PathBuf::from("."));
    base.join("calcpad").join("notes.txt")
}

struct App {
    text: String,
    dirty: bool,
}

impl App {
    fn load() -> Self {
        let text = std::fs::read_to_string(save_path()).unwrap_or_else(|_| {
            "# Type expressions, one per line\nprice = 19.99\nqty = 3\nprice * qty\nsqrt(16) + 2^3\nans * 2\n".to_string()
        });
        App { text, dirty: false }
    }

    fn save(&mut self) {
        let path = save_path();
        if let Some(dir) = path.parent() {
            let _ = std::fs::create_dir_all(dir);
        }
        let _ = std::fs::write(path, &self.text);
        self.dirty = false;
    }
}

impl eframe::App for App {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        let font = egui::FontId::monospace(16.0);
        let row_h = ctx.fonts(|f| f.row_height(&font));

        egui::CentralPanel::default().show(ctx, |ui| {
            egui::ScrollArea::vertical()
                .auto_shrink([false; 2])
                .show(ui, |ui| {
                    ui.horizontal_top(|ui| {
                        ui.spacing_mut().item_spacing = egui::vec2(12.0, 0.0);
                        let total = ui.available_width();

                        // Left: the editor. Wrapping is off so one text line == one result row.
                        let editor = egui::TextEdit::multiline(&mut self.text)
                            .font(font.clone())
                            .frame(false)
                            .desired_width(total * 0.62)
                            .desired_rows(30)
                            .lock_focus(true);
                        if ui.add(editor).changed() {
                            self.dirty = true;
                        }

                        // Right: one result per input line.
                        ui.vertical(|ui| {
                            ui.spacing_mut().item_spacing.y = 0.0;
                            let mut env = Env::new();
                            for line in self.text.split('\n') {
                                let (rect, _) = ui.allocate_exact_size(
                                    egui::vec2(ui.available_width(), row_h),
                                    egui::Sense::hover(),
                                );
                                let (txt, color) = match eval_line(line, &mut env) {
                                    None => (String::new(), egui::Color32::TRANSPARENT),
                                    Some(Ok(v)) => {
                                        (format_num(v), ui.visuals().strong_text_color())
                                    }
                                    Some(Err(e)) => (e, ui.visuals().weak_text_color()),
                                };
                                ui.painter().text(
                                    rect.left_center(),
                                    egui::Align2::LEFT_CENTER,
                                    txt,
                                    font.clone(),
                                    color,
                                );
                            }
                        });
                    });
                });
        });

        if self.dirty {
            self.save();
        }
    }
}

fn main() -> eframe::Result<()> {
    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_inner_size([820.0, 520.0])
            .with_app_id("calcpad"), // lets you target it in sway: for_window [app_id="calcpad"] ...
        ..Default::default()
    };
    eframe::run_native("Calcpad", options, Box::new(|_cc| Box::new(App::load())))
}
