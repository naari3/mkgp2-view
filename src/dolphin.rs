use dolphin_memory::Dolphin;
use std::collections::HashMap;

/// PPC addresses for MKGP2 game state
pub mod addr {
    // Timer freeze targets
    pub const G_GLOBAL_TIMER: u32 = 0x806d11d0; // int, frames (1200=20sec), decremented per frame
    // Race timer 1/60 rate constants — freezing these stops the race clock.
    // See mkgp2_timer_freeze_analysis.md: marked SAFE (no physics/AI impact).
    pub const RACE_TIMER_RATE_ADDRS: &[u32] = &[
        0x806d47bc, // VSMode_FrameUpdate (レースタイマー加算)
        0x806d47f0, // RaceMode_FrameUpdate (レースタイマー加算)
    ];

    // 1/60 rate constants — MENU/UI ONLY (no racing/minigame impact).
    // See mkgp2_timer_freeze_analysis.md for full 46-address survey.
    // Excluded: frame rate control, system tick, RaceMode/VSMode timers,
    //   sound/visual, item/warp init, sync/network, DashZone, etc.
    pub const DECREMENT_RATE_ADDRS: &[u32] = &[
        0x806d8108, // Sprite/アニメフェード
        0x806d8824, // result_screen_update
        0x806d9a78, // clFlowKart確認/フェード (選択画面)
        0x806d9af0, // cc選択→画面遷移
        0x806d9b90, // clFlowChara_HandleConfirm (キャラ選択)
        0x806d9ca0, // カップ選択UI
        0x806d9d44, // コース選択UI
        0x806d9df4, // ルーレット/Sprite演出
        0x806d9ef4, // コース一覧/選択UI
        0x806da140, // clFlowKart_HandleConfirm (カート選択)
        0x806db0f8, // 称号/リザルト計算
        0x806db7ac, // UI/Spriteヘルパー
        0x806db8ac, // UI/Spriteヘルパー
    ];

    // Global pointers
    pub const G_PLAYER_CAR_OBJECT: u32 = 0x806d1098;
    pub const G_CC_CLASS: u32 = 0x806d12cc;
    pub const G_COURSE_ID: u32 = 0x806cf108;
    pub const G_IS_URA_COURSE: u32 = 0x806d1268;
    pub const G_SUB_MODE: u32 = 0x806d1298; // 0-3=omote, 4-7=ura
    pub const G_GAME_MODE: u32 = 0x806d1294; // 0=Race, 1=Battle, 2=TA
    pub const G_RACE_STARTED: u32 = 0x806d1260;
    pub const G_COUNTDOWN_PHASE: u32 = 0x806d1284;
    pub const G_TOTAL_LAPS: u32 = 0x806d1348;
    pub const G_CURRENT_LAP_TIME: u32 = 0x806d1338; // float, seconds (resets each lap)
    pub const G_TOTAL_RACE_TIME: u32 = 0x806d133c; // float, seconds (completed laps sum)
    pub const G_REMAINING_TIME: u32 = 0x806d132c; // float, seconds (countdown)
    pub const G_LAP_BONUS_TIME: u32 = 0x806d1330; // float, seconds (fixed per race)
    pub const G_RACE_POSITION: u32 = 0x806d1340; // player's race position (0=1st)
    pub const G_CURRENT_LAP: u32 = 0x806d1344; // player's current lap (1-indexed)
    pub const G_PATH_MANAGER_RACE: u32 = 0x806d1364; // PathManager pointer (Race mode)

    // PathEntry offsets (0x98 bytes stride, 8 entries)
    pub const PE_STRIDE: u32 = 0x98;
    pub const PE_LAP_TIMES: u32 = 0x28;  // float[10]
    pub const PE_BEST_LAP: u32 = 0x50;   // float
    pub const PE_TOTAL_TIME: u32 = 0x5C; // float
    pub const PE_CURRENT_LAP: u32 = 0x60; // int (-1=pre, 0=lap1...)
    pub const PE_KART_MOVEMENT: u32 = 0xA0; // ptr

    // AI kart controller pointers (5 slots)
    pub const AI_KART_CONTROLLERS: u32 = 0x80679598; // [0..4] = 4 bytes each

