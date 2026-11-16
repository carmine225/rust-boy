use eframe::egui;
use std::path;
mod home;

struct GameEntry {
    title: String,
    subtitle: String,
    rom_path: path::PathBuf,
}

pub struct Emu {
    search_query: String,
    games: Vec<GameEntry>,
    vsync_enabled: bool,
    audio_enabled: bool,
    pixel_perfect: bool,
    directory_path: Option<path::PathBuf>,
}

impl Emu {
    pub fn new(cc: &eframe::CreationContext<'_>) -> Self {
        apply_dark_theme(&cc.egui_ctx);

        Self {
            search_query: String::new(),
            // Lista dei giochi della libreria: titolo, dettagli e percorso della ROM.
            games: vec![],
            vsync_enabled: true,
            audio_enabled: true,
            pixel_perfect: true,
            directory_path: None,
        }
    }
}

impl eframe::App for Emu {
    fn ui(&mut self, ui: &mut egui::Ui, _frame: &mut eframe::Frame) {
        ui.vertical(|ui| {
            ui.add_space(8.0);
            ui.horizontal(|ui| {
                ui.add_space(12.0);

                // Prima tendina: azioni per gestire la libreria dei giochi.
                ui.menu_button("Game", |ui| {
                    ui.set_min_width(160.0);
                    if ui.button("Add Game").clicked() {
                        println!("[INFO] Selezione singolo file ROM");
                        let rom_path = rfd::FileDialog::new()
                            .set_title("Seleziona una ROM del Game Boy")
                            .add_filter("ROM Game Boy", &["gb", "gbc", "gba"])
                            .add_filter("Tutti i file", &["*"])
                            .pick_file();
                        self.add_game(rom_path);
                        ui.close();
                    }
                    if ui.button("Add Folder").clicked() {
                        println!("[INFO] Selezione directory ROM");
                        self.directory_path = rfd::FileDialog::new()
                            .set_title("Seleziona una directory di ROM")
                            .pick_folder();
                        self.add_folder(self.directory_path.clone());
                        ui.close();
                    }
                    ui.separator();
                    if ui.button("Refresh Library").clicked() {
                        println!("[INFO] Aggiornamento libreria");
                        ui.close();
                    }
                });

                ui.add_space(32.0);

                egui::Frame::NONE
                    .fill(egui::Color32::from_rgb(22, 22, 27))
                    .corner_radius(6.0)
                    .stroke(egui::Stroke::new(1.0, egui::Color32::from_rgb(38, 38, 46)))
                    .inner_margin(egui::Margin::symmetric(10, 5))
                    .show(ui, |ui| {
                        ui.horizontal(|ui| {
                            ui.label(
                                egui::RichText::new("Search")
                                    .size(12.0)
                                    .color(egui::Color32::from_rgb(110, 110, 125)),
                            );
                            ui.add_space(4.0);
                            ui.add(
                                egui::TextEdit::singleline(&mut self.search_query)
                                    .hint_text("Filter by title...")
                                    .frame(egui::Frame::NONE)
                                    .desired_width(240.0),
                            );
                            if !self.search_query.is_empty() {
                                if ui
                                    .add(
                                        egui::Button::new(
                                            egui::RichText::new("×")
                                                .size(14.0)
                                                .color(egui::Color32::from_rgb(150, 150, 165)),
                                        )
                                        .frame(false),
                                    )
                                    .clicked()
                                {
                                    self.search_query.clear();
                                }
                            }
                        });
                    });

                // Seconda tendina: impostazioni audio, video e input.
                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    ui.add_space(12.0);

                    ui.menu_button("Settings", |ui| {
                        ui.set_min_width(200.0);
                        ui.label(
                            egui::RichText::new("System Configuration")
                                .size(12.0)
                                .strong()
                                .color(egui::Color32::from_rgb(180, 180, 195)),
                        );
                        ui.separator();

                        ui.checkbox(&mut self.audio_enabled, "APU Audio Output");
                        ui.checkbox(&mut self.vsync_enabled, "V-Sync (59.73 Hz)");
                        ui.checkbox(&mut self.pixel_perfect, "Integer Scaling");

                        ui.separator();
                        if ui.button("Input Mapping").clicked() {
                            println!("[INFO] Apertura configurazione input");
                            ui.close();
                        }
                        if ui.button("Display Palette").clicked() {
                            println!("[INFO] Apertura selezione palette DMG");
                            ui.close();
                        }
                    });
                });
            });

            ui.add_space(8.0);
            ui.separator();
            ui.add_space(14.0);

            ui.horizontal(|ui| {
                ui.add_space(16.0);
                ui.label(
                    egui::RichText::new("Library")
                        .size(15.0)
                        .strong()
                        .color(egui::Color32::from_rgb(225, 225, 235)),
                );

                let count = self
                    .games
                    .iter()
                    .filter(|g| {
                        g.title
                            .to_lowercase()
                            .contains(&self.search_query.to_lowercase())
                    })
                    .count();

                ui.label(
                    egui::RichText::new(format!("({})", count))
                        .size(13.0)
                        .color(egui::Color32::from_rgb(115, 115, 130)),
                );
            });

            ui.add_space(12.0);

            // Pannello centrale: filtra la lista in base al testo cercato.
            let filtered_games: Vec<&GameEntry> = self
                .games
                .iter()
                .filter(|g| {
                    g.title
                        .to_lowercase()
                        .contains(&self.search_query.to_lowercase())
                })
                .collect();

            egui::ScrollArea::vertical().show(ui, |ui| {
                ui.horizontal_wrapped(|ui| {
                    ui.add_space(16.0);

                    // Ciclo che stampa nel pannello centrale una scheda per ogni gioco filtrato.
                    for game in filtered_games {
                        ui.vertical(|ui| {
                            let card_id = ui.id().with(&game.rom_path);
                            let card_size = egui::vec2(140.0, 190.0);

                            // Riquadro della copertina del gioco (segnaposto NO COVER).
                            let frame_response = egui::Frame::NONE
                                .fill(egui::Color32::from_rgb(22, 22, 27))
                                .corner_radius(6.0)
                                .stroke(egui::Stroke::new(1.0, egui::Color32::from_rgb(36, 36, 44)))
                                .show(ui, |ui| {
                                    ui.set_min_size(card_size);
                                    ui.set_max_size(card_size);

                                    ui.centered_and_justified(|ui| {
                                        ui.label(
                                            egui::RichText::new("NO COVER")
                                                .size(11.0)
                                                .strong()
                                                .color(egui::Color32::from_rgb(75, 75, 90)),
                                        );
                                    });
                                })
                                .response;

                            let click_res =
                                ui.interact(frame_response.rect, card_id, egui::Sense::click());

                            if click_res.hovered() {
                                ui.ctx().set_cursor_icon(egui::CursorIcon::PointingHand);
                                ui.painter().rect_stroke(
                                    frame_response.rect,
                                    6.0,
                                    egui::Stroke::new(1.5, egui::Color32::from_rgb(100, 140, 240)),
                                    egui::StrokeKind::Inside,
                                );
                            }

                            if click_res.clicked() {
                                println!(
                                    "[EXEC] Avvio ROM: {} ({})",
                                    game.title,
                                    game.rom_path.display()
                                );
                            }

                            ui.add_space(6.0);

                            ui.set_max_width(card_size.x);
                            ui.add(
                                egui::Label::new(
                                    egui::RichText::new(&game.title)
                                        .size(13.0)
                                        .strong()
                                        .color(egui::Color32::from_rgb(230, 230, 238)),
                                )
                                .truncate(),
                            );

                            ui.label(
                                egui::RichText::new(&game.subtitle)
                                    .size(11.0)
                                    .color(egui::Color32::from_rgb(120, 120, 135)),
                            );
                        });

                        ui.add_space(18.0);
                    }
                });
            });
        });
    }
}

