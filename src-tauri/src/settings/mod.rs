// Mirror of the descriptor-driven frontend schema in `src/settingsSchema.ts`.
// When adding or changing a backend-facing setting, keep the section and
// default in sync with the TypeScript field definitions.

#[derive(Clone, serde::Deserialize, serde::Serialize, Debug, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct ClickPoint {
    #[serde(default)]
    pub id: String,
    pub x: i32,
    pub y: i32,
    #[serde(default = "default_click_point_clicks")]
    pub clicks: u32,
    #[serde(default)]
    pub radius: u32,
}

fn default_click_point_clicks() -> u32 {
    1
}

fn default_keyboard_key_case() -> String {
    "lower".to_string()
}

fn default_true() -> bool {
    true
}

fn default_indicator_corner() -> String {
    "top-right".to_string()
}

fn default_indicator_color_mode() -> String {
    "theme".to_string()
}

fn default_indicator_custom_color() -> String {
    "#ffffff".to_string()
}

fn default_indicator_size() -> u32 {
    32
}

fn default_indicator_offset() -> u32 {
    24
}

fn default_indicator_flash_dim() -> u32 {
    25
}

fn default_indicator_countdown_mode() -> String {
    "words".to_string()
}

fn default_indicator_background_mode() -> String {
    "card".to_string()
}

fn default_indicator_background_strength() -> u32 {
    100
}

use crate::engine::ProcessListEntry;

fn default_pixel_trigger_mode() -> String {
    "line".to_string()
}

fn default_pixel_trigger_hold_ms() -> u32 {
    35
}

fn default_pixel_trigger_hold_max_ms() -> u32 {
    60
}

fn default_pixel_trigger_cooldown_ms() -> u32 {
    1_000
}

fn default_white() -> String {
    "#ffffff".to_string()
}

fn default_green() -> String {
    "#06c105".to_string()
}

fn default_anchor_tolerance() -> u32 {
    30
}

fn default_check_tolerance() -> u32 {
    40
}

/// Line mode of the pixel trigger: anchor colour on line A, check colour on
/// line B in the same column. Coordinates are virtual-screen pixels.
#[derive(Clone, serde::Deserialize, serde::Serialize, Debug, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct PixelTriggerLineSettings {
    #[serde(default)]
    pub x1: i32,
    #[serde(default)]
    pub x2: i32,
    #[serde(default)]
    pub anchor_y: i32,
    #[serde(default)]
    pub check_y: i32,
    #[serde(default = "default_white")]
    pub anchor_color: String,
    #[serde(default = "default_anchor_tolerance")]
    pub anchor_tolerance: u32,
    #[serde(default = "default_green")]
    pub check_color: String,
    #[serde(default = "default_check_tolerance")]
    pub check_tolerance: u32,
}

impl Default for PixelTriggerLineSettings {
    fn default() -> Self {
        Self {
            x1: 0,
            x2: 0,
            anchor_y: 0,
            check_y: 0,
            anchor_color: default_white(),
            anchor_tolerance: default_anchor_tolerance(),
            check_color: default_green(),
            check_tolerance: default_check_tolerance(),
        }
    }
}

fn default_recast_after_matches() -> u32 {
    3
}

fn default_recast_delay_ms() -> u32 {
    1_500
}

fn default_recast_button() -> String {
    "Right".to_string()
}

fn default_recast_step_button() -> String {
    "Left".to_string()
}

/// One click of the recast sequence.
#[derive(Clone, serde::Deserialize, serde::Serialize, Debug, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct PixelTriggerRecastStepSettings {
    #[serde(default)]
    pub id: String,
    #[serde(default)]
    pub x: i32,
    #[serde(default)]
    pub y: i32,
    #[serde(default = "default_recast_step_button")]
    pub button: String,
    #[serde(default)]
    pub delay_ms: u32,
    #[serde(default)]
    pub delay_max_ms: u32,
}

/// Recast sequence of the pixel trigger. `x`/`y`/`button`/`delay*` are the
/// pre-sequence single-click fields, only read when `steps` is empty.
#[derive(Clone, serde::Deserialize, serde::Serialize, Debug, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct PixelTriggerRecastSettings {
    #[serde(default)]
    pub enabled: bool,
    #[serde(default = "default_recast_after_matches")]
    pub after_matches: u32,
    #[serde(default = "default_recast_delay_ms")]
    pub delay_ms: u32,
    /// Upper bound when timings are randomized.
    #[serde(default)]
    pub delay_max_ms: u32,
    #[serde(default)]
    pub x: i32,
    #[serde(default)]
    pub y: i32,
    #[serde(default = "default_recast_button")]
    pub button: String,
    #[serde(default)]
    pub idle_timeout_ms: u32,
    #[serde(default)]
    pub steps: Vec<PixelTriggerRecastStepSettings>,
}

impl Default for PixelTriggerRecastSettings {
    fn default() -> Self {
        Self {
            enabled: false,
            after_matches: default_recast_after_matches(),
            delay_ms: default_recast_delay_ms(),
            delay_max_ms: 0,
            x: 0,
            y: 0,
            button: default_recast_button(),
            idle_timeout_ms: 0,
            steps: Vec::new(),
        }
    }
}

#[derive(Clone, serde::Deserialize, serde::Serialize, Debug, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct PixelTriggerPointSettings {
    #[serde(default)]
    pub id: String,
    pub x: i32,
    pub y: i32,
    #[serde(default = "default_white")]
    pub color: String,
    #[serde(default = "default_anchor_tolerance")]
    pub tolerance: u32,
    #[serde(default)]
    pub negate: bool,
}

