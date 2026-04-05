mod dolphin;

use dolphin::GameState;
use dolphin_memory::Dolphin;
use eframe::egui;
use std::time::Instant;

fn main() -> eframe::Result {
    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_inner_size([520.0, 600.0])
            .with_always_on_top(),
        ..Default::default()
    };
    eframe::run_native(
        "MKGP2 View",
        options,
        Box::new(|cc| {
            setup_japanese_font(&cc.egui_ctx);
            Ok(Box::new(App::new()))
        }),
    )
}

fn setup_japanese_font(ctx: &egui::Context) {
    let mut fonts = egui::FontDefinitions::default();

    // Try loading a Japanese system font
    let candidates = [
        "C:\\Windows\\Fonts\\meiryo.ttc",
        "C:\\Windows\\Fonts\\msgothic.ttc",
        "C:\\Windows\\Fonts\\YuGothR.ttc",
    ];

    for path in candidates {
        if let Ok(data) = std::fs::read(path) {
            fonts.font_data.insert(
                "jp".to_owned(),
                std::sync::Arc::new(egui::FontData::from_owned(data)),
            );
            fonts
                .families
                .entry(egui::FontFamily::Proportional)
                .or_default()
                .insert(1, "jp".to_owned());
            fonts
                .families
                .entry(egui::FontFamily::Monospace)
                .or_default()
                .push("jp".to_owned());
            break;
        }
    }

    ctx.set_fonts(fonts);
}

struct App {
    dolphin: Option<Dolphin>,
    state: GameState,
    last_update: Instant,
    auto_connect: bool,
}

impl App {
    fn new() -> Self {
        Self {
            dolphin: None,
            state: GameState::default(),
            last_update: Instant::now(),
            auto_connect: true,
        }
    }

    fn try_connect(&mut self) {
        match Dolphin::new() {
            Ok(d) => {
                self.dolphin = Some(d);
                self.state.connected = true;
                self.state.error = None;
            }
            Err(e) => {
                self.dolphin = None;
                self.state.connected = false;
                self.state.error = Some(format!("接続失敗: {}", e));
            }
        }
    }

    fn poll(&mut self) {
        let now = Instant::now();
        if now.duration_since(self.last_update).as_millis() > 33 {
            self.last_update = now;

            if self.dolphin.is_none() && self.auto_connect {
                self.try_connect();
            }

            if let Some(ref d) = self.dolphin {
                self.state = dolphin::try_read_state(d);
                if !self.state.connected {
                    self.dolphin = None;
                }
            }
        }
    }
}

impl eframe::App for App {
    fn ui(&mut self, ui: &mut egui::Ui, _frame: &mut eframe::Frame) {
        self.poll();
        ui.ctx().request_repaint_after(std::time::Duration::from_millis(33));

        ui.heading("MKGP2 View");

        // Connection status
        ui.horizontal(|ui| {
            if self.state.connected {
                ui.colored_label(egui::Color32::GREEN, "● 接続中");
            } else {
                ui.colored_label(egui::Color32::RED, "● 未接続");
                if ui.button("接続").clicked() {
                    self.try_connect();
                }
            }
            ui.checkbox(&mut self.auto_connect, "自動再接続");
        });

        if let Some(ref err) = self.state.error {
            ui.colored_label(egui::Color32::YELLOW, err);
        }

        if !self.state.connected {
            return;
        }

        ui.separator();

        // Race info
        ui.horizontal(|ui| {
            ui.label(self.state.game_mode_name());
            ui.label("|");
            ui.strong(self.state.course_name());
            ui.label("|");
            ui.label(self.state.cc_label());
            if self.state.total_laps > 0 {
                ui.label(format!("| {}ラップ", self.state.total_laps));
            }
        });
        ui.horizontal(|ui| {
            if self.state.player_ptr == 0 {
                ui.label("● レース外");
            } else if self.state.race_started {
                ui.colored_label(egui::Color32::GREEN, "● レース中");
            } else if self.state.countdown_phase < 4 {
                ui.colored_label(egui::Color32::YELLOW, format!("● カウントダウン {}", 4 - self.state.countdown_phase));
            } else {
                ui.label("● 待機中");
            }
        });

        ui.separator();

        // Player kart state
        ui.heading("プレイヤー");
        render_kart_state(ui, &self.state.player, self.state.race_started);

        if !self.state.ai_karts.is_empty() {
            ui.separator();
            ui.heading("AI");
            egui::Grid::new("ai_grid")
                .num_columns(5)
                .spacing([8.0, 4.0])
                .show(ui, |ui| {
                    ui.strong("Slot");
                    ui.strong("順位");
                    ui.strong("ラップ");
                    ui.strong("目標速度");
                    ui.strong("位置");
                    ui.end_row();

                    for ai in &self.state.ai_karts {
                        ui.label(format!("S{}", ai.slot));
                        // race_position is 0-indexed (0=1st place)
                        ui.monospace(format!("{}位", ai.race_position + 1));
                        // remaining_laps is a countdown. Convert to current lap display.
                        let total = self.state.total_laps;
                        if total > 0 && ai.remaining_laps <= total + 1 {
                            let current = total + 1 - ai.remaining_laps;
                            ui.monospace(format!("{}/{}", current, total));
                        } else {
                            ui.monospace(format!("(残{})", ai.remaining_laps));
                        }
                        // Target speed is stored ×10000 scaled
                        ui.monospace(format!("{:.1}-{:.1}", ai.target_speed_min / 10000.0, ai.target_speed_max / 10000.0));
                        ui.monospace(format!(
                            "({:.0}, {:.0}, {:.0})",
                            ai.pos[0], ai.pos[1], ai.pos[2]
                        ));
                        ui.end_row();
                    }
                });
        }
    }
}