    // KartController offsets
    // kc+0x10 = RenderObj (visual, 900 bytes). NOT PhysicsState.
    // kc+0x20 = PhysicsState (physics, 0x198 bytes)
    pub const KC_RENDER_OBJ: u32 = 0x10;
    pub const KC_PHYS_STATE: u32 = 0x20;
    pub const KC_POS_X: u32 = 0x30;
    pub const KC_POS_Y: u32 = 0x34;
    pub const KC_POS_Z: u32 = 0x38;
    pub const KC_ITEM_HOLDER: u32 = 0x24; // ItemHolder pointer
    pub const KC_TARGET_SPEED_MIN: u32 = 0x8C;
    pub const KC_TARGET_SPEED_MAX: u32 = 0x90;
    pub const KC_PENDING_ITEM_TIMER: u32 = 0x98; // int, countdown → 0 triggers ItemHolder_SetItem
    pub const KC_PENDING_ITEM_ID: u32 = 0x9C; // int, item to grant when timer reaches 0
    pub const KC_USE_INTERVAL_TIMER: u32 = 0xF0; // int, <=0 = can use
    pub const KC_TARGETING_TIMER: u32 = 0xEC;

    // ItemHolder offsets
    pub const IH_ITEM_TYPE: u32 = 0x1C; // int
    pub const IH_HAS_ITEM: u32 = 0x20; // int (0/1)
    pub const IH_COOLDOWN: u32 = 0x24; // int (frames)

    // PhysicsState offsets (via KartController+0x20)
    pub const PS_POS_X: u32 = 0x4C;
    pub const PS_POS_Y: u32 = 0x50;
    pub const PS_POS_Z: u32 = 0x54;
    pub const PS_SPEED: u32 = 0x68; // internal speed (divide by 10000 for km/h)
    pub const PS_HEADING: u32 = 0x84; // visual heading (cumulative radians). fwd_x=sin(h), fwd_z=cos(h)
    pub const PS_VELOCITY_X: u32 = 0x17C; // frame delta (pos - prevPos)
    pub const PS_VELOCITY_Y: u32 = 0x180;
    pub const PS_VELOCITY_Z: u32 = 0x184;

    // CarObject offsets
    pub const CAR_KART_SLOT: u32 = 0x10;
    pub const CAR_CHAR_ID: u32 = 0x14;
    pub const CAR_CC_CLASS: u32 = 0x18;
    pub const CAR_KART_VARIANT: u32 = 0x1C;
    pub const CAR_IS_PLAYER: u32 = 0x20;
    pub const CAR_KART_MOVEMENT: u32 = 0x28;
    pub const CAR_RENDER_OBJ: u32 = 0x2C; // RenderObj pointer (item master data)
    pub const CAR_COIN_COUNT: u32 = 0xC8;
    pub const CAR_WARP_CONTEXT: u32 = 0x54; // WarpAutoRunContext pointer
    pub const CAR_START_DASH_READY: u32 = 0xD5;
    pub const CAR_START_DASH_FRAME: u32 = 0xD8;

    // KartMovement offsets (player only, via CarObject+0x28)
    pub const KM_RACE_POSITION: u32 = 0x23C;
    pub const KM_CURRENT_LAP: u32 = 0x240;
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
    pub const KM_ACTIVE_ITEM_ENTRY: u32 = 0x340; // ptr to active ItemSlotEntry
    // ItemSlotEntry offsets (via deref KM+0x340)
    pub const ISE_STATE: u32 = 0x2C;    // int: 0=empty, 1=holding, 2=used/active
    pub const ISE_ITEM_ID: u32 = 0x34;  // int, item ID
    pub const KM_SPEED_SCALE: u32 = 0x2D8;
    pub const KM_COIN_SPEED_BONUS: u32 = 0x2E0;
    pub const KM_ANIM_STATE: u32 = 0x1F0; // int: animation/reaction state
    pub const KM_IS_AIRBORNE: u32 = 0x2BD;

    // RenderObj offsets (900 bytes at kc+0x10 for AI, CarObject+0x2C for player)
    pub const RO_CHAR_ID: u32 = 0x1F8;
    pub const RO_RACE_POSITION: u32 = 0x23C;
    pub const RO_CURRENT_LAP: u32 = 0x240;
    pub const RO_ITEM_IN_USE: u32 = 0x2B0;    // byte: 1=in use
    pub const RO_ITEM_EQUIPPED: u32 = 0x2B3;  // byte: 1=equipped
    pub const RO_ITEM_ID: u32 = 0x2C0;        // int: current item ID (master data, -1=none)
    pub const RO_ITEM_HOLD_POSE: u32 = 0x2A4; // int: hold pose (0=none)
    pub const RO_ACTIVE_SLOT_IDX: u32 = 0x2C8; // int: ISE slot index (-1=none)
    pub const RO_ISE_SLOTS: u32 = 0x2DC;       // ptr[10]: ISE slot pointers
    pub const ISE_ACTIVE: u32 = 0x04;           // byte: 1=active

