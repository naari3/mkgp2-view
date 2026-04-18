mod dolphin;

use dolphin::GameState;
use dolphin_memory::Dolphin;
use eframe::egui;
use std::collections::{HashMap, HashSet, VecDeque};
use std::time::{Duration, Instant};

fn main() -> eframe::Result {
    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_inner_size([1200.0, 800.0])
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

// --- Lap Timing (game-time based) ---

struct LapTracker {
    /// Game elapsed time when race timer started (first transition)
    race_start_time: Option<f64>,
    /// Game elapsed time at the start of each kart's current lap
    lap_starts: HashMap<u32, f64>,
    /// Completed lap times (seconds) per kart
    lap_times: HashMap<u32, Vec<f64>>,
    /// Previous values: remaining_laps for AI, current_lap for player
    prev_values: HashMap<u32, u32>,
    /// Slots awaiting their first (spurious) transition
    first_transition: HashSet<u32>,
    /// Slots that have finished the race
    finished_slots: HashSet<u32>,
    /// Player: previous g_totalRaceTime for exact lap time calculation
    prev_total_race_time: f64,
    was_racing: bool,
    finished: bool,
}

impl LapTracker {
    fn new() -> Self {
        Self {
            race_start_time: None,
            lap_starts: HashMap::new(),
            lap_times: HashMap::new(),
            prev_values: HashMap::new(),
            first_transition: HashSet::new(),
            finished_slots: HashSet::new(),
            prev_total_race_time: 0.0,
            was_racing: false,
            finished: false,
        }
    }

    fn reset(&mut self) {
        *self = Self::new();
    }

    const PLAYER_SLOT: u32 = 99;

    /// Game elapsed time = completed laps total + current lap time
    fn game_elapsed(state: &GameState) -> f64 {
        state.total_race_time as f64 + state.current_lap_time as f64
    }

    fn update(&mut self, state: &GameState, dolphin: &Option<Dolphin>) {
        if !state.race_started || state.player_ptr == 0 {
            if self.was_racing {
                self.was_racing = false;
                self.finished = false;
            }
            return;
        }

        if !self.was_racing {
            self.reset();
            self.was_racing = true;
            let game_time = Self::game_elapsed(state);
            self.race_start_time = Some(game_time);

            // Restore lap data from PathManager (mid-race startup, one-time read)
            self.restore_from_path_manager(state, dolphin);

            for ai in &state.ai_karts {
                self.prev_values.insert(ai.slot, ai.remaining_laps);
                // Only need first_transition if no restored data
                if !self.lap_times.contains_key(&ai.slot) {
                    self.first_transition.insert(ai.slot);
                }
                self.lap_starts.insert(ai.slot, game_time);
            }
            self.prev_values.insert(Self::PLAYER_SLOT, state.current_lap);
            if state.current_lap == 0 && !self.lap_times.contains_key(&Self::PLAYER_SLOT) {
                self.first_transition.insert(Self::PLAYER_SLOT);
            }
            self.lap_starts.insert(Self::PLAYER_SLOT, game_time);
            self.prev_total_race_time = state.total_race_time as f64;
            return;
        }

        let game_time = Self::game_elapsed(state);

        // Track AI (remaining_laps decreases) — continue even after player finishes
        for ai in &state.ai_karts {
            if !self.finished_slots.contains(&ai.slot) {
                if let Some(&prev) = self.prev_values.get(&ai.slot) {
                    if ai.remaining_laps < prev {
                        self.on_transition(ai.slot, game_time);
                    }
                }
                if ai.remaining_laps == 0 {
                    self.finished_slots.insert(ai.slot);
                }
            }
            self.prev_values.insert(ai.slot, ai.remaining_laps);
        }

        // Track player using g_totalRaceTime delta (exact game-internal timing)
        if !self.finished {
            if let Some(&prev_lap) = self.prev_values.get(&Self::PLAYER_SLOT) {
                if state.current_lap > prev_lap {
                    let total_rt = state.total_race_time as f64;
                    if self.first_transition.remove(&Self::PLAYER_SLOT) {
                        // Skip first crossing, set baseline
                        self.prev_total_race_time = total_rt;
                    } else {
                        // Lap completed — use totalRaceTime delta for exact timing
                        let lap_time = total_rt - self.prev_total_race_time;
                        if lap_time > 0.0 {
                            self.lap_times.entry(Self::PLAYER_SLOT).or_default().push(lap_time);
                        }
                        self.prev_total_race_time = total_rt;
                    }
                    self.lap_starts.insert(Self::PLAYER_SLOT, game_time);
                }
            }
            self.prev_values.insert(Self::PLAYER_SLOT, state.current_lap);

            if state.total_laps > 0 && state.current_lap > state.total_laps {
                self.finished = true;
                self.finished_slots.insert(Self::PLAYER_SLOT);
            }
        }
    }

    fn on_transition(&mut self, slot: u32, game_time: f64) {
        if self.first_transition.remove(&slot) {
            // First finish line crossing - skip, keep lap_starts at race_start_time
            // L1 will be measured from race start (same for all karts)
            return;
        }
        // Lap completed
        if let Some(&lap_start) = self.lap_starts.get(&slot) {
            let elapsed = game_time - lap_start;
            if elapsed > 0.0 {
                self.lap_times.entry(slot).or_default().push(elapsed);
            }
        }
        self.lap_starts.insert(slot, game_time);
    }

    /// Restore lap times from PathManager entries (for mid-race startup).
    fn restore_from_path_manager(&mut self, state: &GameState, dolphin: &Option<Dolphin>) {
        let Some(d) = dolphin else { return };
        let path_data = dolphin::read_path_lap_data(d);
        if path_data.is_empty() { return; }

        // Build km_ptr -> slot mapping
        let mut km_to_slot: HashMap<u32, u32> = HashMap::new();
        if state.player_ptr != 0 && state.player.km_ptr != 0 {
            km_to_slot.insert(state.player.km_ptr, Self::PLAYER_SLOT);
        }
        for ai in &state.ai_karts {
            if ai.render_obj_ptr != 0 {
                km_to_slot.insert(ai.render_obj_ptr, ai.slot);
            }
        }

        for pld in &path_data {
            if pld.lap_times.is_empty() { continue; }
            if let Some(&slot) = km_to_slot.get(&pld.km_ptr) {
                let times: Vec<f64> = pld.lap_times.iter().map(|&t| t as f64).collect();
                self.lap_times.insert(slot, times);
            }
        }

        // Adjust race_start_time if we have player data
        if self.lap_times.contains_key(&Self::PLAYER_SLOT) {
            let game_time = Self::game_elapsed(state);
            self.race_start_time = Some(game_time - state.total_race_time as f64 - state.current_lap_time as f64);
            self.prev_total_race_time = state.total_race_time as f64;
        }
    }

    /// Current game elapsed for display
    fn current_elapsed(&self, state: &GameState) -> Option<f64> {
        self.race_start_time
            .map(|start| Self::game_elapsed(state) - start)
    }

    /// Current lap elapsed for a specific slot
    fn current_lap_elapsed(&self, slot: u32, state: &GameState) -> Option<f64> {
        if slot == Self::PLAYER_SLOT {
            // Use game's own current lap timer for exact value
            Some(state.current_lap_time as f64)
        } else {
            self.lap_starts
                .get(&slot)
                .map(|&start| Self::game_elapsed(state) - start)
        }
    }

    fn format_time(secs: f64) -> String {
        let mins = (secs / 60.0) as u32;
        let s = secs - mins as f64 * 60.0;
        if mins > 0 {
            format!("{}:{:05.2}", mins, s)
        } else {
            format!("{:.2}s", s)
        }
    }
}

// --- Speed History ---

const SPEED_HISTORY_LEN: usize = 300; // ~10 seconds at 30fps polling

struct SpeedHistory {
    /// slot -> speed samples (newest at back)
    data: HashMap<u32, VecDeque<f64>>,
    sample_counter: u32,
}

impl SpeedHistory {
    fn new() -> Self {
        Self { data: HashMap::new(), sample_counter: 0 }
    }

    fn push(&mut self, slot: u32, speed: f64) {
        let buf = self.data.entry(slot).or_insert_with(|| VecDeque::with_capacity(SPEED_HISTORY_LEN));
        if buf.len() >= SPEED_HISTORY_LEN {
            buf.pop_front();
        }
        buf.push_back(speed);
    }

    fn get(&self, slot: u32) -> Option<&VecDeque<f64>> {
        self.data.get(&slot)
    }

    fn clear(&mut self) {
        self.data.clear();
        self.sample_counter = 0;
    }
}

// --- Pinned Item State ---

struct PinnedItemState {
    item_id: u32,
    roulette_category: u32, // ItemSelect slot index
}

struct AiPinnedItemState {
    item_id: u32,
}

// --- Minimap ---

#[derive(PartialEq, Clone, Copy)]
enum MinimapMode {
    Follow,  // auto-fit to current positions
    Accumulate, // remember historical min/max
}

struct MinimapState {
    mode: MinimapMode,
    acc_x_min: f32,
    acc_x_max: f32,
    acc_z_min: f32,
    acc_z_max: f32,
}

impl MinimapState {
    fn new() -> Self {
        Self {
            mode: MinimapMode::Follow,
            acc_x_min: f32::MAX,
            acc_x_max: f32::MIN,
            acc_z_min: f32::MAX,
            acc_z_max: f32::MIN,
        }
    }

    fn feed(&mut self, x: f32, z: f32) {
        self.acc_x_min = self.acc_x_min.min(x);
        self.acc_x_max = self.acc_x_max.max(x);
        self.acc_z_min = self.acc_z_min.min(z);
        self.acc_z_max = self.acc_z_max.max(z);
    }

    fn reset_accumulator(&mut self) {
        self.acc_x_min = f32::MAX;
        self.acc_x_max = f32::MIN;
        self.acc_z_min = f32::MAX;
        self.acc_z_max = f32::MIN;
    }
}

// --- App ---

struct App {
    dolphin: Option<Dolphin>,
    state: GameState,
    last_update: Instant,
    auto_connect: bool,
    lap_tracker: LapTracker,
    minimap: MinimapState,
    speed_history: SpeedHistory,
    /// AI previous positions for minimap trail (slot -> [x,y,z])
    ai_prev_pos: HashMap<u32, [f32; 3]>,
    /// Menu timer freeze toggle
    freeze_menu_timers: bool,
    /// Race timer freeze toggle
    freeze_race_timer: bool,
    /// Generic per-address value pinning
    freeze_map: dolphin::FreezeMap,
    /// Item pin: ro_ptr → snapshot of item state for restoration
    pinned_items: HashMap<u32, PinnedItemState>,
    /// AI item pin: kc_ptr → pinned item state
    ai_pinned_items: HashMap<u32, AiPinnedItemState>,
}

impl App {
    fn new() -> Self {
        Self {
            dolphin: None,
            state: GameState::default(),
            last_update: Instant::now(),
            auto_connect: true,
            lap_tracker: LapTracker::new(),
            minimap: MinimapState::new(),
            speed_history: SpeedHistory::new(),
            ai_prev_pos: HashMap::new(),
            freeze_menu_timers: false,
            freeze_race_timer: false,
            freeze_map: dolphin::FreezeMap::new(),
            pinned_items: HashMap::new(),
            ai_pinned_items: HashMap::new(),
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
                if self.state.connected {
                    self.state.path_lap_data = dolphin::read_path_lap_data(d);
                    // Apply all frozen values (includes timer freeze if active)
                    self.freeze_map.apply_all(d);
                    // Apply pinned items: after throw, trigger re-equip via ItemSelect
                    if let Ok(is_ptr) = dolphin::read_u32(d, dolphin::addr::G_ITEM_SELECT) {
                        if is_ptr > 0x80000000 && is_ptr < 0x81800000 {
                            // Safeguard: fix stuck rouletteActive when no pin is active
                            if self.pinned_items.is_empty() {
                                let ra = dolphin::read_u32(d, is_ptr + dolphin::addr::IS_ROULETTE_ACTIVE).unwrap_or(0);
                                let rd = dolphin::read_u32(d, is_ptr + dolphin::addr::IS_ROULETTE_DISPLAY_IDX).unwrap_or(0);
                                if ra > 0 && rd == 0xFFFFFFFF {
                                    // Stuck state: rouletteActive>0 but displayIdx=-1 → clear
                                    let _ = dolphin::write_u32(d, is_ptr + dolphin::addr::IS_ROULETTE_ACTIVE, 0);
                                }
                            }

                            if !self.pinned_items.is_empty() {
                                let ro_ptr = self.state.player.render_obj_ptr;
                                if ro_ptr > 0x80000000 && ro_ptr < 0x81800000 {
                                    let equipped = d.read_u8(((ro_ptr + dolphin::addr::RO_ITEM_EQUIPPED) & 0x01FFFFFF) as usize, None).unwrap_or(0);
                                    let in_use = d.read_u8(((ro_ptr + dolphin::addr::RO_ITEM_IN_USE) & 0x01FFFFFF) as usize, None).unwrap_or(0);
                                    let ra = dolphin::read_u32(d, is_ptr + dolphin::addr::IS_ROULETTE_ACTIVE).unwrap_or(0);
                                    // Safeguard: if equipped but rouletteActive stuck, clear it
                                    if equipped == 1 && ra > 0 {
                                        let _ = dolphin::write_u32(d, is_ptr + dolphin::addr::IS_ROULETTE_ACTIVE, 0);
                                    }
                                    if equipped == 0 && in_use == 0 {
                                        for (_, pin) in &self.pinned_items {
                                            let cat = pin.roulette_category;
                                            // Ensure slot has our item
                                            let slot_addr = is_ptr + dolphin::addr::IS_SLOT_ITEMS + cat * 8;
                                            let _ = dolphin::write_u32(d, slot_addr, pin.item_id);
                                            let _ = dolphin::write_u32(d, slot_addr + 4, 1);
                                            // Clear currentItemId so EquipItem doesn't reject same-item re-equip
                                            let _ = dolphin::write_u32(d, ro_ptr + dolphin::addr::RO_ITEM_ID, 0xFFFFFFFF);
                                            // Trigger re-equip (set both category AND displayIdx)
                                            let _ = dolphin::write_u32(d, is_ptr + dolphin::addr::IS_ROULETTE_CATEGORY, cat);
                                            let _ = dolphin::write_u32(d, is_ptr + dolphin::addr::IS_ROULETTE_DISPLAY_IDX, cat);
                                            let _ = dolphin::write_u32(d, is_ptr + dolphin::addr::IS_ROULETTE_ACTIVE, 1);
                                            let _ = dolphin::write_u32(d, is_ptr + dolphin::addr::IS_REMAINING_USES, 1);
                                        }
                                    }
                                }
                            }
                        }
                    }
                    // Apply AI pinned items: after use, trigger re-equip via KC pending item timer
                    if !self.ai_pinned_items.is_empty() {
                        for ai in &self.state.ai_karts {
                            if let Some(pin) = self.ai_pinned_items.get(&ai.kc_ptr) {
                                // Keep IH itemType frozen to block AI_GrantItem from natural pickup
                                // Only write when ih_state==0 to avoid interfering with cleanup
                                let ih = dolphin::read_u32(d, ai.kc_ptr + dolphin::addr::KC_ITEM_HOLDER).unwrap_or(0);
                                if ih != 0 {
                                    let ih_state = dolphin::read_u32(d, ih + dolphin::addr::IH_HAS_ITEM).unwrap_or(99);
                                    if ih_state == 0 {
                                        let _ = dolphin::write_u32(d, ih + dolphin::addr::IH_ITEM_TYPE, pin.item_id);
                                    }
                                }
                                let ro = ai.render_obj_ptr;
                                if ro > 0x80000000 && ro < 0x81800000 {
                                    let equipped = d.read_u8(((ro + dolphin::addr::RO_ITEM_EQUIPPED) & 0x01FFFFFF) as usize, None).unwrap_or(0);
                                    let in_use = d.read_u8(((ro + dolphin::addr::RO_ITEM_IN_USE) & 0x01FFFFFF) as usize, None).unwrap_or(0);
                                    if equipped == 0 && in_use == 0 {
                                        let _ = dolphin::write_u32(d, ai.kc_ptr + dolphin::addr::KC_PENDING_ITEM_ID, pin.item_id);
                                        let _ = dolphin::write_u32(d, ai.kc_ptr + dolphin::addr::KC_PENDING_ITEM_TIMER, 1);
                                    }
                                }
                            }
                        }
                    }
                } else {
                    self.dolphin = None;
                }
            }

            // Update AI prev positions for minimap
            if self.state.connected {
                for ai in &self.state.ai_karts {
                    self.ai_prev_pos.insert(ai.slot, ai.pos);
                }
            }

            // Update speed history (AI now reads velocity directly from PhysicsState)
            if self.state.connected && self.state.player_ptr != 0 {
                self.speed_history.push(LapTracker::PLAYER_SLOT, self.state.player.speed_magnitude() as f64);
                for ai in &self.state.ai_karts {
                    self.speed_history.push(ai.slot, ai.speed_kmh() as f64);
                }
                self.speed_history.sample_counter += 1;
            }

        }
    }

    fn render_minimap(&mut self, ui: &mut egui::Ui) {
        ui.heading("ミニマップ");

        // Mode selector
        ui.horizontal(|ui| {
            ui.selectable_value(&mut self.minimap.mode, MinimapMode::Follow, "追従");
            ui.selectable_value(&mut self.minimap.mode, MinimapMode::Accumulate, "累積");
            if self.minimap.mode == MinimapMode::Accumulate {
                if ui.small_button("リセット").clicked() {
                    self.minimap.reset_accumulator();
                }
            }
        });

        let available = ui.available_size();
        let size = (available.x - 8.0).min(available.y * 0.55).max(100.0);
        let (response, painter) =
            ui.allocate_painter(egui::vec2(size, size), egui::Sense::hover());
        let rect = response.rect;

        // Background
        painter.rect_filled(rect, 4.0, egui::Color32::from_gray(25));

        // Feed current positions into accumulator
        if self.state.player_ptr != 0 {
            self.minimap.feed(self.state.player.pos[0], self.state.player.pos[2]);
        }
        for ai in &self.state.ai_karts {
            self.minimap.feed(ai.pos[0], ai.pos[2]);
        }

        // Compute bounds based on mode
        let (x_min, x_max, z_min, z_max) = match self.minimap.mode {
            MinimapMode::Follow => {
                let mut xs = Vec::new();
                let mut zs = Vec::new();
                if self.state.player_ptr != 0 {
                    xs.push(self.state.player.pos[0]);
                    zs.push(self.state.player.pos[2]);
                }
                for ai in &self.state.ai_karts {
                    xs.push(ai.pos[0]);
                    zs.push(ai.pos[2]);
                }
                if xs.is_empty() { return; }
                (
                    xs.iter().copied().reduce(f32::min).unwrap(),
                    xs.iter().copied().reduce(f32::max).unwrap(),
                    zs.iter().copied().reduce(f32::min).unwrap(),
                    zs.iter().copied().reduce(f32::max).unwrap(),
                )
            }
            MinimapMode::Accumulate => {
                if self.minimap.acc_x_min > self.minimap.acc_x_max { return; }
                (self.minimap.acc_x_min, self.minimap.acc_x_max,
                 self.minimap.acc_z_min, self.minimap.acc_z_max)
            }
        };

        // Use the larger span for both axes (keep aspect ratio 1:1), with 10% margin
        let span = (x_max - x_min).max(z_max - z_min).max(100.0);
        let margin = span * 0.1;
        let cx = (x_min + x_max) * 0.5;
        let cz = (z_min + z_max) * 0.5;
        let half = span * 0.5 + margin;
        let world_min_x = cx - half;
        let world_max_x = cx + half;
        let world_min_z = cz - half;
        let world_max_z = cz + half;
        let world_range = world_max_x - world_min_x;

        // Grid: pick a nice interval
        let grid_interval = nice_grid_interval(world_range);
        let grid_start_x = (world_min_x / grid_interval).floor() as i32;
        let grid_end_x = (world_max_x / grid_interval).ceil() as i32;
        let grid_start_z = (world_min_z / grid_interval).floor() as i32;
        let grid_end_z = (world_max_z / grid_interval).ceil() as i32;

        let world_to_screen = |wx: f32, wz: f32| -> egui::Pos2 {
            let nx = (wx - world_min_x) / world_range;
            let nz = (wz - world_min_z) / world_range;
            egui::pos2(
                rect.left() + nx * rect.width(),
                rect.bottom() - nz * rect.height(),
            )
        };

        for i in grid_start_x..=grid_end_x {
            let wx = i as f32 * grid_interval;
            let t = (wx - world_min_x) / world_range;
            if t < 0.0 || t > 1.0 { continue; }
            let x = rect.left() + t * rect.width();
            let alpha = if i == 0 { 80 } else { 35 };
            painter.line_segment(
                [egui::pos2(x, rect.top()), egui::pos2(x, rect.bottom())],
                egui::Stroke::new(1.0, egui::Color32::from_white_alpha(alpha)),
            );
        }
        for i in grid_start_z..=grid_end_z {
            let wz = i as f32 * grid_interval;
            let t = (wz - world_min_z) / world_range;
            if t < 0.0 || t > 1.0 { continue; }
            let y = rect.bottom() - t * rect.height();
            let alpha = if i == 0 { 80 } else { 35 };
            painter.line_segment(
                [egui::pos2(rect.left(), y), egui::pos2(rect.right(), y)],
                egui::Stroke::new(1.0, egui::Color32::from_white_alpha(alpha)),
            );
        }

        let ai_colors = ai_slot_colors();
        let arrow_len = 25.0_f32; // fixed pixel length for heading arrows

        let draw_heading_arrow = |painter: &egui::Painter, pos: egui::Pos2, heading: f32, color: egui::Color32| {
            // Convention: Atan2(deltaX, deltaZ) → forward_x = sin(heading), forward_z = cos(heading)
            let fwd_x = heading.sin();
            let fwd_z = heading.cos();
            let dx = fwd_x * arrow_len;
            let dz = -fwd_z * arrow_len; // Z is flipped on screen
            let tip = egui::pos2(pos.x + dx, pos.y + dz);
            painter.line_segment([pos, tip], egui::Stroke::new(2.0, color));
            // Arrowhead
            let nx = fwd_x;
            let nz = -fwd_z;
            let head = 4.0_f32;
            let p1 = egui::pos2(tip.x - head * nx + head * 0.4 * nz, tip.y - head * nz - head * 0.4 * nx);
            let p2 = egui::pos2(tip.x - head * nx - head * 0.4 * nz, tip.y - head * nz + head * 0.4 * nx);
            painter.add(egui::Shape::convex_polygon(vec![tip, p1, p2], color, egui::Stroke::NONE));
        };

        // Draw AI karts (heading from PhysicsState+0x84)
        for (i, ai) in self.state.ai_karts.iter().enumerate() {
            let pos = world_to_screen(ai.pos[0], ai.pos[2]);
            let color = ai_colors[i % ai_colors.len()];
            painter.circle_filled(pos, 4.0, color);
            draw_heading_arrow(&painter, pos, ai.heading, color);
        }

        // Draw player (use velocity direction as fallback since player has no PhysicsState heading)
        if self.state.player_ptr != 0 {
            let p = &self.state.player;
            let pos = world_to_screen(p.pos[0], p.pos[2]);
            painter.circle_filled(pos, 4.0, egui::Color32::GREEN);
            painter.circle_stroke(pos, 4.0, egui::Stroke::new(1.0, egui::Color32::WHITE));
            let vel_mag = (p.velocity[0] * p.velocity[0] + p.velocity[2] * p.velocity[2]).sqrt();
            if vel_mag > 0.01 {
                let heading = p.velocity[0].atan2(p.velocity[2]); // atan2(x, z) = game convention
                draw_heading_arrow(&painter, pos, heading, egui::Color32::GREEN);
            }
        }

        // Legend
        ui.add_space(4.0);
        ui.horizontal_wrapped(|ui| {
            ui.colored_label(egui::Color32::GREEN, "●");
            ui.label("P");
            for (i, ai) in self.state.ai_karts.iter().enumerate() {
                ui.add_space(4.0);
                ui.colored_label(ai_colors[i % ai_colors.len()], "●");
                ui.label(format!("S{}", ai.slot));
            }
        });
    }

    fn render_lap_times(&self, ui: &mut egui::Ui) {
        ui.heading("ラップタイム");

        if !self.lap_tracker.was_racing && !self.lap_tracker.finished {
            ui.label("(レース開始待ち)");
            return;
        }

        let finished = self.lap_tracker.finished;

        // Race elapsed time
        if finished {
            let total: f64 = self.lap_tracker.lap_times
                .get(&LapTracker::PLAYER_SLOT)
                .map_or(0.0, |v| v.iter().sum());
            ui.monospace(format!("完走: {}", LapTracker::format_time(total)));
        } else if let Some(elapsed) = self.lap_tracker.current_elapsed(&self.state) {
            ui.monospace(format!("経過: {}", LapTracker::format_time(elapsed)));
        }

        // Collect all tracked slots (always include all, even if no completed laps yet)
        let mut slots: HashSet<u32> = self.lap_tracker.lap_times.keys().copied().collect();
        slots.extend(self.lap_tracker.lap_starts.keys());
        let mut slots: Vec<_> = slots.into_iter().collect();
        slots.sort_by_key(|&s| if s == LapTracker::PLAYER_SLOT { 0 } else { s + 1 });

        if slots.is_empty() {
            return;
        }

        let max_completed = self
            .lap_tracker
            .lap_times
            .values()
            .map(|v| v.len())
            .max()
            .unwrap_or(0);

        let show_now = !finished;
        let any_finished = !self.lap_tracker.finished_slots.is_empty();
        let extra_cols = if show_now { 1 } else { 0 } + if any_finished { 1 } else { 0 };
        let num_cols = max_completed + extra_cols + 1;

        egui::Grid::new("lap_times_grid")
            .num_columns(num_cols)
            .spacing([6.0, 2.0])
            .show(ui, |ui| {
                ui.strong("");
                for lap in 1..=max_completed {
                    ui.strong(format!("L{}", lap));
                }
                if show_now {
                    ui.strong("now");
                }
                if any_finished {
                    ui.strong("合計");
                }
                ui.end_row();

                for &slot in &slots {
                    if slot == LapTracker::PLAYER_SLOT {
                        ui.colored_label(egui::Color32::GREEN, "P");
                    } else {
                        ui.label(format!("S{}", slot));
                    }

                    let completed = self.lap_tracker.lap_times.get(&slot);
                    let n_completed = completed.map_or(0, |v| v.len());
                    if let Some(times) = completed {
                        for &t in times {
                            ui.monospace(LapTracker::format_time(t));
                        }
                    }
                    for _ in n_completed..max_completed {
                        ui.label("");
                    }

                    if show_now {
                        if let Some(current) = self.lap_tracker.current_lap_elapsed(slot, &self.state) {
                            ui.colored_label(
                                egui::Color32::from_gray(160),
                                LapTracker::format_time(current),
                            );
                        } else {
                            ui.label("");
                        }
                    }

                    if any_finished {
                        if self.lap_tracker.finished_slots.contains(&slot) {
                            let total: f64 = completed.map_or(0.0, |v| v.iter().sum());
                            ui.strong(LapTracker::format_time(total));
                        } else {
                            ui.label("");
                        }
                    }

                    ui.end_row();
                }
            });
    }

    fn render_main_ui(&mut self, ui: &mut egui::Ui) {
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
            ui.separator();
            let prev_menu = self.freeze_menu_timers;
            ui.checkbox(&mut self.freeze_menu_timers, "メニュータイマー凍結");
            if !prev_menu && self.freeze_menu_timers {
                self.freeze_map.add_menu_timer_freeze();
            } else if prev_menu && !self.freeze_menu_timers {
                if let Some(ref d) = self.dolphin {
                    self.freeze_map.remove_menu_timer_freeze(d);
                }
            }
            let prev_race = self.freeze_race_timer;
            ui.checkbox(&mut self.freeze_race_timer, "レースタイマー凍結");
            if !prev_race && self.freeze_race_timer {
                self.freeze_map.add_race_timer_freeze();
            } else if prev_race && !self.freeze_race_timer {
                if let Some(ref d) = self.dolphin {
                    self.freeze_map.remove_race_timer_freeze(d);
                }
            }
            if !self.freeze_map.is_empty() {
                ui.label(format!("({}件固定中)", self.freeze_map.len()));
            }
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
        if self.state.race_started && self.state.remaining_time > 0.0 {
            ui.horizontal(|ui| {
                ui.label(format!(
                    "残り: {:.1}s | ラップボーナス: +{:.0}s",
                    self.state.remaining_time, self.state.lap_bonus_time
                ));
            });
        }
        ui.horizontal(|ui| {
            if self.state.player_ptr == 0 {
                ui.label("● レース外");
            } else if self.state.race_started {
                ui.colored_label(egui::Color32::GREEN, "● レース中");
            } else if self.state.countdown_phase == 0 {
                ui.colored_label(egui::Color32::YELLOW, "● GO!");
            } else if self.state.countdown_phase <= 3 {
                ui.colored_label(
                    egui::Color32::YELLOW,
                    format!("● カウントダウン {}", self.state.countdown_phase),
                );
            } else {
                ui.label("● 待機中");
            }
        });

        ui.separator();

        // Player kart state
        ui.heading("プレ��ヤー");
        Self::render_kart_state(ui, &self.state.player, self.state.race_started, self.state.total_laps, self.state.current_lap, &self.dolphin, &mut self.freeze_map, &mut self.pinned_items);

        if !self.state.ai_karts.is_empty() {
            ui.separator();
            ui.heading("AI");
            let ai_colors = ai_slot_colors();
            let ai_data: Vec<_> = self.state.ai_karts.iter().enumerate().map(|(i, ai)| {
                (i, ai.clone())
            }).collect();
            let total_laps = self.state.total_laps;
            egui::Grid::new("ai_grid")
                .num_columns(9)
                .spacing([6.0, 4.0])
                .show(ui, |ui| {
                    ui.strong("Slot");
                    ui.strong("キャラ");
                    ui.strong("順位");
                    ui.strong("ラップ");
                    ui.strong("速度");
                    ui.strong("目標速度");
                    ui.strong("アイテム");
                    ui.strong("使用判定");
                    ui.strong("位置");
                    ui.end_row();

                    for (i, ai) in &ai_data {
                        let color = ai_colors[i % ai_colors.len()];
                        let kc = ai.kc_ptr;
                        let ro = ai.render_obj_ptr;

                        ui.colored_label(color, format!("S{}", ai.slot));

                        // キャラ
                        ui.horizontal(|ui| {
                            if ro != 0 {
                                editable_named_u32(
                                    ui, ai.char_id, ro + dolphin::addr::RO_CHAR_ID,
                                    |id| dolphin::CHAR_NAMES.get(id as usize).copied().unwrap_or("???"),
                                    &self.dolphin, &mut self.freeze_map,
                                );
                            } else {
                                ui.label(ai.char_name());
                            }
                        });

                        // 順位
                        ui.horizontal(|ui| {
                            if ro != 0 {
                                editable_u32(ui, ai.race_position, ro + dolphin::addr::RO_RACE_POSITION, &self.dolphin, &mut self.freeze_map);
                            } else {
                                ui.monospace(format!("{}", ai.race_position + 1));
                            }
                        });

                        // ラップ
                        if total_laps > 0 && ai.remaining_laps <= total_laps + 1 {
                            let current = total_laps + 1 - ai.remaining_laps;
                            if current > total_laps {
                                ui.monospace("ゴール");
                            } else {
                                ui.monospace(format!("{}/{}", current, total_laps));
                            }
                        } else {
                            ui.monospace(format!("(残{})", ai.remaining_laps));
                        }

                        // 速度
                        ui.monospace(format!("{:.1}", ai.speed_kmh()));

                        // 目標速度
                        ui.horizontal(|ui| {
                            editable_f32(ui, ai.target_speed_min, kc + dolphin::addr::KC_TARGET_SPEED_MIN, &self.dolphin, &mut self.freeze_map);
                            ui.label("-");
                            editable_f32(ui, ai.target_speed_max, kc + dolphin::addr::KC_TARGET_SPEED_MAX, &self.dolphin, &mut self.freeze_map);
                        });

                        // アイテム
                        ui.horizontal(|ui| {
                            let ih = self.dolphin.as_ref()
                                .and_then(|d| dolphin::read_u32(d, kc + dolphin::addr::KC_ITEM_HOLDER).ok())
                                .unwrap_or(0);
                            let ai_pinned = self.ai_pinned_items.contains_key(&kc);
                            // Item display
                            if ai_pinned {
                                if let Some(pin) = self.ai_pinned_items.get(&kc) {
                                    ui.colored_label(PIN_COLOR, dolphin::item_name(pin.item_id as i32));
                                    // DragValue for editing pinned item ID
                                    let mut v = pin.item_id as f64;
                                    ui.scope(|ui| {
                                        subtle_drag_style(ui);
                                        ui.style_mut().visuals.override_text_color = Some(PIN_COLOR);
                                        let dv = egui::DragValue::new(&mut v).speed(1.0).prefix("id=");
                                        if ui.add(dv).changed() {
                                            if let Some(p) = self.ai_pinned_items.get_mut(&kc) {
                                                p.item_id = v as u32;
                                                // Update IH freeze too
                                                if ih != 0 {
                                                    self.freeze_map.set_u32(ih + dolphin::addr::IH_ITEM_TYPE, v as u32);
                                                }
                                            }
                                        }
                                    });
                                }
                            } else if ih != 0 && ai.has_item {
                                editable_named_u32(
                                    ui, ai.item_type as u32, ih + dolphin::addr::IH_ITEM_TYPE,
                                    |id| dolphin::item_name(id as i32),
                                    &self.dolphin, &mut self.freeze_map,
                                );
                            } else {
                                ui.monospace("なし");
                            }
                            // 固 pin button — after item name
                            let can_pin_ai = kc != 0;
                            if can_pin_ai {
                                let text = if ai_pinned {
                                    egui::RichText::new("P").monospace().size(9.0).color(PIN_COLOR)
                                } else {
                                    egui::RichText::new("P").monospace().size(9.0).color(egui::Color32::from_gray(80))
                                };
                                if ui.add(egui::Button::new(text).frame(false).min_size(egui::vec2(14.0, 12.0))).clicked() {
                                    if self.ai_pinned_items.contains_key(&kc) {
                                        self.ai_pinned_items.remove(&kc);
                                        if ih != 0 {
                                            self.freeze_map.remove(ih + dolphin::addr::IH_ITEM_TYPE);
                                        }
                                    } else {
                                        let item_id = if ai.item_type > 0 {
                                            ai.item_type as u32
                                        } else if ai.current_item_id >= 0 {
                                            ai.current_item_id as u32
                                        } else {
                                            1 // default: green shell
                                        };
                                        self.ai_pinned_items.insert(kc, AiPinnedItemState { item_id });
                                        if ih != 0 {
                                            self.freeze_map.set_u32(ih + dolphin::addr::IH_ITEM_TYPE, item_id);
                                        }
                                    }
                                }
                            }
                        });

                        // 使用判定
                        ui.horizontal(|ui| {
                            let ih = self.dolphin.as_ref()
                                .and_then(|d| dolphin::read_u32(d, kc + dolphin::addr::KC_ITEM_HOLDER).ok())
                                .unwrap_or(0);
                            let cd_addr = kc + dolphin::addr::KC_USE_INTERVAL_TIMER;
                            let tgt_addr = kc + dolphin::addr::KC_TARGETING_TIMER;
                            let force_use = self.freeze_map.contains(cd_addr) && self.freeze_map.contains(tgt_addr);
                            if ai.has_item {
                                if force_use {
                                    ui.colored_label(PIN_COLOR, "即使用");
                                } else if ai.use_interval_timer > 0 {
                                    ui.colored_label(egui::Color32::from_gray(120),
                                        format!("CD:{}", ai.use_interval_timer));
                                } else if ai.targeting_timer > 0 {
                                    ui.colored_label(egui::Color32::LIGHT_BLUE,
                                        format!("照準:{}", ai.targeting_timer));
                                } else {
                                    ui.colored_label(egui::Color32::LIGHT_RED, "発射可");
                                }
                            } else if force_use {
                                ui.colored_label(PIN_COLOR, "即使用");
                            } else {
                                ui.colored_label(egui::Color32::from_gray(60), "-");
                            }
                            // 即使用トグル (装の左)
                            let ih_cd_addr = if ih != 0 { ih + dolphin::addr::IH_COOLDOWN } else { 0 };
                            let btn_text = if force_use {
                                egui::RichText::new("即").monospace().size(9.0).color(PIN_COLOR)
                            } else {
                                egui::RichText::new("即").monospace().size(9.0).color(egui::Color32::from_gray(80))
                            };
                            if kc != 0 && ui.add(egui::Button::new(btn_text).frame(false).min_size(egui::vec2(12.0, 12.0))).clicked() {
                                if force_use {
                                    self.freeze_map.remove(cd_addr);
                                    self.freeze_map.remove(tgt_addr);
                                    if ih_cd_addr != 0 { self.freeze_map.remove(ih_cd_addr); }
                                } else {
                                    self.freeze_map.set_u32(cd_addr, 0);
                                    self.freeze_map.set_u32(tgt_addr, 0);
                                    if ih_cd_addr != 0 { self.freeze_map.set_u32(ih_cd_addr, 0); }
                                }
                            }
                            // 装クールダウン表示
                            if ai.ih_cooldown > 0 && !force_use {
                                ui.colored_label(egui::Color32::from_gray(80),
                                    format!("装:{}", ai.ih_cooldown));
                            }
                        });

                        // 位置
                        ui.horizontal(|ui| {
                            editable_vec3(
                                ui, ai.pos,
                                [kc + dolphin::addr::KC_POS_X, kc + dolphin::addr::KC_POS_Y, kc + dolphin::addr::KC_POS_Z],
                                &self.dolphin, &mut self.freeze_map,
                            );
                        });

                        ui.end_row();
                    }
                });
        }

        // Warp gate status
        if self.state.player_ptr != 0 && self.state.player.warp_context_ptr != 0 {
            ui.separator();
            ui.colored_label(egui::Color32::LIGHT_BLUE, format!(
                "ワープ: ctx=0x{:08X}", self.state.player.warp_context_ptr
            ));
        }
    }

    fn render_speed_graph(&self, ui: &mut egui::Ui) {
        ui.heading("速度グラフ");

        let width = ui.available_width();
        let height = 180.0_f32;
        let (response, painter) = ui.allocate_painter(egui::vec2(width, height), egui::Sense::hover());
        let rect = response.rect;

        painter.rect_filled(rect, 4.0, egui::Color32::from_gray(25));

        // Find max speed across all visible history + reference lines
        let mut max_speed = 1.0_f64;
        let all_slots: Vec<u32> = {
            let mut s = vec![LapTracker::PLAYER_SLOT];
            for ai in &self.state.ai_karts { s.push(ai.slot); }
            s
        };
        for &slot in &all_slots {
            if let Some(buf) = self.speed_history.get(slot) {
                for &v in buf { if v > max_speed { max_speed = v; } }
            }
        }
        // Include speedCap reference
        let player_cap = self.state.player.speed_cap as f64 * 3.6;
        if player_cap > max_speed { max_speed = player_cap; }
        max_speed *= 1.1;

        let colors = ai_slot_colors();

        // Draw grid lines
        let grid_interval = nice_grid_interval(max_speed as f32);
        let mut gy = grid_interval;
        while (gy as f64) < max_speed {
            let t = gy as f64 / max_speed;
            let y = rect.bottom() - t as f32 * rect.height();
            painter.line_segment(
                [egui::pos2(rect.left(), y), egui::pos2(rect.right(), y)],
                egui::Stroke::new(1.0, egui::Color32::from_white_alpha(30)),
            );
            painter.text(
                egui::pos2(rect.left() + 2.0, y - 10.0),
                egui::Align2::LEFT_TOP,
                format!("{:.0}", gy),
                egui::FontId::monospace(9.0),
                egui::Color32::from_white_alpha(80),
            );
            gy += grid_interval;
        }

        // Reference line: player speedCap * 3.6
        if player_cap > 0.0 {
            let y = rect.bottom() - (player_cap / max_speed) as f32 * rect.height();
            painter.line_segment(
                [egui::pos2(rect.left(), y), egui::pos2(rect.right(), y)],
                egui::Stroke::new(1.0, egui::Color32::from_rgba_unmultiplied(0, 255, 0, 60)),
            );
            painter.text(
                egui::pos2(rect.right() - 50.0, y - 10.0),
                egui::Align2::LEFT_TOP,
                format!("cap:{:.0}", player_cap),
                egui::FontId::monospace(8.0),
                egui::Color32::from_rgba_unmultiplied(0, 255, 0, 120),
            );
        }

        // Draw speed lines
        for (ai_idx, &slot) in all_slots.iter().enumerate() {
            if let Some(buf) = self.speed_history.get(slot) {
                if buf.len() < 2 { continue; }
                let color = if slot == LapTracker::PLAYER_SLOT {
                    egui::Color32::GREEN
                } else {
                    colors[(ai_idx - 1) % colors.len()]
                };
                let points: Vec<egui::Pos2> = buf.iter().enumerate().map(|(i, &v)| {
                    let x = rect.left() + (i as f32 / SPEED_HISTORY_LEN as f32) * rect.width();
                    let y = rect.bottom() - (v / max_speed) as f32 * rect.height();
                    egui::pos2(x, y)
                }).collect();
                for w in points.windows(2) {
                    painter.line_segment([w[0], w[1]], egui::Stroke::new(
                        if slot == LapTracker::PLAYER_SLOT { 2.0 } else { 1.0 },
                        color,
                    ));
                }
            }
        }

        // Current speed labels (right edge)
        let x_label = rect.right() - 40.0;
        let mut y_offset = rect.top() + 2.0;
        if self.state.player_ptr != 0 {
            painter.text(
                egui::pos2(x_label, y_offset),
                egui::Align2::LEFT_TOP,
                format!("P:{:.0}", self.state.player.speed_magnitude()),
                egui::FontId::monospace(9.0),
                egui::Color32::GREEN,
            );
            y_offset += 11.0;
        }
        for (i, ai) in self.state.ai_karts.iter().enumerate() {
            let color = colors[i % colors.len()];
            painter.text(
                egui::pos2(x_label, y_offset),
                egui::Align2::LEFT_TOP,
                format!("{}:{:.0}", i, ai.speed_kmh()),
                egui::FontId::monospace(9.0),
                color,
            );
            y_offset += 11.0;
        }
    }

    fn render_kart_state(
        ui: &mut egui::Ui,
        kart: &dolphin::KartState,
        race_started: bool,
        total_laps: u32,
        current_lap: u32,
        dolphin: &Option<Dolphin>,
        freeze_map: &mut dolphin::FreezeMap,
        pinned_items: &mut HashMap<u32, PinnedItemState>,
    ) {
        let km = kart.km_ptr;
        let co = kart.car_obj_ptr;

        egui::Grid::new(format!("kart_{}", kart.slot))
            .num_columns(2)
            .spacing([20.0, 4.0])
            .show(ui, |ui| {
                // キャラ (named)
                ui.label("キャラ:");
                ui.horizontal(|ui| {
                    editable_named_u32(
                        ui, kart.char_id, co + dolphin::addr::CAR_CHAR_ID,
                        |id| dolphin::CHAR_NAMES.get(id as usize).copied().unwrap_or("???"),
                        dolphin, freeze_map,
                    );
                });
                ui.end_row();

                // カート (named)
                ui.label("カート:");
                ui.horizontal(|ui| {
                    editable_named_u32(
                        ui, kart.kart_variant, co + dolphin::addr::CAR_KART_VARIANT,
                        |id| dolphin::kart_variant_name(id),
                        dolphin, freeze_map,
                    );
                });
                ui.end_row();

                // 順位
                ui.label("順位:");
                ui.horizontal(|ui| {
                    editable_u32(ui, kart.race_position, dolphin::addr::G_RACE_POSITION, dolphin, freeze_map);
                    ui.label("位");
                });
                ui.end_row();

                // ラップ
                if total_laps > 0 && current_lap > 0 {
                    ui.label("ラップ:");
                    if current_lap > total_laps {
                        ui.strong("ゴール");
                    } else {
                        ui.strong(format!("{}/{}", current_lap, total_laps));
                    }
                    ui.end_row();
                }

                // 速度 (read-only, computed)
                ui.label("速度:");
                ui.strong(format!("{:.1}", kart.speed_magnitude()));
                ui.end_row();

                // speedCap
                ui.label("speedCap:");
                ui.horizontal(|ui| {
                    editable_f32(ui, kart.speed_cap, km + dolphin::addr::KM_SPEED_CAP, dolphin, freeze_map);
                });
                ui.end_row();

                // travelProgress
                ui.label("travelProgress:");
                ui.horizontal(|ui| {
                    editable_f32(ui, kart.travel_progress, km + dolphin::addr::KM_TRAVEL_PROGRESS, dolphin, freeze_map);
                });
                ui.end_row();

                // speedTableIdx (named)
                ui.label("speedTableIdx:");
                ui.horizontal(|ui| {
                    editable_named_u32(
                        ui, kart.speed_table_index, km + dolphin::addr::KM_SPEED_TABLE_INDEX,
                        |id| match id {
                            0 => "通常",
                            4 => "スタートダッシュ",
                            5 => "ドリフトブースト",
                            6 => "アドバンスト",
                            _ => "???",
                        },
                        dolphin, freeze_map,
                    );
                });
                ui.end_row();

                // speedScale
                ui.label("speedScale:");
                ui.horizontal(|ui| {
                    editable_f32(ui, kart.speed_scale, km + dolphin::addr::KM_SPEED_SCALE, dolphin, freeze_map);
                });
                ui.end_row();

                // coinBonus
                ui.label("coinBonus:");
                ui.horizontal(|ui| {
                    editable_f32(ui, kart.coin_speed_bonus, km + dolphin::addr::KM_COIN_SPEED_BONUS, dolphin, freeze_map);
                });
                ui.end_row();

                // コイン
                ui.label("コイン:");
                ui.horizontal(|ui| {
                    editable_u32(ui, kart.coin_count, co + dolphin::addr::CAR_COIN_COUNT, dolphin, freeze_map);
                });
                ui.end_row();

                // アイテム (RenderObj + ISE snapshot pin)
                ui.label("アイテム:");
                ui.horizontal(|ui| {
                    let ro = kart.render_obj_ptr;
                    let pinned = ro != 0 && pinned_items.contains_key(&ro);
                    let display_id = if pinned {
                        pinned_items.get(&ro).unwrap().item_id as i32
                    } else {
                        kart.current_item_id
                    };
                    let name = dolphin::item_name(display_id);
                    if pinned {
                        ui.colored_label(PIN_COLOR, name);
                    } else if display_id >= 0 {
                        ui.colored_label(egui::Color32::YELLOW, name);
                    } else {
                        ui.label("なし");
                    }
                    // DragValue for item ID (editable even when no item)
                    let mut v = (if display_id >= 0 { display_id } else { 0 }) as f64;
                    ui.scope(|ui| {
                        subtle_drag_style(ui);
                        if pinned { ui.style_mut().visuals.override_text_color = Some(PIN_COLOR); }
                        let dv = egui::DragValue::new(&mut v).speed(1.0).prefix("id=");
                        if ui.add(dv).changed() && pinned {
                            if let Some(pin) = pinned_items.get_mut(&ro) {
                                pin.item_id = v as u32;
                            }
                        }
                    });
                    // Pin button — always available when ro is valid
                    if ro != 0 {
                        let text = if pinned {
                            egui::RichText::new("P").monospace().size(9.0).color(PIN_COLOR)
                        } else {
                            egui::RichText::new("-").monospace().size(9.0).color(egui::Color32::from_gray(80))
                        };
                        if ui.add(egui::Button::new(text).frame(false).min_size(egui::vec2(12.0, 12.0))).clicked() {
                            if pinned {
                                pinned_items.remove(&ro);
                            } else {
                                let item_id = if v as u32 > 0 { v as u32 }
                                    else if kart.current_item_id >= 0 { kart.current_item_id as u32 }
                                    else { 1 }; // default to item 1 if nothing set
                                let roulette_category = dolphin.as_ref()
                                    .and_then(|d| dolphin::read_u32(d, dolphin::addr::G_ITEM_SELECT).ok())
                                    .and_then(|is_ptr| dolphin.as_ref()
                                        .and_then(|d| dolphin::read_u32(d, is_ptr + dolphin::addr::IS_ROULETTE_CATEGORY).ok()))
                                    .map(|cat| if cat <= 5 { cat } else { 0 })
                                    .unwrap_or(0);
                                pinned_items.insert(ro, PinnedItemState {
                                    item_id,
                                    roulette_category,
                                });
                            }
                        }
                    }
                });
                ui.end_row();

                // mue
                ui.label("mue:");
                ui.horizontal(|ui| {
                    editable_f32(ui, kart.mue, km + dolphin::addr::KM_MUE, dolphin, freeze_map);
                });
                ui.end_row();

                // 位置
                ui.label("位置:");
                ui.horizontal(|ui| {
                    editable_vec3(
                        ui, kart.pos,
                        [km + dolphin::addr::KM_POS_X, km + dolphin::addr::KM_POS_Y, km + dolphin::addr::KM_POS_Z],
                        dolphin, freeze_map,
                    );
                });
                ui.end_row();

                // 移動量
                ui.label("移動量:");
                ui.horizontal(|ui| {
                    editable_vec3(
                        ui, kart.velocity,
                        [km + dolphin::addr::KM_VELOCITY_X, km + dolphin::addr::KM_VELOCITY_Y, km + dolphin::addr::KM_VELOCITY_Z],
                        dolphin, freeze_map,
                    );
                });
                ui.end_row();

                // 状態 (read-only flags)
                ui.label("状態:");
                let mut flags = Vec::new();
                if kart.is_airborne {
                    flags.push("空中");
                }
                match kart.speed_table_index {
                    4 => flags.push("ロケスタブースト"),
                    5 => flags.push("ドリフトブースト"),
                    6 => flags.push("アドバンストブースト"),
                    _ => {}
                }
                if (kart.speed_scale - 1.0).abs() > 0.01 {
                    flags.push("速度効果");
                }
                if let Some(anim) = dolphin::anim_state_name(kart.anim_state) {
                    flags.push(anim);
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
}

// --- Editable value UI helpers ---

/// Make DragValue look like inline text (no button frame).
fn subtle_drag_style(ui: &mut egui::Ui) {
    let v = &mut ui.style_mut().visuals;
    v.widgets.inactive.bg_fill = egui::Color32::TRANSPARENT;
    v.widgets.inactive.weak_bg_fill = egui::Color32::TRANSPARENT;
    v.widgets.inactive.bg_stroke = egui::Stroke::NONE;
    v.widgets.hovered.bg_fill = egui::Color32::from_white_alpha(15);
    v.widgets.hovered.bg_stroke = egui::Stroke::new(1.0, egui::Color32::from_white_alpha(40));
}

/// Pin color for text when pinned.
const PIN_COLOR: egui::Color32 = egui::Color32::from_rgb(100, 180, 255);

/// Editable u32 field with DragValue and pin toggle.
fn editable_u32(
    ui: &mut egui::Ui,
    current: u32,
    addr: u32,
    dolphin: &Option<Dolphin>,
    freeze_map: &mut dolphin::FreezeMap,
) {
    let pinned = freeze_map.contains(addr);
    let display = if pinned {
        if let Some(dolphin::FrozenValue::U32(v)) = freeze_map.get(addr) { *v } else { current }
    } else {
        current
    };
    let mut v = display as f64;
    ui.scope(|ui| {
        subtle_drag_style(ui);
        if pinned { ui.style_mut().visuals.override_text_color = Some(PIN_COLOR); }
        let dv = egui::DragValue::new(&mut v).speed(1.0);
        if ui.add(dv).changed() {
            let new_val = v as u32;
            if let Some(d) = dolphin { let _ = dolphin::write_u32(d, addr, new_val); }
            if pinned { freeze_map.set_u32(addr, new_val); }
        }
    });
    pin_button(ui, addr, dolphin::FrozenValue::U32(display), freeze_map);
}

/// Editable f32 field with DragValue and pin toggle.
fn editable_f32(
    ui: &mut egui::Ui,
    current: f32,
    addr: u32,
    dolphin: &Option<Dolphin>,
    freeze_map: &mut dolphin::FreezeMap,
) {
    let pinned = freeze_map.contains(addr);
    let display = if pinned {
        if let Some(dolphin::FrozenValue::F32(v)) = freeze_map.get(addr) { *v } else { current }
    } else {
        current
    };
    let mut v = display as f64;
    ui.scope(|ui| {
        subtle_drag_style(ui);
        if pinned { ui.style_mut().visuals.override_text_color = Some(PIN_COLOR); }
        let dv = egui::DragValue::new(&mut v).speed(0.1).max_decimals(4);
        if ui.add(dv).changed() {
            let new_val = v as f32;
            if let Some(d) = dolphin { let _ = dolphin::write_f32(d, addr, new_val); }
            if pinned { freeze_map.set_f32(addr, new_val); }
        }
    });
    pin_button(ui, addr, dolphin::FrozenValue::F32(display), freeze_map);
}

/// Editable f32 component WITHOUT pin button (for use inside vec3).
fn editable_f32_bare(
    ui: &mut egui::Ui,
    current: f32,
    addr: u32,
    pinned: bool,
    dolphin: &Option<Dolphin>,
    freeze_map: &mut dolphin::FreezeMap,
) {
    let display = if pinned {
        if let Some(dolphin::FrozenValue::F32(v)) = freeze_map.get(addr) { *v } else { current }
    } else {
        current
    };
    let mut v = display as f64;
    ui.scope(|ui| {
        subtle_drag_style(ui);
        if pinned { ui.style_mut().visuals.override_text_color = Some(PIN_COLOR); }
        let dv = egui::DragValue::new(&mut v).speed(0.1).max_decimals(1);
        if ui.add(dv).changed() {
            let new_val = v as f32;
            if let Some(d) = dolphin { let _ = dolphin::write_f32(d, addr, new_val); }
            if pinned { freeze_map.set_f32(addr, new_val); }
        }
    });
}

/// Editable vec3 as `(X, Y, Z)` with per-component pin buttons.
fn editable_vec3(
    ui: &mut egui::Ui,
    vals: [f32; 3],
    addrs: [u32; 3],
    dolphin: &Option<Dolphin>,
    freeze_map: &mut dolphin::FreezeMap,
) {
    ui.label("(");
    for i in 0..3 {
        let pinned = freeze_map.contains(addrs[i]);
        editable_f32_bare(ui, vals[i], addrs[i], pinned, dolphin, freeze_map);
        pin_button(ui, addrs[i], dolphin::FrozenValue::F32(vals[i]), freeze_map);
        if i < 2 { ui.label(","); }
    }
    ui.label(")");
}

/// Named u32 value: shows "name (id=N)" with DragValue for the ID and pin toggle.
fn editable_named_u32(
    ui: &mut egui::Ui,
    current: u32,
    addr: u32,
    name_fn: impl Fn(u32) -> &'static str,
    dolphin: &Option<Dolphin>,
    freeze_map: &mut dolphin::FreezeMap,
) {
    let pinned = freeze_map.contains(addr);
    let display = if pinned {
        if let Some(dolphin::FrozenValue::U32(v)) = freeze_map.get(addr) { *v } else { current }
    } else {
        current
    };
    let name = name_fn(display);
    if pinned {
        ui.colored_label(PIN_COLOR, name);
    } else {
        ui.label(name);
    }
    let mut v = display as f64;
    ui.scope(|ui| {
        subtle_drag_style(ui);
        if pinned { ui.style_mut().visuals.override_text_color = Some(PIN_COLOR); }
        let dv = egui::DragValue::new(&mut v).speed(1.0).prefix("id=");
        if ui.add(dv).changed() {
            let new_val = v as u32;
            if let Some(d) = dolphin { let _ = dolphin::write_u32(d, addr, new_val); }
            if pinned { freeze_map.set_u32(addr, new_val); }
        }
    });
    pin_button(ui, addr, dolphin::FrozenValue::U32(display), freeze_map);
}

/// Pin/unpin toggle button (small, inline).
fn pin_button(
    ui: &mut egui::Ui,
    addr: u32,
    current_val: dolphin::FrozenValue,
    freeze_map: &mut dolphin::FreezeMap,
) {
    let pinned = freeze_map.contains(addr);
    let text = if pinned {
        egui::RichText::new("P").monospace().size(9.0).color(PIN_COLOR)
    } else {
        egui::RichText::new("-").monospace().size(9.0).color(egui::Color32::from_gray(80))
    };
    let btn = egui::Button::new(text)
        .frame(false)
        .min_size(egui::vec2(12.0, 12.0));
    let resp = ui.add(btn);
    if resp.clicked() {
        if pinned {
            freeze_map.remove(addr);
        } else {
            match current_val {
                dolphin::FrozenValue::U32(v) => freeze_map.set_u32(addr, v),
                dolphin::FrozenValue::F32(v) => freeze_map.set_f32(addr, v),
            }
        }
    }
    if resp.hovered() {
        resp.on_hover_text(format!("0x{:08X}{}", addr, if pinned { " (固定中)" } else { "" }));
    }
}

/// Pick a round grid interval that gives ~4-6 lines across the range.
fn nice_grid_interval(range: f32) -> f32 {
    let rough = range / 5.0;
    let mag = 10.0_f32.powf(rough.log10().floor());
    let norm = rough / mag;
    let nice = if norm < 1.5 { 1.0 } else if norm < 3.5 { 2.0 } else if norm < 7.5 { 5.0 } else { 10.0 };
    nice * mag
}

fn ai_slot_colors() -> [egui::Color32; 5] {
    [
        egui::Color32::from_rgb(100, 130, 255),
        egui::Color32::from_rgb(255, 100, 100),
        egui::Color32::from_rgb(255, 220, 80),
        egui::Color32::from_rgb(80, 220, 220),
        egui::Color32::from_rgb(220, 100, 255),
    ]
}

impl eframe::App for App {
    fn ui(&mut self, ui: &mut egui::Ui, _frame: &mut eframe::Frame) {
        self.poll();
        self.lap_tracker.update(&self.state, &self.dolphin);
        ui.ctx()
            .request_repaint_after(Duration::from_millis(33));

        ui.columns(2, |cols| {
            egui::ScrollArea::vertical()
                .id_salt("main_scroll")
                .show(&mut cols[0], |ui| {
                    self.render_main_ui(ui);
                });

            egui::ScrollArea::vertical()
                .id_salt("right_scroll")
                .show(&mut cols[1], |ui| {
                    // Minimap and speed graph side by side
                    ui.columns(2, |inner| {
                        self.render_minimap(&mut inner[0]);
                        self.render_speed_graph(&mut inner[1]);
                    });
                    ui.separator();
                    self.render_lap_times(ui);
                });
        });
    }
}