fn render_kart_state(ui: &mut egui::Ui, kart: &dolphin::KartState, race_started: bool) {
    egui::Grid::new(format!("kart_{}", kart.slot))
        .num_columns(2)
        .spacing([20.0, 4.0])
        .show(ui, |ui| {
            ui.label("キャラ:");
            ui.strong(format!("{} (id={})", kart.char_name(), kart.char_id));
            ui.end_row();

            // Player position/lap not yet available (KartMovement+0x23C/0x240 are different types for player)
            // TODO: find player ranking source

            ui.label("速度:");
            ui.strong(format!("{:.1}", kart.speed_magnitude()));
            ui.end_row();

            ui.label("speedCap:");
            ui.monospace(format!("{:.2}", kart.speed_cap));
            ui.end_row();

            ui.label("travelProgress:");
            ui.monospace(format!("{:.1}", kart.travel_progress));
            ui.end_row();

            ui.label("speedTableIdx:");
            let idx_text = match kart.speed_table_index {
                0 => "0 (通常)".to_string(),
                4 => "4 (スタートダッシュ)".to_string(),
                5 => "5 (ドリフトブースト)".to_string(),
                6 => "6 (アドバンスト)".to_string(),
                n => format!("{}", n),
            };
            ui.monospace(idx_text);
            ui.end_row();

            ui.label("speedScale:");
            ui.monospace(format!("{:.3}", kart.speed_scale));
            ui.end_row();

            ui.label("coinBonus:");
            ui.monospace(format!("{:.3}", kart.coin_speed_bonus));
            ui.end_row();

            ui.label("コイン:");
            ui.monospace(format!("{}", kart.coin_count));
            ui.end_row();

            ui.label("mue:");
            ui.monospace(format!("{:.4}", kart.mue));
            ui.end_row();

            ui.label("位置:");
            ui.monospace(format!(
                "({:.1}, {:.1}, {:.1})",
                kart.pos[0], kart.pos[1], kart.pos[2]
            ));
            ui.end_row();

            ui.label("状態:");
            let mut flags = Vec::new();
            if kart.is_airborne {
                flags.push("空中");
            }
            if kart.start_dash_ready && !race_started {
                flags.push("ロケスタ準備");
            }
            if flags.is_empty() {
                ui.label("通常");
            } else {
                ui.colored_label(egui::Color32::YELLOW, flags.join(", "));
            }
            ui.end_row();

            if kart.start_dash_frame > 0 {
                ui.label("ダッシュフレーム:");
                ui.monospace(format!("{}", kart.start_dash_frame));
                ui.end_row();
            }
        });
}
