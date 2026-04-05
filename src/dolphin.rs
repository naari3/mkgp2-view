use dolphin_memory::Dolphin;

/// PPC addresses for MKGP2 game state
pub mod addr {
    // Global pointers
    pub const G_PLAYER_CAR_OBJECT: u32 = 0x806d1098;
    pub const G_CC_CLASS: u32 = 0x806d12cc;
    pub const G_COURSE_ID: u32 = 0x806cf108;
    pub const G_RACE_STARTED: u32 = 0x806d1260;
    pub const G_COUNTDOWN_PHASE: u32 = 0x806d1284;
    pub const G_TOTAL_LAPS: u32 = 0x806d1348;
    pub const G_TOTAL_RACERS: u32 = 0x806d1340;

    // AI kart controller pointers (5 slots)
    pub const AI_KART_CONTROLLERS: u32 = 0x80679598; // [0..4] = 4 bytes each

    // KartController offsets
    // KartController_GetPhysicsState returns *(kc+0x10), NOT +0x20
    // +0x20 is a different object used for physics ops in KartController_Update
    pub const KC_PHYSICS_STATE: u32 = 0x10;
    pub const KC_POS_X: u32 = 0x30;
    pub const KC_POS_Y: u32 = 0x34;
    pub const KC_POS_Z: u32 = 0x38;
    pub const KC_TARGET_SPEED_MIN: u32 = 0x8C;
    pub const KC_TARGET_SPEED_MAX: u32 = 0x90;

    // PhysicsState offsets (via KartController+0x20)
    pub const PS_RACE_POSITION: u32 = 0x23C;
    pub const PS_CURRENT_LAP: u32 = 0x240;

    // CarObject offsets
    pub const CAR_KART_SLOT: u32 = 0x10;
    pub const CAR_CHAR_ID: u32 = 0x14;
    pub const CAR_CC_CLASS: u32 = 0x18;
    pub const CAR_KART_VARIANT: u32 = 0x1C;
    pub const CAR_IS_PLAYER: u32 = 0x20;
    pub const CAR_KART_MOVEMENT: u32 = 0x28;
    pub const CAR_COIN_COUNT: u32 = 0xC8;
    pub const CAR_START_DASH_READY: u32 = 0xD5;
    pub const CAR_START_DASH_FRAME: u32 = 0xD8;

    // KartMovement offsets
    pub const KM_SPEED_TABLE_INDEX: u32 = 0x08;
    pub const KM_TRAVEL_PROGRESS: u32 = 0x0C;
    pub const KM_SPEED_CAP: u32 = 0x10;
    pub const KM_MUE: u32 = 0x38;
    pub const KM_POS_X: u32 = 0x40;
    pub const KM_POS_Y: u32 = 0x44;
    pub const KM_POS_Z: u32 = 0x48;
    pub const KM_VELOCITY_X: u32 = 0x17C;
    pub const KM_VELOCITY_Y: u32 = 0x180;
    pub const KM_VELOCITY_Z: u32 = 0x184;
    pub const KM_SPEED_SCALE: u32 = 0x2D8;
    pub const KM_COIN_SPEED_BONUS: u32 = 0x2E0;
    pub const KM_IS_AIRBORNE: u32 = 0x2BD;
}

pub const CHAR_NAMES: &[&str] = &[
    "マリオ",       // 0
    "ルイージ",     // 1
    "ピーチ",       // 2
    "ヨッシー",     // 3
    "キノピオ",     // 4
    "ワリオ",       // 5
    "ドンキー",     // 6
    "ワルイージ",   // 7
    "クッパ",       // 8
    "?9",           // 9 — パックマン or ミズパックマン (要確認)
    "アカベイ",     // 10
    "キノピコ",     // 11
    "まめっち",     // 12
    "?13",          // 13 — パックマン or ミズパックマン (要確認)
    "?14",          // 14 — 不明
];

pub const COURSE_NAMES: &[&str] = &[
    "(none)", "マリオ", "DK", "ワリオ", "パックマン",
    "クッパ", "レインボー", "ヨシー", "ワルイージ",
];

pub const CC_LABELS: &[&str] = &["50cc", "100cc", "150cc"];

#[derive(Default)]
pub struct KartState {
    pub slot: u32,
    pub char_id: u32,
    pub is_player: bool,
    pub coin_count: u32,
    pub speed_table_index: u32,
    pub travel_progress: f32,
    pub speed_cap: f32,
    pub pos: [f32; 3],
    pub velocity: [f32; 3],
    pub speed_scale: f32,
    pub coin_speed_bonus: f32,
    pub is_airborne: bool,
    pub mue: f32,
    pub start_dash_ready: bool,
    pub start_dash_frame: u32,
}

impl KartState {
    pub fn speed_magnitude(&self) -> f32 {
        let vx = self.velocity[0];
        let vy = self.velocity[1];
        let vz = self.velocity[2];
        (vx * vx + vy * vy + vz * vz).sqrt() * 3.6
    }

    pub fn char_name(&self) -> &'static str {
        CHAR_NAMES.get(self.char_id as usize).unwrap_or(&"???")
    }
}

#[derive(Default)]
pub struct AiKartState {
    pub slot: u32,
    pub pos: [f32; 3],
    pub target_speed_min: f32,
    pub target_speed_max: f32,
    pub race_position: u32,
    pub current_lap: u32,
}

#[derive(Default)]
pub struct GameState {
    pub connected: bool,
    pub cc_class: u32,
    pub course_id: u32,
    pub race_started: bool,
    pub countdown_phase: u32,
    pub total_laps: u32,
    pub player: KartState,
    pub ai_karts: Vec<AiKartState>,
    pub error: Option<String>,
}