    // ItemSelect (player item UI state, accessed via DAT_805d2af0)
    pub const G_ITEM_SELECT: u32 = 0x805d2af0;     // ptr to ItemSelect struct
    pub const IS_SLOT_ITEMS: u32 = 0x04;            // 8-byte entries: [itemId:u32, count:u32] × slots
    pub const IS_ROULETTE_CATEGORY: u32 = 0x54;     // int: active slot index (-1=none)
    pub const IS_ROULETTE_DISPLAY_IDX: u32 = 0x58;  // int: display index (copied to category on confirm)
    pub const IS_ROULETTE_ACTIVE: u32 = 0x64;       // int: countdown (1=equip next frame, 0=idle)
    pub const IS_REMAINING_USES: u32 = 0x6C;        // int: remaining uses (0 triggers consume)
}

// charId mapping confirmed in mkgp2docs/mkgp2_characters_and_karts.md
pub const CHAR_NAMES: &[&str] = &[
    "マリオ",           // 0
    "ルイージ",         // 1
    "ワリオ",           // 2
    "ピーチ",           // 3
    "クッパ",           // 4
    "キノピオ",         // 5
    "ドンキーコング",   // 6
    "ヨッシー",         // 7
    "パックマン",       // 8
    "ミズパックマン",   // 9
    "アカベイ",         // 10
    "ワルイージ",       // 11
    "まめっち",         // 12
    "?13",              // 13
    "?14",              // 14
];

pub const COURSE_NAMES: &[&str] = &[
    "(none)",              // 0
    "マリオカップ",         // 1
    "DKカップ",             // 2
    "ワリオカップ",         // 3
    "パックマンカ���プ",     // 4
    "クッパカップ",         // 5
    "レインボーカップ",     // 6
    "ヨッシー��ップ",       // 7
    "ワル��ージカップ",     // 8
    "巨大スイカを運べ！",   // 9  (マリオ)
    "ノコノコパニック！",   // 10 (DK)
    "���ックでレー���！",     // 11 (ワリオ)
    "パッ��チャレンジ",     // 12 (パックマン)
    "クッパを倒せ���",       // 13 (クッパ)
    "��ボマリオと一騎打ち！", // 14 (レイ���ボー)
    "コインコレクター！",   // 15 (ヨッシー)
    "鳥カートコンテスト！", // 16 (ワルイージ)
];

pub const GAME_MODE_NAMES: &[&str] = &["Race", "Battle", "TA"];

pub const CC_LABELS: &[&str] = &["50cc", "100cc", "150cc"];

#[derive(Default)]
pub struct KartState {
    pub car_obj_ptr: u32,    // CarObject address (for write-back)
    pub km_ptr: u32,         // KartMovement address (for PathEntry matching)
    pub render_obj_ptr: u32, // RenderObj address (item master data)
    pub slot: u32,
    pub char_id: u32,
    pub kart_variant: u32,
    pub is_player: bool,
    pub race_position: u32,
    pub remaining_laps: u32,
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
    pub anim_state: i32, // KM+0x1F0: -1=none, 0=idle, 0x17=projectile hit, etc.
    pub current_item_id: i32, // -1=none
    pub item_slot_entry_ptr: u32, // resolved ISE pointer (for item write-back)
    pub start_dash_ready: bool,
    pub start_dash_frame: u32,
    pub warp_context_ptr: u32,
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

#[derive(Default, Clone)]
pub struct AiKartState {
    pub kc_ptr: u32,         // KartController address (for write-back)
    pub render_obj_ptr: u32, // RenderObj at kc+0x10
    pub phys_state_ptr: u32, // PhysicsState at kc+0x20
    pub slot: u32,
    pub char_id: u32,
    pub pos: [f32; 3],
    pub velocity: [f32; 3], // from PhysicsState+0x17C (frame delta, for direction only)
    pub internal_speed: f32, // from PhysicsState+0x68 (divide by 10000 for km/h)
    pub heading: f32, // from PhysicsState+0x84 (cumulative radians). fwd_x=sin(h), fwd_z=cos(h)
    pub target_speed_min: f32,
    pub target_speed_max: f32,
    pub race_position: u32,
    pub remaining_laps: u32,
    pub item_type: i32,   // -1=none (from ItemHolder)
    pub has_item: bool,
    pub current_item_id: i32, // from RenderObj (same as player), -1=none
    pub use_interval_timer: i32,  // kc+0xF0: ≤0 = can use item
    pub targeting_timer: i32,     // kc+0xEC: ≤0 = targeting complete
    pub ih_cooldown: i32,         // ItemHolder+0x24: equip cooldown remaining
}

impl AiKartState {
    pub fn char_name(&self) -> &'static str {
        CHAR_NAMES.get(self.char_id as usize).unwrap_or(&"???")
    }