fn apply_dark_theme(ctx: &egui::Context) {
    let mut visuals = egui::Visuals::dark();
    visuals.window_fill = egui::Color32::from_rgb(13, 13, 16);
    visuals.panel_fill = egui::Color32::from_rgb(13, 13, 16);
    visuals.widgets.noninteractive.bg_fill = egui::Color32::from_rgb(22, 22, 27);
    visuals.widgets.inactive.bg_fill = egui::Color32::from_rgb(26, 26, 32);
    visuals.widgets.hovered.bg_fill = egui::Color32::from_rgb(38, 38, 46);
    visuals.widgets.active.bg_fill = egui::Color32::from_rgb(48, 48, 58);

    visuals.window_corner_radius = egui::CornerRadius::same(6);
    visuals.widgets.noninteractive.corner_radius = egui::CornerRadius::same(6);
    visuals.widgets.inactive.corner_radius = egui::CornerRadius::same(4);
    visuals.widgets.hovered.corner_radius = egui::CornerRadius::same(4);
    visuals.widgets.active.corner_radius = egui::CornerRadius::same(4);

    visuals.widgets.noninteractive.bg_stroke =
        egui::Stroke::new(1.0, egui::Color32::from_rgb(34, 34, 42));

    ctx.set_visuals(visuals);
}