#[derive(Clone, serde::Deserialize, serde::Serialize, Debug, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct PixelTriggerActionSettings {
    #[serde(default)]
    pub id: String,
    pub x: i32,
    pub y: i32,
    #[serde(default)]
    pub delay_ms: u32,
    /// Upper bound when timings are randomized.
    #[serde(default)]
    pub delay_max_ms: u32,
}

#[derive(Clone, PartialEq, Eq, serde::Deserialize, serde::Serialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct StopZoneConfig {
    #[serde(default)]
    pub id: String,
    pub x: i32,
    pub y: i32,
    pub width: i32,
    pub height: i32,
    #[serde(default = "default_stop_zone_action")]
    pub action: String,
}

fn default_stop_zone_action() -> String {
    "stop".to_string()
}

#[derive(Clone, serde::Deserialize, serde::Serialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct ClickerSettings {
    // Meta
    pub version: u32,

    // Preset snapshot fields
    pub click_speed: f64,
    pub click_interval: String,
    pub input_type: String,
    pub keyboard_key: String,
    #[serde(default = "default_keyboard_key_case")]
    pub keyboard_key_case: String,
    pub mouse_button: String,
    pub mode: String,

    pub duty_cycle_mode: String,
    pub saved_click_speed: f64,
    pub saved_click_interval: String,

    pub duty_cycle_enabled: bool,
    pub duty_cycle: f64,

    pub speed_randomization_enabled: bool,
    pub speed_randomization: f64,

    pub double_click_enabled: bool,

    pub click_limit_enabled: bool,
    pub click_limit: i32,

    pub time_limit_enabled: bool,
    pub time_limit: f64,
    pub time_limit_unit: String,

    pub corner_stop_enabled: bool,
    #[serde(rename = "cornerStopTL")]
    pub corner_stop_tl: i32,
    #[serde(rename = "cornerStopTR")]
    pub corner_stop_tr: i32,
    #[serde(rename = "cornerStopBL")]
    pub corner_stop_bl: i32,
    #[serde(rename = "cornerStopBR")]
    pub corner_stop_br: i32,

    pub edge_stop_enabled: bool,
    pub edge_stop_top: i32,
    pub edge_stop_right: i32,
    pub edge_stop_bottom: i32,
    pub edge_stop_left: i32,

    pub click_points_enabled: bool,
    #[serde(rename = "stopZonesEnabled")]
    pub stop_zones_enabled: bool,
    #[serde(default)]
    pub stop_when_complete: bool,
    pub click_points: Vec<ClickPoint>,

    pub process_list_enabled: bool,
    pub process_list_mode: String,
    pub process_list_entries: Vec<ProcessListEntry>,

    #[serde(default)]
    pub pixel_trigger_enabled: bool,
    #[serde(default = "default_pixel_trigger_mode")]
    pub pixel_trigger_mode: String,
    #[serde(default)]
    pub pixel_trigger_line: PixelTriggerLineSettings,
    #[serde(default)]
    pub pixel_trigger_points: Vec<PixelTriggerPointSettings>,
    #[serde(default)]
    pub pixel_trigger_actions: Vec<PixelTriggerActionSettings>,
    #[serde(default = "default_pixel_trigger_cooldown_ms")]
    pub pixel_trigger_cooldown_ms: u32,
    #[serde(default)]
    pub pixel_trigger_recast: PixelTriggerRecastSettings,
    /// Randomize waits and click hold within their min..max ranges.
    #[serde(default)]
    pub pixel_trigger_randomize: bool,
    #[serde(default)]
    pub pixel_trigger_cooldown_max_ms: u32,
    #[serde(default = "default_pixel_trigger_hold_ms")]
    pub pixel_trigger_hold_ms: u32,
    #[serde(default = "default_pixel_trigger_hold_max_ms")]
    pub pixel_trigger_hold_max_ms: u32,

    // settings-only fields
    #[serde(default = "default_true")]
    pub task_switcher_stop_enabled: bool,
    /// Delay in milliseconds between activating the clicker and the first click. 0 = off.
    #[serde(default)]
    pub start_delay_ms: u32,
    pub hotkey: String,
    pub rate_input_mode: String,
    pub duration_hours: u32,
    pub duration_minutes: u32,
    pub duration_seconds: u32,
    pub duration_milliseconds: u32,

    #[serde(default)]
    pub stop_zones: Vec<StopZoneConfig>,

    pub disable_screenshots: bool,
    pub advanced_settings_enabled: bool,
    pub last_panel: String,
    pub show_stop_reason: bool,
    pub show_stop_overlay: bool,
    pub strict_hotkey_modifiers: bool,
    #[serde(default)]
    pub minimize_to_tray: bool,

    #[serde(default = "default_true")]
    pub indicator_enabled: bool,
    #[serde(default = "default_indicator_corner")]
    pub indicator_corner: String,
    #[serde(default = "default_indicator_color_mode")]
    pub indicator_color_mode: String,
    #[serde(default = "default_indicator_custom_color")]
    pub indicator_custom_color: String,
    #[serde(default = "default_indicator_size")]
    pub indicator_size: u32,
    #[serde(default = "default_indicator_offset")]
    pub indicator_offset_x: u32,
    #[serde(default = "default_indicator_offset")]
    pub indicator_offset_y: u32,
    #[serde(default)]
    pub indicator_flash_enabled: bool,
    #[serde(default = "default_indicator_flash_dim")]
    pub indicator_flash_dim: u32,
    #[serde(default)]
    pub indicator_countdown_enabled: bool,
    #[serde(default = "default_indicator_countdown_mode")]
    pub indicator_countdown_mode: String,
    #[serde(default = "default_indicator_background_mode")]
    pub indicator_background_mode: String,
    #[serde(default = "default_indicator_background_strength")]
    pub indicator_background_strength: u32,
}