    /// Speed in km/h from PhysicsState internal speed
    pub fn speed_kmh(&self) -> f32 {
        self.internal_speed / 10000.0
    }
}

pub fn anim_state_name(state: i32) -> Option<&'static str> {
    match state {
        0 => None, // idle = normal
        1 => Some("装備中"),
        0x15 => Some("側面衝突"),
        0x16 => Some("前面衝突"),
        0x17 => Some("被弾(甲羅等)"),
        0x18..=0x1b => Some("スピン"),
        0x1c => Some("爆発"),
        0x1d => Some("小ダメージ"),
        0x21..=0x25 => Some("壁衝突"),
        0x26 => Some("トゲ甲羅"),
        0x27 => Some("雷"),
        0x28 => Some("スター被弾"),
        0x2a => Some("ゴール"),
        0x2b => Some("タイムアウト"),
        0x2d..=0x30 => Some("アイテム使用中"),
        -1 => Some("無効"),
        _ => Some("不明"),
    }
}

pub const KART_VARIANT_NAMES: &[&str] = &["ノーマル", "GOLD", "オリジナル", "パワーアップ"];

pub fn kart_variant_name(variant: u32) -> &'static str {
    KART_VARIANT_NAMES.get(variant as usize).unwrap_or(&"???")
}

/// Item ID to display string. Full table from 0x8041cba8 (160 entries).
pub fn item_name(id: i32) -> &'static str {
    match id {
        -1 => "なし",
        0 => "test",
        1 => "ハンマー(大)",
        2 => "ハンマー(中)",
        3 => "キノコハンマー",
        4 => "ピコピコハンマー",
        5 => "ピコピコハンマー(W)",
        6 => "10tハンマー",
        7 => "モンスターハンマー",
        8 => "電撃スティック",
        9 => "ハリセン",
        10 => "フライパン",
        11 => "ビヨヨンパンチ",
        12 => "ケーキ",
        13 => "ニンニク",
        14 => "パワーエサ",
        15 => "モンスターエサ",
        16 => "スターバナナ",
        17 => "メロン",
        18 => "炎のもと",
        19 => "おいしいバナナ",
        20 => "ニク",
        21 => "プリンセスドリンク",
        22 => "ミラクルキノコ",
        23 => "ダッシュキノコ",
        24 => "ゲンキキノコ",
        25 => "巨大化キノコ",
        26 => "吸収キノコ",
        27 => "ヌルヌルキノコ",
        28 => "透明キノコ",
        29 => "ヘビーキノコ",
        30 => "ワリオカー",
        31 => "シェルボディ",
        32 => "タマゴボディ",
        33 => "霊魂化",
        34 => "シールド",
        35 => "反射シールド",
        36 => "スター",
        37 => "SPコイン(大)",
        38 => "SPコイン(中)",
        39 => "フェアリー",
        40 => "モンスターシールド",
        41 => "ウソアラーム",
        42 => "コンガ",
        43 => "テレサ",
        44 => "クラクション",
        45 => "ハリセンボン",
        46 => "時限爆弾(小)",
        47 => "時限爆弾(大)",
        48 => "ハンドル菌",
        49 => "めまい菌",
        50 => "バナナ",
        51 => "ゴールドバナナ",
        52 => "ジャンボバナナ",
        55 => "タル",
        56 => "ゴロゴロタマゴ",
        57 => "タライ",
        58 => "ゴールドタライ",
        59 => "ドッスン",
        60 => "音痴スピーカー",
        61 => "ミドリこうら",
        62 => "黒こうら",
        63 => "ゴールドこうら",
        64 => "演楽",
        65 => "クッパのこうら",
        66 => "ハート",
        67 => "ブロック(大)",
        68 => "ブロック(中)",
        70 => "ガビョウ",
        71 => "毒キノコ",
        72 => "幻覚キノコ",
        74 => "ネバネバオイル",
        76 => "ボム兵",
        77 => "ワンワン",
        78 => "竜巻",
        80 => "パイ",
        81 => "オナラ",
        82 => "ハイビーム",
        84 => "マジックハンド",
        85 => "落書きペン",
        86 => "パックンフラワー",
        88 => "キノコの粉",
        89 => "ジャンプ封印ミサイル",
        90 => "アキカン",
        91 => "オンボロハンドル",
        92 => "デカチビタイヤ",
        93 => "カクカクタイヤ",
        94 => "笑い袋",
        95 => "ファイアボール(中)",
        96 => "ファイアボール(大)",
        97 => "ラリーX",
        105 => "かんしゃく玉",
        106 => "プチモンスター",
        107 => "ボスギャラガ",
        108 => "バナナ砲2",
        109 => "毒バナナ砲2",
        110 => "キノコ砲2",
        111 => "キラー",
        113 => "雷雲",
        114 => "雪雲",
        115 => "雨雲",
        116 => "アイテム封印ミサイル",
        117 => "鼻くそ爆弾",
        118 => "プーカ",
        119 => "謎のタマゴ",
        120 => "プリンセスバード",
        121 => "びっくりボール",
        122 => "ベロ",
        123 => "トリプルこうら",
        124 => "トリプルバナナ",
        125 => "トリプルキノコ",
        126 => "トリプルパイ",
        127 => "トリプルタライ",
        128 => "トリプル煙幕",
        129 => "トリプルテレサ",
        130 => "トリプルガビョウ",
        131 => "トリプルカクカクタイヤ",
        132 => "トリプルハリセンボン",
        133 => "トリプル落書きペン",
        134 => "トリプル竜巻",
        135 => "トリプル黒こうら",
        136 => "ヒゲミサイル",
        137 => "スパナ",
        138 => "ニセマリオコイン",
        139 => "栄養ドリンク",
        140 => "ウニボー",
        142 => "マメブロック",
        143 => "ドでかえんぴつ",
        144 => "バグバグチ",
        145 => "マメドリンク",
        146 => "チビッコ薬",
        147 => "カメン",
        148 => "おじゃまフレーム",
        149 => "ウサミミ",
        150 => "ネズミ爆弾",
        151 => "びっくり箱",
        152 => "グニャグミ",
        153 => "スーパースター",
        154 => "マグネット",
        155 => "バナナトレイン",
        157 => "ポンプ",
        159 => "no equip",
        _ => "???",
    }
}