impl GameState {
    pub fn course_name(&self) -> &'static str {
        COURSE_NAMES.get(self.course_id as usize).unwrap_or(&"???")
    }

    pub fn cc_label(&self) -> &'static str {
        CC_LABELS.get(self.cc_class as usize).unwrap_or(&"???")
    }
}

// Convenience wrappers that strip the PPC high bit and pass no pointer offsets
fn read_u32(dolphin: &Dolphin, addr: u32) -> Result<u32, String> {
    dolphin
        .read_u32((addr & 0x01FFFFFF) as usize, None)
        .map_err(|e| e.to_string())
}

fn read_f32(dolphin: &Dolphin, addr: u32) -> Result<f32, String> {
    dolphin
        .read_f32((addr & 0x01FFFFFF) as usize, None)
        .map_err(|e| e.to_string())
}

fn read_u8(dolphin: &Dolphin, addr: u32) -> Result<u8, String> {
    dolphin
        .read_u8((addr & 0x01FFFFFF) as usize, None)
        .map_err(|e| e.to_string())
}

pub fn try_read_state(dolphin: &Dolphin) -> GameState {
    let mut state = GameState {
        connected: true,
        ..Default::default()
    };

    match (|| -> Result<(), String> {
        state.cc_class = read_u32(dolphin, addr::G_CC_CLASS)?;
        state.course_id = read_u32(dolphin, addr::G_COURSE_ID)?;
        state.race_started = read_u8(dolphin, addr::G_RACE_STARTED)? != 0;
        state.countdown_phase = read_u32(dolphin, addr::G_COUNTDOWN_PHASE)?;
        state.total_laps = read_u32(dolphin, addr::G_TOTAL_LAPS)?;

        // Read player CarObject
        let player_ptr = read_u32(dolphin, addr::G_PLAYER_CAR_OBJECT)?;
        if player_ptr != 0 {
            state.player = read_kart_state(dolphin, player_ptr)?;
        }

        // Read AI kart controllers
        for i in 0u32..5 {
            let kc_ptr = read_u32(dolphin, addr::AI_KART_CONTROLLERS + i * 4)?;
            if kc_ptr == 0 {
                continue;
            }
            let ps_ptr = read_u32(dolphin, kc_ptr + addr::KC_PHYSICS_STATE)?;
            let ai = AiKartState {
                slot: i,
                pos: [
                    read_f32(dolphin, kc_ptr + addr::KC_POS_X)?,
                    read_f32(dolphin, kc_ptr + addr::KC_POS_Y)?,
                    read_f32(dolphin, kc_ptr + addr::KC_POS_Z)?,
                ],
                target_speed_min: read_f32(dolphin, kc_ptr + addr::KC_TARGET_SPEED_MIN)?,
                target_speed_max: read_f32(dolphin, kc_ptr + addr::KC_TARGET_SPEED_MAX)?,
                race_position: if ps_ptr != 0 {
                    read_u32(dolphin, ps_ptr + addr::PS_RACE_POSITION)?
                } else {
                    0
                },
                current_lap: if ps_ptr != 0 {
                    read_u32(dolphin, ps_ptr + addr::PS_CURRENT_LAP)?
                } else {
                    0
                },
            };
            state.ai_karts.push(ai);
        }

        Ok(())
    })() {
        Ok(()) => {}
        Err(e) => {
            state.error = Some(e);
        }
    }

    state
}

fn read_kart_state(dolphin: &Dolphin, car_obj: u32) -> Result<KartState, String> {
    let km_ptr = read_u32(dolphin, car_obj + addr::CAR_KART_MOVEMENT)?;
    if km_ptr == 0 {
        return Err("KartMovement is null".into());
    }

    Ok(KartState {
        slot: read_u32(dolphin, car_obj + addr::CAR_KART_SLOT)?,
        char_id: read_u32(dolphin, car_obj + addr::CAR_CHAR_ID)?,
        is_player: read_u8(dolphin, car_obj + addr::CAR_IS_PLAYER)? != 0,
        coin_count: read_u32(dolphin, car_obj + addr::CAR_COIN_COUNT)?,
        speed_table_index: read_u32(dolphin, km_ptr + addr::KM_SPEED_TABLE_INDEX)?,
        travel_progress: read_f32(dolphin, km_ptr + addr::KM_TRAVEL_PROGRESS)?,
        speed_cap: read_f32(dolphin, km_ptr + addr::KM_SPEED_CAP)?,
        pos: [
            read_f32(dolphin, km_ptr + addr::KM_POS_X)?,
            read_f32(dolphin, km_ptr + addr::KM_POS_Y)?,
            read_f32(dolphin, km_ptr + addr::KM_POS_Z)?,
        ],
        velocity: [
            read_f32(dolphin, km_ptr + addr::KM_VELOCITY_X)?,
            read_f32(dolphin, km_ptr + addr::KM_VELOCITY_Y)?,
            read_f32(dolphin, km_ptr + addr::KM_VELOCITY_Z)?,
        ],
        speed_scale: read_f32(dolphin, km_ptr + addr::KM_SPEED_SCALE)?,
        coin_speed_bonus: read_f32(dolphin, km_ptr + addr::KM_COIN_SPEED_BONUS)?,
        is_airborne: read_u8(dolphin, km_ptr + addr::KM_IS_AIRBORNE)? != 0,
        mue: read_f32(dolphin, km_ptr + addr::KM_MUE)?,
        start_dash_ready: read_u8(dolphin, car_obj + addr::CAR_START_DASH_READY)? != 0,
        start_dash_frame: read_u32(dolphin, car_obj + addr::CAR_START_DASH_FRAME)?,
    })
}