// Frontend-only settings intentionally omitted from Rust:
// language, minimizeToTray, theme, alwaysOnTop, accentColor, presets,
// activePresetId.

impl Default for ClickerSettings {
    fn default() -> Self {
        Self {
            // Meta
            version: 11,

            // Preset snapshot fields
            click_speed: 25.0,
            click_interval: "s".to_string(),
            input_type: "mouse".to_string(),
            keyboard_key: String::new(),
            keyboard_key_case: default_keyboard_key_case(),
            mouse_button: "Left".to_string(),
            mode: "Toggle".to_string(),

            duty_cycle_mode: "Click".to_string(),
            saved_click_speed: 25.0,
            saved_click_interval: "s".to_string(),

            duty_cycle_enabled: true,
            duty_cycle: 45.0,

            speed_randomization_enabled: true,
            speed_randomization: 35.0,

            double_click_enabled: false,

            click_limit_enabled: false,
            click_limit: 1000,

            time_limit_enabled: false,
            time_limit: 60.0,
            time_limit_unit: "s".to_string(),

            corner_stop_enabled: true,
            corner_stop_tl: 50,
            corner_stop_tr: 50,
            corner_stop_bl: 50,
            corner_stop_br: 50,

            edge_stop_enabled: true,
            edge_stop_top: 40,
            edge_stop_right: 40,
            edge_stop_bottom: 40,
            edge_stop_left: 40,

            click_points_enabled: false,
            stop_zones_enabled: false,
            stop_when_complete: false,
            click_points: Vec::new(),

            process_list_enabled: false,
            process_list_mode: "whitelist".to_string(),
            process_list_entries: Vec::new(),

            pixel_trigger_enabled: false,
            pixel_trigger_mode: default_pixel_trigger_mode(),
            pixel_trigger_line: PixelTriggerLineSettings::default(),
            pixel_trigger_points: Vec::new(),
            pixel_trigger_actions: Vec::new(),
            pixel_trigger_cooldown_ms: default_pixel_trigger_cooldown_ms(),
            pixel_trigger_recast: PixelTriggerRecastSettings::default(),
            pixel_trigger_randomize: false,
            pixel_trigger_cooldown_max_ms: 0,
            pixel_trigger_hold_ms: default_pixel_trigger_hold_ms(),
            pixel_trigger_hold_max_ms: default_pixel_trigger_hold_max_ms(),

            // settings-only defaults
            task_switcher_stop_enabled: true,
            start_delay_ms: 0,
            hotkey: "ctrl+y".to_string(),
            rate_input_mode: "rate".to_string(),
            duration_hours: 0,
            duration_minutes: 0,
            duration_seconds: 0,
            duration_milliseconds: 40,

            stop_zones: Vec::new(),

            disable_screenshots: false,
            advanced_settings_enabled: true,
            last_panel: "simple".to_string(),
            show_stop_reason: true,
            show_stop_overlay: true,
            strict_hotkey_modifiers: false,
            minimize_to_tray: false,

            indicator_enabled: true,
            indicator_corner: default_indicator_corner(),
            indicator_color_mode: default_indicator_color_mode(),
            indicator_custom_color: default_indicator_custom_color(),
            indicator_size: default_indicator_size(),
            indicator_offset_x: default_indicator_offset(),
            indicator_offset_y: default_indicator_offset(),
            indicator_flash_enabled: false,
            indicator_flash_dim: default_indicator_flash_dim(),
            indicator_countdown_enabled: false,
            indicator_countdown_mode: default_indicator_countdown_mode(),
            indicator_background_mode: default_indicator_background_mode(),
            indicator_background_strength: default_indicator_background_strength(),
        }
    }
}

/// Inclusive bounds mirroring the `limit` blocks in `src/settingsSchema.ts`. The
/// frontend clamps to these before sending, so a value outside them arrived by
/// walking around the UI.
mod limits {
    pub const CLICK_SPEED: (f64, f64) = (1.0, 1_000.0);
    pub const SAVED_CLICK_SPEED: (f64, f64) = (1.0, f64::MAX);
    pub const DUTY_CYCLE: (f64, f64) = (0.0, 100.0);
    pub const SPEED_RANDOMIZATION: (f64, f64) = (0.0, 200.0);
    pub const TIME_LIMIT: (f64, f64) = (1.0, f64::MAX);
    pub const CLICK_LIMIT: (i32, i32) = (1, 100_000_000);
    pub const STOP_BOUNDARY: (i32, i32) = (0, 10_000);
    pub const CLICK_POINT_CLICKS: (u32, u32) = (1, 999_999);
    pub const CLICK_POINT_RADIUS: (u32, u32) = (0, 9_999);
    pub const INDICATOR_SIZE: (u32, u32) = (16, 512);
    pub const INDICATOR_OFFSET: (u32, u32) = (0, 4_000);
    pub const INDICATOR_FLASH_DIM: (u32, u32) = (0, 100);
    pub const INDICATOR_CORNERS: [&str; 4] =
        ["top-right", "top-left", "bottom-right", "bottom-left"];
    pub const INDICATOR_COLOR_MODES: [&str; 4] = ["theme", "black", "white", "custom"];
    pub const INDICATOR_COUNTDOWN_MODES: [&str; 3] = ["plain", "words", "short"];
    pub const INDICATOR_BACKGROUND_MODES: [&str; 3] = ["off", "card", "shadow"];
    pub const STOP_ZONE_OFFSET: (i32, i32) = (-99_999, 99_999);
    pub const STOP_ZONE_SIZE: (i32, i32) = (1, 99_999);
    pub const STOP_ZONE_ACTIONS: [&str; 3] = ["stop", "pause", "start"];
    pub const STOP_ZONE_MAX_COUNT: usize = 64;
    pub const START_DELAY_MS: (u32, u32) = (0, 60_000);
    pub const PIXEL_TOLERANCE: (u32, u32) = (0, 255);
    pub const PIXEL_DELAY_MS: (u32, u32) = (0, 600_000);
    pub const PIXEL_TRIGGER_MODES: [&str; 2] = ["line", "points"];
    pub const PIXEL_TRIGGER_MAX_ITEMS: usize = 16;
    pub const RECAST_AFTER_MATCHES: (u32, u32) = (0, 9_999);
    pub const PIXEL_HOLD_MS: (u32, u32) = (0, 10_000);
    pub const RECAST_MAX_STEPS: usize = 8;
    pub const MOUSE_BUTTONS: [&str; 3] = ["Left", "Right", "Middle"];
}