#[derive(Default)]
pub struct GameState {
    pub connected: bool,
    pub game_mode: u32,
    pub cc_class: u32,
    pub course_id: u32,
    pub is_ura: bool,
    pub sub_mode: u32,
    pub race_started: bool,
    pub countdown_phase: u32,
    pub total_laps: u32,
    pub current_lap: u32,       // g_currentLap (1-indexed, player)
    pub current_lap_time: f32,  // g_currentLapTime (seconds, resets per lap)
    pub total_race_time: f32,   // g_totalRaceTime (seconds, completed laps sum)
    pub remaining_time: f32,    // g_remainingTime (seconds, countdown)
    pub lap_bonus_time: f32,    // g_lapBonusTime (seconds, fixed per race)
    pub player_ptr: u32,
    pub player: KartState,
    pub ai_karts: Vec<AiKartState>,
    /// Per-kart lap times from PathManager (km_ptr -> lap times)
    pub path_lap_data: Vec<PathLapData>,
    pub error: Option<String>,
}

#[derive(Default, Clone)]
pub struct PathLapData {
    pub km_ptr: u32,
    pub current_lap: i32, // -1=pre, 0=lap1...
    pub lap_times: Vec<f32>, // completed lap times
    pub total_time: f32,
    pub best_lap: f32,
}

impl GameState {
    pub fn course_name(&self) -> String {
        let name = COURSE_NAMES
            .get(self.course_id as usize)
            .copied()
            .unwrap_or("???");
        if self.course_id == 0 {
            format!("courseId={}", self.course_id)
        } else if self.course_id <= 8 {
            let side = if self.is_ura { "ウラ" } else { "オモテ" };
            format!("{} {}", side, name)
        } else if self.course_id <= 16 {
            name.to_string() // minigame: no side
        } else {
            format!("courseId={}", self.course_id)
        }
    }

    pub fn cc_label(&self) -> &'static str {
        CC_LABELS.get(self.cc_class as usize).unwrap_or(&"???")
    }

    pub fn game_mode_name(&self) -> String {
        GAME_MODE_NAMES
            .get(self.game_mode as usize)
            .map(|s| s.to_string())
            .unwrap_or_else(|| format!("mode={}", self.game_mode))
    }

    pub fn is_in_race(&self) -> bool {
        // Player CarObject must exist for a race to be active
        self.player.char_id < 15 && self.race_started
    }
}

// Convenience wrappers that strip the PPC high bit and pass no pointer offsets
pub fn read_u32(dolphin: &Dolphin, addr: u32) -> Result<u32, String> {
    dolphin
        .read_u32((addr & 0x01FFFFFF) as usize, None)
        .map_err(|e| e.to_string())
}