/// Non-finite input falls back to `min`: `f64::clamp` propagates NaN, and an
/// infinite interval is exactly what turns into a panic further down.
fn clamp_f64(field: &str, value: &mut f64, (min, max): (f64, f64)) {
    let clamped = if value.is_finite() {
        value.clamp(min, max)
    } else {
        min
    };
    if clamped != *value {
        log::warn!("[Settings] {field} = {value} is outside {min}..={max}, using {clamped}");
        *value = clamped;
    }
}

fn clamp_i32(field: &str, value: &mut i32, (min, max): (i32, i32)) {
    let clamped = (*value).clamp(min, max);
    if clamped != *value {
        log::warn!("[Settings] {field} = {value} is outside {min}..={max}, using {clamped}");
        *value = clamped;
    }
}

fn clamp_u32(field: &str, value: &mut u32, (min, max): (u32, u32)) {
    let clamped = (*value).clamp(min, max);
    if clamped != *value {
        log::warn!("[Settings] {field} = {value} is outside {min}..={max}, using {clamped}");
        *value = clamped;
    }
}

fn sanitize_choice(field: &str, value: &mut String, allowed: &[&str], fallback: &str) {
    if !allowed.contains(&value.as_str()) {
        log::warn!("[Settings] {field} = {value:?} is not one of {allowed:?}, using {fallback:?}");
        *value = fallback.to_string();
    }
}

fn is_hex_color(value: &str) -> bool {
    let bytes = value.as_bytes();
    bytes.len() == 7 && bytes[0] == b'#' && bytes[1..].iter().all(u8::is_ascii_hexdigit)
}

impl ClickerSettings {
    fn sanitize_pixel_trigger(&mut self) {
        sanitize_choice(
            "pixelTriggerMode",
            &mut self.pixel_trigger_mode,
            &limits::PIXEL_TRIGGER_MODES,
            "line",
        );
        clamp_u32(
            "pixelTriggerCooldownMs",
            &mut self.pixel_trigger_cooldown_ms,
            limits::PIXEL_DELAY_MS,
        );

        let line = &mut self.pixel_trigger_line;
        clamp_u32(
            "pixelTriggerLine.anchorTolerance",
            &mut line.anchor_tolerance,
            limits::PIXEL_TOLERANCE,
        );
        clamp_u32(
            "pixelTriggerLine.checkTolerance",
            &mut line.check_tolerance,
            limits::PIXEL_TOLERANCE,
        );
        if !is_hex_color(&line.anchor_color) {
            line.anchor_color = default_white();
        }
        if !is_hex_color(&line.check_color) {
            line.check_color = default_green();
        }

        self.pixel_trigger_points
            .truncate(limits::PIXEL_TRIGGER_MAX_ITEMS);
        for point in &mut self.pixel_trigger_points {
            clamp_u32(
                "pixelTriggerPoints[].tolerance",
                &mut point.tolerance,
                limits::PIXEL_TOLERANCE,
            );
            if !is_hex_color(&point.color) {
                point.color = default_white();
            }
        }

        let recast = &mut self.pixel_trigger_recast;
        clamp_u32(
            "pixelTriggerRecast.afterMatches",
            &mut recast.after_matches,
            limits::RECAST_AFTER_MATCHES,
        );
        clamp_u32(
            "pixelTriggerRecast.delayMs",
            &mut recast.delay_ms,
            limits::PIXEL_DELAY_MS,
        );
        clamp_u32(
            "pixelTriggerRecast.idleTimeoutMs",
            &mut recast.idle_timeout_ms,
            limits::PIXEL_DELAY_MS,
        );
        clamp_u32(
            "pixelTriggerRecast.delayMaxMs",
            &mut recast.delay_max_ms,
            limits::PIXEL_DELAY_MS,
        );
        sanitize_choice(
            "pixelTriggerRecast.button",
            &mut recast.button,
            &limits::MOUSE_BUTTONS,
            "Right",
        );
        recast.steps.truncate(limits::RECAST_MAX_STEPS);
        for step in &mut recast.steps {
            clamp_u32(
                "pixelTriggerRecast.steps[].delayMs",
                &mut step.delay_ms,
                limits::PIXEL_DELAY_MS,
            );
            clamp_u32(
                "pixelTriggerRecast.steps[].delayMaxMs",
                &mut step.delay_max_ms,
                limits::PIXEL_DELAY_MS,
            );
            sanitize_choice(
                "pixelTriggerRecast.steps[].button",
                &mut step.button,
                &limits::MOUSE_BUTTONS,
                "Left",
            );
        }

        self.pixel_trigger_actions
            .truncate(limits::PIXEL_TRIGGER_MAX_ITEMS);
        for action in &mut self.pixel_trigger_actions {
            clamp_u32(
                "pixelTriggerActions[].delayMs",
                &mut action.delay_ms,
                limits::PIXEL_DELAY_MS,
            );
            clamp_u32(
                "pixelTriggerActions[].delayMaxMs",
                &mut action.delay_max_ms,
                limits::PIXEL_DELAY_MS,
            );
        }
        clamp_u32(
            "pixelTriggerCooldownMaxMs",
            &mut self.pixel_trigger_cooldown_max_ms,
            limits::PIXEL_DELAY_MS,
        );
        clamp_u32(
            "pixelTriggerHoldMs",
            &mut self.pixel_trigger_hold_ms,
            limits::PIXEL_HOLD_MS,
        );
        clamp_u32(
            "pixelTriggerHoldMaxMs",
            &mut self.pixel_trigger_hold_max_ms,
            limits::PIXEL_HOLD_MS,
        );
    }

    /// Pin every numeric field to the range the frontend already enforces.
    ///
    /// IPC arguments arrive as JSON, where serde checks the type and nothing
    /// else. A caller that skips the UI can send `clickSpeed: 5e-20`, which
    /// `interval_secs_from_settings` turns into an interval above
    /// `Duration::MAX`; the clicker thread then dies on its first batch and the
    /// UI keeps reporting a run that is already over. Clamping rather than
    /// rejecting is what `sanitizeFields` does on the frontend, so both sides
    /// agree on what got saved.
    pub fn sanitize(&mut self) {
        clamp_f64("clickSpeed", &mut self.click_speed, limits::CLICK_SPEED);
        clamp_f64(
            "savedClickSpeed",
            &mut self.saved_click_speed,
            limits::SAVED_CLICK_SPEED,
        );
        clamp_f64("dutyCycle", &mut self.duty_cycle, limits::DUTY_CYCLE);
        clamp_f64(
            "speedRandomization",
            &mut self.speed_randomization,
            limits::SPEED_RANDOMIZATION,
        );
        clamp_f64("timeLimit", &mut self.time_limit, limits::TIME_LIMIT);

        clamp_i32("clickLimit", &mut self.click_limit, limits::CLICK_LIMIT);
        for (field, value) in [
            ("cornerStopTL", &mut self.corner_stop_tl),
            ("cornerStopTR", &mut self.corner_stop_tr),
            ("cornerStopBL", &mut self.corner_stop_bl),
            ("cornerStopBR", &mut self.corner_stop_br),
            ("edgeStopTop", &mut self.edge_stop_top),
            ("edgeStopRight", &mut self.edge_stop_right),
            ("edgeStopBottom", &mut self.edge_stop_bottom),
            ("edgeStopLeft", &mut self.edge_stop_left),
        ] {
            clamp_i32(field, value, limits::STOP_BOUNDARY);
        }

        for point in &mut self.click_points {
            clamp_u32(
                "clickPoints[].clicks",
                &mut point.clicks,
                limits::CLICK_POINT_CLICKS,
            );
            clamp_u32(
                "clickPoints[].radius",
                &mut point.radius,
                limits::CLICK_POINT_RADIUS,
            );
        }

        for zone in &mut self.stop_zones {
            clamp_i32("stopZones[].x", &mut zone.x, limits::STOP_ZONE_OFFSET);
            clamp_i32("stopZones[].y", &mut zone.y, limits::STOP_ZONE_OFFSET);
            clamp_i32("stopZones[].width", &mut zone.width, limits::STOP_ZONE_SIZE);
            clamp_i32(
                "stopZones[].height",
                &mut zone.height,
                limits::STOP_ZONE_SIZE,
            );
            sanitize_choice(
                "stopZones[].action",
                &mut zone.action,
                &limits::STOP_ZONE_ACTIONS,
                "stop",
            );
        }
        self.sanitize_pixel_trigger();
        clamp_u32(
            "startDelayMs",
            &mut self.start_delay_ms,
            limits::START_DELAY_MS,
        );

        if self.stop_zones.len() > limits::STOP_ZONE_MAX_COUNT {
            log::warn!(
                "[Settings] stopZones has {} entries, keeping the first {}",
                self.stop_zones.len(),
                limits::STOP_ZONE_MAX_COUNT
            );
            self.stop_zones.truncate(limits::STOP_ZONE_MAX_COUNT);
        }

        clamp_u32(
            "indicatorSize",
            &mut self.indicator_size,
            limits::INDICATOR_SIZE,
        );
        clamp_u32(
            "indicatorOffsetX",
            &mut self.indicator_offset_x,
            limits::INDICATOR_OFFSET,
        );
        clamp_u32(
            "indicatorOffsetY",
            &mut self.indicator_offset_y,
            limits::INDICATOR_OFFSET,
        );
        clamp_u32(
            "indicatorFlashDim",
            &mut self.indicator_flash_dim,
            limits::INDICATOR_FLASH_DIM,
        );

        sanitize_choice(
            "indicatorCorner",
            &mut self.indicator_corner,
            &limits::INDICATOR_CORNERS,
            "top-right",
        );
        sanitize_choice(
            "indicatorColorMode",
            &mut self.indicator_color_mode,
            &limits::INDICATOR_COLOR_MODES,
            "theme",
        );
        if self.indicator_countdown_mode == "off" {
            self.indicator_countdown_mode = String::from("plain");
        }
        sanitize_choice(
            "indicatorCountdownMode",
            &mut self.indicator_countdown_mode,
            &limits::INDICATOR_COUNTDOWN_MODES,
            "plain",
        );
        sanitize_choice(
            "indicatorBackgroundMode",
            &mut self.indicator_background_mode,
            &limits::INDICATOR_BACKGROUND_MODES,
            "off",
        );
        clamp_u32(
            "indicatorBackgroundStrength",
            &mut self.indicator_background_strength,
            limits::INDICATOR_FLASH_DIM,
        );
        if !is_hex_color(&self.indicator_custom_color) {
            log::warn!(
                "[Settings] indicatorCustomColor = {:?} is not #RRGGBB, using default",
                self.indicator_custom_color
            );
            self.indicator_custom_color = default_indicator_custom_color();
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn zone(id: &str, x: i32, y: i32, width: i32, height: i32, action: &str) -> StopZoneConfig {
        StopZoneConfig {
            id: id.into(),
            x,
            y,
            width,
            height,
            action: action.into(),
        }
    }

    fn sanitized(mutate: impl FnOnce(&mut ClickerSettings)) -> ClickerSettings {
        let mut settings = ClickerSettings::default();
        mutate(&mut settings);
        settings.sanitize();
        settings
    }

    #[test]
    fn sanitize_leaves_the_defaults_alone() {
        let before = ClickerSettings::default();
        let after = sanitized(|_| {});
        assert_eq!(before.click_speed, after.click_speed);
        assert_eq!(before.speed_randomization, after.speed_randomization);
        assert_eq!(before.duty_cycle, after.duty_cycle);
        assert_eq!(before.click_limit, after.click_limit);
        assert_eq!(before.corner_stop_tl, after.corner_stop_tl);
    }

    #[test]
    fn sanitize_raises_a_click_speed_that_would_overflow_a_duration() {
        for value in [5e-20, 0.0, -3.0, f64::INFINITY, f64::NEG_INFINITY, f64::NAN] {
            assert_eq!(sanitized(|s| s.click_speed = value).click_speed, 1.0);
        }
        assert_eq!(sanitized(|s| s.click_speed = 1e9).click_speed, 1_000.0);
    }

    #[test]
    fn sanitize_bounds_speed_randomization_and_duty_cycle() {
        assert_eq!(
            sanitized(|s| s.speed_randomization = 1e12).speed_randomization,
            200.0
        );
        assert_eq!(
            sanitized(|s| s.speed_randomization = -1.0).speed_randomization,
            0.0
        );
        assert_eq!(sanitized(|s| s.duty_cycle = 101.0).duty_cycle, 100.0);
    }

    #[test]
    fn sanitize_keeps_values_inside_the_range() {
        let settings = sanitized(|s| {
            s.click_speed = 500.0;
            s.speed_randomization = 35.0;
            s.duty_cycle = 45.0;
            s.time_limit = 60.0;
            s.click_limit = 1_000;
            s.corner_stop_tl = 50;
        });
        assert_eq!(settings.click_speed, 500.0);
        assert_eq!(settings.speed_randomization, 35.0);
        assert_eq!(settings.duty_cycle, 45.0);
        assert_eq!(settings.time_limit, 60.0);
        assert_eq!(settings.click_limit, 1_000);
        assert_eq!(settings.corner_stop_tl, 50);
    }

    #[test]
    fn sanitize_bounds_click_limits_and_boundaries() {
        assert_eq!(sanitized(|s| s.click_limit = 0).click_limit, 1);
        assert_eq!(
            sanitized(|s| s.click_limit = 100_000_001).click_limit,
            100_000_000
        );
        assert_eq!(sanitized(|s| s.edge_stop_left = -1).edge_stop_left, 0);
        assert_eq!(
            sanitized(|s| s.corner_stop_br = 20_000).corner_stop_br,
            10_000
        );
    }

    #[test]
    fn sanitize_bounds_click_points() {
        let settings = sanitized(|s| {
            s.click_points = vec![
                ClickPoint {
                    id: "a".into(),
                    x: 0,
                    y: 0,
                    clicks: 0,
                    radius: 50_000,
                },
                ClickPoint {
                    id: "b".into(),
                    x: 0,
                    y: 0,
                    clicks: u32::MAX,
                    radius: 10,
                },
            ];
        });
        assert_eq!(settings.click_points[0].clicks, 1);
        assert_eq!(settings.click_points[0].radius, 9_999);
        assert_eq!(settings.click_points[1].clicks, 999_999);
        assert_eq!(settings.click_points[1].radius, 10);
    }

    #[test]
    fn old_payload_without_pixel_trigger_fields_loads_defaults() {
        let mut payload = serde_json::to_value(ClickerSettings::default()).unwrap();
        let object = payload.as_object_mut().unwrap();
        for key in [
            "pixelTriggerEnabled",
            "pixelTriggerMode",
            "pixelTriggerLine",
            "pixelTriggerPoints",
            "pixelTriggerActions",
            "pixelTriggerCooldownMs",
            "pixelTriggerRecast",
            "pixelTriggerRandomize",
            "pixelTriggerCooldownMaxMs",
            "pixelTriggerHoldMs",
            "pixelTriggerHoldMaxMs",
            "startDelayMs",
        ] {
            assert!(object.remove(key).is_some(), "{key} should be serialized");
        }
        let restored: ClickerSettings = serde_json::from_value(payload).unwrap();
        assert!(!restored.pixel_trigger_enabled);
        assert_eq!(restored.pixel_trigger_mode, "line");
        assert_eq!(
            restored.pixel_trigger_line,
            PixelTriggerLineSettings::default()
        );
        assert_eq!(restored.pixel_trigger_cooldown_ms, 1_000);
        assert_eq!(restored.start_delay_ms, 0);
        assert_eq!(
            restored.pixel_trigger_recast,
            PixelTriggerRecastSettings::default()
        );
    }

    #[test]
    fn sanitize_bounds_recast_fields() {
        let settings = sanitized(|s| {
            s.pixel_trigger_recast.after_matches = u32::MAX;
            s.pixel_trigger_recast.delay_ms = u32::MAX;
            s.pixel_trigger_recast.idle_timeout_ms = u32::MAX;
            s.pixel_trigger_recast.button = String::from("Thumb");
            s.pixel_trigger_recast.steps = (0..20)
                .map(|i| PixelTriggerRecastStepSettings {
                    id: i.to_string(),
                    x: i,
                    y: i,
                    button: String::from("Thumb"),
                    delay_ms: u32::MAX,
                    delay_max_ms: 5,
                })
                .collect();
        });
        let r = &settings.pixel_trigger_recast;
        assert_eq!(r.after_matches, 9_999);
        assert_eq!(r.delay_ms, 600_000);
        assert_eq!(r.idle_timeout_ms, 600_000);
        assert_eq!(r.button, "Right");
        assert_eq!(r.steps.len(), 8);
        assert_eq!(r.steps[0].button, "Left");
        assert_eq!(r.steps[0].delay_ms, 600_000);
    }

    #[test]
    fn sanitize_bounds_pixel_trigger_fields() {
        let settings = sanitized(|s| {
            s.pixel_trigger_mode = String::from("bogus");
            s.pixel_trigger_cooldown_ms = u32::MAX;
            s.pixel_trigger_line.anchor_tolerance = 9_999;
            s.pixel_trigger_line.check_color = String::from("green");
            s.start_delay_ms = u32::MAX;
            s.pixel_trigger_points = (0..40)
                .map(|i| PixelTriggerPointSettings {
                    id: i.to_string(),
                    x: i,
                    y: i,
                    color: String::from("#zzzzzz"),
                    tolerance: 999,
                    negate: false,
                })
                .collect();
            s.pixel_trigger_actions = vec![PixelTriggerActionSettings {
                id: String::new(),
                x: 1,
                y: 2,
                delay_ms: u32::MAX,
                delay_max_ms: u32::MAX,
            }];
            s.pixel_trigger_hold_max_ms = u32::MAX;
        });
        assert_eq!(settings.pixel_trigger_mode, "line");
        assert_eq!(settings.pixel_trigger_cooldown_ms, 600_000);
        assert_eq!(settings.pixel_trigger_line.anchor_tolerance, 255);
        assert_eq!(settings.pixel_trigger_line.check_color, default_green());
        assert_eq!(settings.start_delay_ms, 60_000);
        assert_eq!(settings.pixel_trigger_points.len(), 16);
        assert_eq!(settings.pixel_trigger_points[0].tolerance, 255);
        assert_eq!(settings.pixel_trigger_points[0].color, default_white());
        assert_eq!(settings.pixel_trigger_actions[0].delay_ms, 600_000);
        assert_eq!(settings.pixel_trigger_actions[0].delay_max_ms, 600_000);
        assert_eq!(settings.pixel_trigger_hold_max_ms, 10_000);
    }

    #[test]
    fn minimize_to_tray_survives_a_round_trip() {
        for value in [true, false] {
            let settings = ClickerSettings {
                minimize_to_tray: value,
                ..ClickerSettings::default()
            };
            let json = serde_json::to_string(&settings).unwrap();
            let restored: ClickerSettings = serde_json::from_str(&json).unwrap();
            assert_eq!(restored.minimize_to_tray, value);
        }
    }

    #[test]
    fn a_payload_without_minimize_to_tray_gets_the_default() {
        let mut payload = serde_json::to_value(ClickerSettings::default()).unwrap();
        let removed = payload.as_object_mut().unwrap().remove("minimizeToTray");
        assert_eq!(removed, Some(serde_json::Value::Bool(false)));

        let restored: ClickerSettings = serde_json::from_value(payload).unwrap();
        assert!(!restored.minimize_to_tray);
        assert_eq!(restored.click_speed, 25.0);
    }

    #[test]
    fn sanitize_leaves_the_minimize_to_tray_choice_alone() {
        assert!(!ClickerSettings::default().minimize_to_tray);
        assert!(sanitized(|s| s.minimize_to_tray = true).minimize_to_tray);
    }

    #[test]
    fn sanitize_leaves_the_indicator_defaults_alone() {
        let settings = sanitized(|_| {});
        assert!(settings.indicator_enabled);
        assert_eq!(settings.indicator_corner, "top-right");
        assert_eq!(settings.indicator_color_mode, "theme");
        assert_eq!(settings.indicator_custom_color, "#ffffff");
        assert_eq!(settings.indicator_size, 32);
        assert_eq!(settings.indicator_offset_x, 24);
        assert_eq!(settings.indicator_offset_y, 24);
        assert!(!settings.indicator_flash_enabled);
        assert_eq!(settings.indicator_flash_dim, 25);
        assert_eq!(settings.indicator_countdown_mode, "words");
        assert!(!settings.indicator_countdown_enabled);
        assert_eq!(settings.indicator_background_mode, "card");
        assert_eq!(settings.indicator_background_strength, 100);
    }

    #[test]
    fn sanitize_bounds_the_indicator_numbers() {
        assert_eq!(sanitized(|s| s.indicator_size = 0).indicator_size, 16);
        assert_eq!(sanitized(|s| s.indicator_size = 513).indicator_size, 512);
        assert_eq!(
            sanitized(|s| s.indicator_offset_x = 4_001).indicator_offset_x,
            4_000
        );
        assert_eq!(
            sanitized(|s| s.indicator_offset_y = u32::MAX).indicator_offset_y,
            4_000
        );
        assert_eq!(
            sanitized(|s| s.indicator_flash_dim = 101).indicator_flash_dim,
            100
        );

        let inside = sanitized(|s| {
            s.indicator_size = 128;
            s.indicator_offset_x = 0;
            s.indicator_offset_y = 4_000;
            s.indicator_flash_dim = 0;
        });
        assert_eq!(inside.indicator_size, 128);
        assert_eq!(inside.indicator_offset_x, 0);
        assert_eq!(inside.indicator_offset_y, 4_000);
        assert_eq!(inside.indicator_flash_dim, 0);
    }

    #[test]
    fn sanitize_repairs_the_indicator_strings() {
        assert_eq!(
            sanitized(|s| s.indicator_corner = "middle".into()).indicator_corner,
            "top-right"
        );
        assert_eq!(
            sanitized(|s| s.indicator_corner = String::new()).indicator_corner,
            "top-right"
        );
        assert_eq!(
            sanitized(|s| s.indicator_corner = "bottom-left".into()).indicator_corner,
            "bottom-left"
        );

        assert_eq!(
            sanitized(|s| s.indicator_color_mode = "rainbow".into()).indicator_color_mode,
            "theme"
        );
        assert_eq!(
            sanitized(|s| s.indicator_color_mode = "custom".into()).indicator_color_mode,
            "custom"
        );
    }

    #[test]
    fn sanitize_repairs_the_indicator_custom_color() {
        for bad in ["red", "#fff", "#GGGGGG", "22c55e", "#22c55e00", ""] {
            assert_eq!(
                sanitized(|s| s.indicator_custom_color = bad.into()).indicator_custom_color,
                "#ffffff"
            );
        }
        assert_eq!(
            sanitized(|s| s.indicator_custom_color = "#ABCDEF".into()).indicator_custom_color,
            "#ABCDEF"
        );
    }

    #[test]
    fn sanitize_repairs_an_unknown_stop_zone_action() {
        let settings = sanitized(|s| {
            s.stop_zones = vec![
                zone("a", 10, 20, 100, 200, "explode"),
                zone("b", 10, 20, 100, 200, ""),
                zone("c", 10, 20, 100, 200, "Start"),
                zone("d", 10, 20, 100, 200, "pause"),
                zone("e", 10, 20, 100, 200, "start"),
            ];
        });
        assert_eq!(
            settings.stop_zones,
            vec![
                zone("a", 10, 20, 100, 200, "stop"),
                zone("b", 10, 20, 100, 200, "stop"),
                zone("c", 10, 20, 100, 200, "stop"),
                zone("d", 10, 20, 100, 200, "pause"),
                zone("e", 10, 20, 100, 200, "start"),
            ]
        );
    }

    #[test]
    fn sanitize_raises_a_zero_stop_zone_size_to_one() {
        let settings = sanitized(|s| {
            s.stop_zones = vec![zone("a", 5, 6, 0, 0, "stop")];
        });
        assert_eq!(settings.stop_zones, vec![zone("a", 5, 6, 1, 1, "stop")]);
    }

    #[test]
    fn sanitize_raises_a_negative_stop_zone_size_to_one() {
        let settings = sanitized(|s| {
            s.stop_zones = vec![zone("a", -40, 7, -500, -1, "pause")];
        });
        assert_eq!(settings.stop_zones, vec![zone("a", -40, 7, 1, 1, "pause")]);
    }

    #[test]
    fn sanitize_clamps_stop_zone_coordinates_to_the_editable_range() {
        let settings = sanitized(|s| {
            s.stop_zones = vec![
                zone("a", i32::MAX, i32::MIN, 100_000, 1, "stop"),
                zone("b", -100_000, 100_000, 1, 100_000, "stop"),
                zone("c", 99_999, -99_999, 99_999, 99_999, "stop"),
            ];
        });
        assert_eq!(
            settings.stop_zones,
            vec![
                zone("a", 99_999, -99_999, 99_999, 1, "stop"),
                zone("b", -99_999, 99_999, 1, 99_999, "stop"),
                zone("c", 99_999, -99_999, 99_999, 99_999, "stop"),
            ]
        );
        for zone in &settings.stop_zones {
            assert!(zone.x.checked_add(zone.width).is_some());
            assert!(zone.y.checked_add(zone.height).is_some());
        }
    }

    #[test]
    fn sanitize_caps_the_stop_zone_count() {
        let settings = sanitized(|s| {
            s.stop_zones = (0..(limits::STOP_ZONE_MAX_COUNT as i32 + 6))
                .map(|i| zone(&format!("z{i}"), i, i, 10, 10, "stop"))
                .collect();
        });
        assert_eq!(settings.stop_zones.len(), limits::STOP_ZONE_MAX_COUNT);
        assert_eq!(
            settings.stop_zones.first(),
            Some(&zone("z0", 0, 0, 10, 10, "stop"))
        );
        assert_eq!(
            settings.stop_zones.last(),
            Some(&zone("z63", 63, 63, 10, 10, "stop"))
        );
    }
}