pub fn read_f32(dolphin: &Dolphin, addr: u32) -> Result<f32, String> {
    dolphin
        .read_f32((addr & 0x01FFFFFF) as usize, None)
        .map_err(|e| e.to_string())
}

pub fn read_u8(dolphin: &Dolphin, addr: u32) -> Result<u8, String> {
    dolphin
        .read_u8((addr & 0x01FFFFFF) as usize, None)
        .map_err(|e| e.to_string())
}

pub fn write_u32(dolphin: &Dolphin, addr: u32, val: u32) -> Result<(), String> {
    dolphin
        .write_u32(val, (addr & 0x01FFFFFF) as usize, None)
        .map_err(|e| e.to_string())
}

pub fn write_f32(dolphin: &Dolphin, addr: u32, val: f32) -> Result<(), String> {
    dolphin
        .write_f32(val, (addr & 0x01FFFFFF) as usize, None)
        .map_err(|e| e.to_string())
}

// --- FreezeMap: generic per-address value pinning ---

#[derive(Clone, Debug)]
pub enum FrozenValue {
    U32(u32),
    F32(f32),
}

/// Maps PPC address → frozen value. Every poll cycle, all entries are written back.
#[derive(Default)]
pub struct FreezeMap {
    entries: HashMap<u32, FrozenValue>,
}

impl FreezeMap {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn set_u32(&mut self, addr: u32, val: u32) {
        self.entries.insert(addr, FrozenValue::U32(val));
    }

    pub fn set_f32(&mut self, addr: u32, val: f32) {
        self.entries.insert(addr, FrozenValue::F32(val));
    }

    pub fn remove(&mut self, addr: u32) {
        self.entries.remove(&addr);
    }

    pub fn contains(&self, addr: u32) -> bool {
        self.entries.contains_key(&addr)
    }

    pub fn get(&self, addr: u32) -> Option<&FrozenValue> {
        self.entries.get(&addr)
    }

    pub fn is_empty(&self) -> bool {
        self.entries.is_empty()
    }

    pub fn len(&self) -> usize {
        self.entries.len()
    }

    /// Write all frozen values to Dolphin memory.
    pub fn apply_all(&self, dolphin: &Dolphin) {
        for (&addr, val) in &self.entries {
            match val {
                FrozenValue::U32(v) => { let _ = write_u32(dolphin, addr, *v); }
                FrozenValue::F32(v) => { let _ = write_f32(dolphin, addr, *v); }
            }
        }
    }

    /// Bulk insert menu timer freeze entries (global timer + UI rate constants).
    pub fn add_menu_timer_freeze(&mut self) {
        self.set_u32(addr::G_GLOBAL_TIMER, 0x7FFFFFFF);
        for &a in addr::DECREMENT_RATE_ADDRS {
            self.set_f32(a, 0.0);
        }
    }

    /// Bulk remove menu timer freeze entries and restore defaults.
    pub fn remove_menu_timer_freeze(&mut self, dolphin: &Dolphin) {
        self.remove(addr::G_GLOBAL_TIMER);
        let rate = 1.0_f32 / 60.0;
        for &a in addr::DECREMENT_RATE_ADDRS {
            self.remove(a);
            let _ = write_f32(dolphin, a, rate);
        }
    }

    /// Bulk insert race timer freeze entries.
    pub fn add_race_timer_freeze(&mut self) {
        for &a in addr::RACE_TIMER_RATE_ADDRS {
            self.set_f32(a, 0.0);
        }
    }

    /// Bulk remove race timer freeze entries and restore defaults.
    pub fn remove_race_timer_freeze(&mut self, dolphin: &Dolphin) {
        let rate = 1.0_f32 / 60.0;
        for &a in addr::RACE_TIMER_RATE_ADDRS {
            self.remove(a);
            let _ = write_f32(dolphin, a, rate);
        }
    }
}


pub fn try_read_state(dolphin: &Dolphin) -> GameState {
    let mut state = GameState {
        connected: true,
        ..Default::default()
    };

    match (|| -> Result<(), String> {
        state.game_mode = read_u32(dolphin, addr::G_GAME_MODE)?;
        state.cc_class = read_u32(dolphin, addr::G_CC_CLASS)?;
        state.course_id = read_u32(dolphin, addr::G_COURSE_ID)?;
        state.sub_mode = read_u32(dolphin, addr::G_SUB_MODE)?;
        // Use both g_isUraCourse and subMode for redundancy
        let is_ura_flag = read_u32(dolphin, addr::G_IS_URA_COURSE)? != 0;
        state.is_ura = is_ura_flag || state.sub_mode >= 4;
        state.race_started = read_u8(dolphin, addr::G_RACE_STARTED)? != 0;
        state.countdown_phase = read_u32(dolphin, addr::G_COUNTDOWN_PHASE)?;
        state.total_laps = read_u32(dolphin, addr::G_TOTAL_LAPS)?;
        state.current_lap = read_u32(dolphin, addr::G_CURRENT_LAP)?;
        state.current_lap_time = read_f32(dolphin, addr::G_CURRENT_LAP_TIME)?;
        state.total_race_time = read_f32(dolphin, addr::G_TOTAL_RACE_TIME)?;
        state.remaining_time = read_f32(dolphin, addr::G_REMAINING_TIME)?;
        state.lap_bonus_time = read_f32(dolphin, addr::G_LAP_BONUS_TIME)?;

        // Read player CarObject
        let player_ptr = read_u32(dolphin, addr::G_PLAYER_CAR_OBJECT)?;
        state.player_ptr = player_ptr;
        if player_ptr != 0 {
            state.player = read_kart_state(dolphin, player_ptr)?;
            // KM+0x23C is not updated for the player; use g_racePosition instead
            state.player.race_position = read_u32(dolphin, addr::G_RACE_POSITION)?;
        }

        // Read AI kart controllers
        for i in 0u32..5 {
            let kc_ptr = read_u32(dolphin, addr::AI_KART_CONTROLLERS + i * 4)?;
            if kc_ptr == 0 {
                continue;
            }
            let ro_ptr = read_u32(dolphin, kc_ptr + addr::KC_RENDER_OBJ)?;
            let ps_ptr = read_u32(dolphin, kc_ptr + addr::KC_PHYS_STATE)?;
            let ai_char_id = if ro_ptr != 0 {
                read_u32(dolphin, ro_ptr + addr::RO_CHAR_ID).unwrap_or(0xFF)
            } else {
                0xFF
            };
            let (velocity, internal_speed, heading) = if ps_ptr > 0x80000000 && ps_ptr < 0x81800000 {
                (
                    [
                        read_f32(dolphin, ps_ptr + addr::PS_VELOCITY_X).unwrap_or(0.0),
                        read_f32(dolphin, ps_ptr + addr::PS_VELOCITY_Y).unwrap_or(0.0),
                        read_f32(dolphin, ps_ptr + addr::PS_VELOCITY_Z).unwrap_or(0.0),
                    ],
                    read_f32(dolphin, ps_ptr + addr::PS_SPEED).unwrap_or(0.0),
                    read_f32(dolphin, ps_ptr + addr::PS_HEADING).unwrap_or(0.0),
                )
            } else {
                ([0.0; 3], 0.0, 0.0)
            };
            let ai = AiKartState {
                kc_ptr,
                render_obj_ptr: ro_ptr,
                phys_state_ptr: ps_ptr,
                slot: i,
                char_id: ai_char_id,
                pos: [
                    read_f32(dolphin, kc_ptr + addr::KC_POS_X)?,
                    read_f32(dolphin, kc_ptr + addr::KC_POS_Y)?,
                    read_f32(dolphin, kc_ptr + addr::KC_POS_Z)?,
                ],
                velocity,
                internal_speed,
                heading,
                target_speed_min: read_f32(dolphin, kc_ptr + addr::KC_TARGET_SPEED_MIN)?,
                target_speed_max: read_f32(dolphin, kc_ptr + addr::KC_TARGET_SPEED_MAX)?,
                race_position: if ro_ptr != 0 {
                    read_u32(dolphin, ro_ptr + addr::RO_RACE_POSITION)?
                } else {
                    0
                },
                remaining_laps: if ro_ptr != 0 {
                    read_u32(dolphin, ro_ptr + addr::RO_CURRENT_LAP)?
                } else {
                    0
                },
                item_type: {
                    let ih = read_u32(dolphin, kc_ptr + addr::KC_ITEM_HOLDER).unwrap_or(0);
                    if ih != 0 && read_u32(dolphin, ih + addr::IH_HAS_ITEM).unwrap_or(0) != 0 {
                        read_u32(dolphin, ih + addr::IH_ITEM_TYPE).unwrap_or(0) as i32
                    } else { -1 }
                },
                has_item: {
                    let ih = read_u32(dolphin, kc_ptr + addr::KC_ITEM_HOLDER).unwrap_or(0);
                    ih != 0 && read_u32(dolphin, ih + addr::IH_HAS_ITEM).unwrap_or(0) != 0
                },
                current_item_id: if ro_ptr != 0 {
                    let equipped = dolphin.read_u8(((ro_ptr + addr::RO_ITEM_EQUIPPED) & 0x01FFFFFF) as usize, None).unwrap_or(0);
                    if equipped == 1 {
                        let id = read_u32(dolphin, ro_ptr + addr::RO_ITEM_ID).unwrap_or(0xFFFFFFFF);
                        let valid = (id >= 1 && id <= 0x114) || (id >= 0x116 && id <= 0x136);
                        if valid { id as i32 } else { -1 }
                    } else { -1 }
                } else { -1 },
                use_interval_timer: read_u32(dolphin, kc_ptr + addr::KC_USE_INTERVAL_TIMER).unwrap_or(0) as i32,
                targeting_timer: read_u32(dolphin, kc_ptr + addr::KC_TARGETING_TIMER).unwrap_or(0) as i32,
                ih_cooldown: {
                    let ih = read_u32(dolphin, kc_ptr + addr::KC_ITEM_HOLDER).unwrap_or(0);
                    if ih != 0 { read_u32(dolphin, ih + addr::IH_COOLDOWN).unwrap_or(0) as i32 } else { 0 }
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

    // RenderObj holds item master data
    let ro_ptr = read_u32(dolphin, car_obj + addr::CAR_RENDER_OBJ).unwrap_or(0);
    let valid_ro = ro_ptr > 0x80000000 && ro_ptr < 0x81800000;

    Ok(KartState {
        car_obj_ptr: car_obj,
        km_ptr,
        render_obj_ptr: if valid_ro { ro_ptr } else { 0 },
        slot: read_u32(dolphin, car_obj + addr::CAR_KART_SLOT)?,
        char_id: read_u32(dolphin, car_obj + addr::CAR_CHAR_ID)?,
        kart_variant: read_u32(dolphin, car_obj + addr::CAR_KART_VARIANT)?,
        is_player: read_u8(dolphin, car_obj + addr::CAR_IS_PLAYER)? != 0,
        race_position: read_u32(dolphin, km_ptr + addr::KM_RACE_POSITION)?,
        remaining_laps: read_u32(dolphin, km_ptr + addr::KM_CURRENT_LAP)?,
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
        anim_state: read_u32(dolphin, km_ptr + addr::KM_ANIM_STATE).unwrap_or(0) as i32,
        // Read item from RenderObj (master data), not ISE (downstream)
        current_item_id: if valid_ro
            && read_u8(dolphin, ro_ptr + addr::RO_ITEM_EQUIPPED).unwrap_or(0) == 1
        {
            let id = read_u32(dolphin, ro_ptr + addr::RO_ITEM_ID).unwrap_or(0xFFFFFFFF);
            // Valid item ranges from game's HUD code (FUN_8023d458)
            let valid = (id >= 1 && id <= 0x114) || (id >= 0x116 && id <= 0x136);
            if valid { id as i32 } else { -1 }
        } else {
            -1
        },
        item_slot_entry_ptr: 0, // deprecated, kept for compat
        start_dash_ready: read_u8(dolphin, car_obj + addr::CAR_START_DASH_READY)? != 0,
        start_dash_frame: read_u32(dolphin, car_obj + addr::CAR_START_DASH_FRAME)?,
        warp_context_ptr: read_u32(dolphin, car_obj + addr::CAR_WARP_CONTEXT)?,
    })
}

/// Read PathManager lap data on demand (not every frame).
pub fn read_path_lap_data(dolphin: &Dolphin) -> Vec<PathLapData> {
    let mut result = Vec::new();
    let pm_ptr = match read_u32(dolphin, addr::G_PATH_MANAGER_RACE) {
        Ok(p) if p > 0x80000000 && p < 0x81800000 => p,
        _ => return result,
    };
    for i in 0u32..8 {
        let entry = pm_ptr + i * addr::PE_STRIDE;
        let km = read_u32(dolphin, entry + addr::PE_KART_MOVEMENT).unwrap_or(0);
        if km == 0 || km < 0x80000000 { continue; }
        let current_lap = read_u32(dolphin, entry + addr::PE_CURRENT_LAP).unwrap_or(0) as i32;
        let total_time = read_f32(dolphin, entry + addr::PE_TOTAL_TIME).unwrap_or(0.0);
        let best_lap = read_f32(dolphin, entry + addr::PE_BEST_LAP).unwrap_or(0.0);
        let completed = (current_lap.max(0) as usize).min(10);
        let mut lap_times = Vec::with_capacity(completed);
        for lap in 0..completed {
            let t = read_f32(dolphin, entry + addr::PE_LAP_TIMES + lap as u32 * 4)
                .unwrap_or(0.0);
            lap_times.push(t);
        }
        result.push(PathLapData { km_ptr: km, current_lap, lap_times, total_time, best_lap });
    }
    result
}
