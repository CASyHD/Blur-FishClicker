//! Pixel trigger: watch screen pixels and click when a colour condition holds.
//!
//! Two rule types (see `pixel_match.rs` for the pure matching logic):
//! - **Line**: find the anchor colour on line A (e.g. a white marker sliding
//!   under a bar) and require the check colour on line B in the same column
//!   (e.g. the green "catch" zone of the bar).
//! - **Points**: every configured pixel must (or must not) have its colour.
//!
//! When the rule matches, the configured click actions run in order (each with
//! an optional delay before it), followed by a cooldown.

use std::sync::atomic::Ordering;
use std::time::{Duration, Instant};

use tauri::Manager;
use windows_sys::Win32::Graphics::Gdi::{
    BitBlt, CreateCompatibleDC, CreateDIBSection, DeleteDC, DeleteObject, GetDC, ReleaseDC,
    SelectObject, BITMAPINFO, BITMAPINFOHEADER, DIB_RGB_COLORS, HBITMAP, HDC, HGDIOBJ, SRCCOPY,
};

use super::failsafe::should_stop_for_failsafe;
use super::mouse::{current_cursor_position, get_button_flags, move_mouse, send_mouse_event};
use super::pixel_match::{
    evaluate_line, ColorRule, LineEvaluation, LineRule, MsRange, PointRule, Rgb,
};
use super::process;
use super::rng::SmallRng;
use super::worker::{emit_status, sleep_interruptible, RunControl};
use super::ClickerConfig;
use super::CLICK_COUNT;
use crate::error::{AppError, AppResult};
use crate::settings::{PixelTriggerLineSettings, PixelTriggerPointSettings};
use crate::ClickerSettings;
use crate::ClickerState;

/// Time between moving the cursor and pressing, so browser games register the hover.
const MOVE_SETTLE: Duration = Duration::from_millis(15);
/// Sleep between polls. With the 1 ms timer resolution the run already holds,
/// capture time dominates the loop.
const POLL_SLEEP: Duration = Duration::from_millis(1);
/// Give up after this many consecutive failed captures (~ a few seconds).
const MAX_CAPTURE_FAILURES: u32 = 2_000;
/// Hard cap on the capture rectangle so a misconfigured line cannot grab half a screen per poll.
const MAX_CAPTURE_PIXELS: usize = 4_000_000;

pub const MAX_POINTS: usize = 16;
pub const MAX_ACTIONS: usize = 16;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct PixelTriggerAction {
    pub x: i32,
    pub y: i32,
    pub delay: MsRange,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum PixelTriggerRule {
    Line(LineRule),
    Points(Vec<PointRule>),
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct PixelTriggerConfig {
    pub rule: PixelTriggerRule,
    pub actions: Vec<PixelTriggerAction>,
    pub cooldown: MsRange,
    /// How long each click holds the button down.
    pub hold: MsRange,
    pub recast: Option<PixelTriggerRecast>,
}

/// One click of the recast sequence.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct RecastStep {
    /// Pause before this click.
    pub delay: MsRange,
    pub x: i32,
    pub y: i32,
    /// 1 = left, 2 = right, 3 = middle (same as `ClickerConfig::button`).
    pub button: i32,
}

/// "Throw the line again": a sequence of clicks, each with its own position,
/// button and wait. Extra steps can close notifications first (anti-disruption).
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct PixelTriggerRecast {
    /// Recast after this many matches (0 = only the idle fallback).
    pub after_matches: u32,
    pub steps: Vec<RecastStep>,
    /// Recast when nothing matched for this long (0 = off).
    pub idle_timeout_ms: u64,
}

pub const MAX_RECAST_STEPS: usize = 8;

fn button_from_name(name: &str) -> i32 {
    match name {
        "Right" => 2,
        "Middle" => 3,
        _ => 1,
    }
}

fn recast_from_settings(
    settings: &ClickerSettings,
    randomize: bool,
) -> AppResult<Option<PixelTriggerRecast>> {
    let r = &settings.pixel_trigger_recast;
    if !r.enabled {
        return Ok(None);
    }
    let steps: Vec<RecastStep> = if r.steps.is_empty() && (r.x != 0 || r.y != 0) {
        // Settings from before recast sequences: one step from the legacy fields.
        vec![RecastStep {
            delay: MsRange::new(u64::from(r.delay_ms), u64::from(r.delay_max_ms), randomize),
            x: r.x,
            y: r.y,
            button: button_from_name(&r.button),
        }]
    } else {
        r.steps
            .iter()
            .take(MAX_RECAST_STEPS)
            .map(|step| RecastStep {
                delay: MsRange::new(
                    u64::from(step.delay_ms),
                    u64::from(step.delay_max_ms),
                    randomize,
                ),
                x: step.x,
                y: step.y,
                button: button_from_name(&step.button),
            })
            .collect()
    };
    if steps.is_empty() {
        return Err(AppError::InvalidSetting(String::from(
            "Pixel trigger: add at least one recast click",
        )));
    }
    if let Some(index) = steps.iter().position(|step| step.x == 0 && step.y == 0) {
        return Err(AppError::InvalidSetting(format!(
            "Pixel trigger: capture the position of recast step {}",
            index + 1
        )));
    }
    if r.after_matches == 0 && r.idle_timeout_ms == 0 {
        return Err(AppError::InvalidSetting(String::from(
            "Pixel trigger: recast needs a match count or an idle timeout",
        )));
    }
    Ok(Some(PixelTriggerRecast {
        after_matches: r.after_matches,
        steps,
        idle_timeout_ms: u64::from(r.idle_timeout_ms),
    }))
}

fn color_rule(hex: &str, tolerance: u32, what: &str) -> AppResult<ColorRule> {
    let color = Rgb::parse_hex(hex).ok_or_else(|| {
        AppError::InvalidSetting(format!("Pixel trigger: invalid {what} colour '{hex}'"))
    })?;
    Ok(ColorRule {
        color,
        tolerance: tolerance.min(255) as u8,
    })
}

pub fn line_rule_from_settings(line: &PixelTriggerLineSettings) -> AppResult<LineRule> {
    let rule = LineRule::new(
        line.x1,
        line.x2,
        line.anchor_y,
        line.check_y,
        color_rule(&line.anchor_color, line.anchor_tolerance, "line A")?,
        color_rule(&line.check_color, line.check_tolerance, "line B")?,
    );
    if rule.width() < 2 {
        return Err(AppError::InvalidSetting(String::from(
            "Pixel trigger: set the left and right edge of the line first",
        )));
    }
    if rule.width().saturating_mul(rule.height()) > MAX_CAPTURE_PIXELS {
        return Err(AppError::InvalidSetting(String::from(
            "Pixel trigger: line A and line B are too far apart",
        )));
    }
    Ok(rule)
}

pub fn point_rules_from_settings(
    points: &[PixelTriggerPointSettings],
) -> AppResult<Vec<PointRule>> {
    let rules = points
        .iter()
        .take(MAX_POINTS)
        .map(|p| -> AppResult<PointRule> {
            Ok(PointRule {
                x: p.x,
                y: p.y,
                rule: color_rule(&p.color, p.tolerance, "point")?,
                negate: p.negate,
            })
        })
        .collect::<AppResult<Vec<_>>>()?;
    if rules.is_empty() {
        return Err(AppError::InvalidSetting(String::from(
            "Pixel trigger: add at least one pixel",
        )));
    }
    Ok(rules)
}

pub fn rule_from_settings(settings: &ClickerSettings) -> AppResult<PixelTriggerRule> {
    if settings.pixel_trigger_mode == "points" {
        Ok(PixelTriggerRule::Points(point_rules_from_settings(
            &settings.pixel_trigger_points,
        )?))
    } else {
        Ok(PixelTriggerRule::Line(line_rule_from_settings(
            &settings.pixel_trigger_line,
        )?))
    }
}

/// `None` when the trigger is switched off.
pub fn config_from_settings(settings: &ClickerSettings) -> AppResult<Option<PixelTriggerConfig>> {
    if !settings.pixel_trigger_enabled {
        return Ok(None);
    }
    let rule = rule_from_settings(settings)?;
    let randomize = settings.pixel_trigger_randomize;
    let actions: Vec<PixelTriggerAction> = settings
        .pixel_trigger_actions
        .iter()
        .take(MAX_ACTIONS)
        .map(|a| PixelTriggerAction {
            x: a.x,
            y: a.y,
            delay: MsRange::new(u64::from(a.delay_ms), u64::from(a.delay_max_ms), randomize),
        })
        .collect();
    if actions.is_empty() {
        return Err(AppError::InvalidSetting(String::from(
            "Pixel trigger: add at least one click position",
        )));
    }
    Ok(Some(PixelTriggerConfig {
        rule,
        actions,
        cooldown: MsRange::new(
            u64::from(settings.pixel_trigger_cooldown_ms),
            u64::from(settings.pixel_trigger_cooldown_max_ms),
            randomize,
        ),
        hold: MsRange::new(
            u64::from(settings.pixel_trigger_hold_ms),
            u64::from(settings.pixel_trigger_hold_max_ms),
            randomize,
        ),
        recast: recast_from_settings(settings, randomize)?,
    }))
}

// ---------------------------------------------------------------------------
// Screen capture (GDI). Small rectangles only, so BitBlt is cheap enough.
// ---------------------------------------------------------------------------

pub struct ScreenGrabber {
    screen_dc: HDC,
    mem_dc: HDC,
    bitmap: HBITMAP,
    old_object: HGDIOBJ,
    bits: *mut u32,
    width: usize,
    height: usize,
}

impl ScreenGrabber {
    pub fn new() -> Option<Self> {
        unsafe {
            let screen_dc = GetDC(std::ptr::null_mut());
            if screen_dc.is_null() {
                return None;
            }
            let mem_dc = CreateCompatibleDC(screen_dc);
            if mem_dc.is_null() {
                ReleaseDC(std::ptr::null_mut(), screen_dc);
                return None;
            }
            Some(Self {
                screen_dc,
                mem_dc,
                bitmap: std::ptr::null_mut(),
                old_object: std::ptr::null_mut(),
                bits: std::ptr::null_mut(),
                width: 0,
                height: 0,
            })
        }
    }

    fn release_bitmap(&mut self) {
        unsafe {
            if !self.bitmap.is_null() {
                SelectObject(self.mem_dc, self.old_object);
                DeleteObject(self.bitmap);
            }
        }
        self.bitmap = std::ptr::null_mut();
        self.old_object = std::ptr::null_mut();
        self.bits = std::ptr::null_mut();
        self.width = 0;
        self.height = 0;
    }

    fn ensure_size(&mut self, width: usize, height: usize) -> bool {
        if !self.bitmap.is_null() && self.width >= width && self.height >= height {
            return true;
        }
        self.release_bitmap();
        unsafe {
            let mut header: BITMAPINFOHEADER = std::mem::zeroed();
            header.biSize = std::mem::size_of::<BITMAPINFOHEADER>() as u32;
            header.biWidth = width as i32;
            header.biHeight = -(height as i32); // top-down
            header.biPlanes = 1;
            header.biBitCount = 32;
            header.biCompression = 0; // BI_RGB
            let bmi = BITMAPINFO {
                bmiHeader: header,
                bmiColors: [std::mem::zeroed()],
            };
            let mut bits: *mut std::ffi::c_void = std::ptr::null_mut();
            let bitmap = CreateDIBSection(
                self.mem_dc,
                &bmi as *const _,
                DIB_RGB_COLORS,
                &mut bits,
                std::ptr::null_mut(),
                0,
            );
            if bitmap.is_null() || bits.is_null() {
                if !bitmap.is_null() {
                    DeleteObject(bitmap);
                }
                return false;
            }
            self.old_object = SelectObject(self.mem_dc, bitmap);
            self.bitmap = bitmap;
            self.bits = bits as *mut u32;
            self.width = width;
            self.height = height;
        }
        true
    }

    /// Copies the screen rectangle and returns it row by row (`width` pixels per row).
    pub fn capture(&mut self, x: i32, y: i32, width: usize, height: usize) -> Option<Vec<Rgb>> {
        if width == 0 || height == 0 || width.saturating_mul(height) > MAX_CAPTURE_PIXELS {
            return None;
        }
        if !self.ensure_size(width, height) {
            return None;
        }
        unsafe {
            let ok = BitBlt(
                self.mem_dc,
                0,
                0,
                width as i32,
                height as i32,
                self.screen_dc,
                x,
                y,
                SRCCOPY,
            );
            if ok == 0 {
                return None;
            }
            let stride = self.width;
            let pixels = std::slice::from_raw_parts(self.bits, stride * self.height);
            let mut out = Vec::with_capacity(width * height);
            for row in 0..height {
                let start = row * stride;
                out.extend(
                    pixels[start..start + width]
                        .iter()
                        .map(|&p| Rgb::from_bgrx(p)),
                );
            }
            Some(out)
        }
    }

    pub fn pixel(&mut self, x: i32, y: i32) -> Option<Rgb> {
        self.capture(x, y, 1, 1).and_then(|v| v.first().copied())
    }
}

impl Drop for ScreenGrabber {
    fn drop(&mut self) {
        self.release_bitmap();
        unsafe {
            DeleteDC(self.mem_dc);
            ReleaseDC(std::ptr::null_mut(), self.screen_dc);
        }
    }
}

// ---------------------------------------------------------------------------
// Evaluation
// ---------------------------------------------------------------------------

#[derive(Clone, Debug, Default, serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct PointProbe {
    pub color: Option<String>,
    pub matched: bool,
}

#[derive(Clone, Debug, Default, serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct PixelTriggerProbe {
    pub matched: bool,
    /// Line mode: absolute x of the detected anchor.
    pub anchor_x: Option<i32>,
    /// Line mode: colour on line B at the anchor column.
    pub check_color: Option<String>,
    /// Points mode: per-point result, same order as the settings.
    pub points: Vec<PointProbe>,
    pub error: Option<String>,
}

fn evaluate_line_rule(grabber: &mut ScreenGrabber, rule: &LineRule) -> Option<LineEvaluation> {
    let width = rule.width();
    let top = rule.top();
    let pixels = grabber.capture(rule.x1, top, width, rule.height())?;
    let row = |y: i32| {
        let start = (y - top) as usize * width;
        &pixels[start..start + width]
    };
    Some(evaluate_line(
        row(rule.anchor_y),
        row(rule.check_y),
        &rule.anchor,
        &rule.check,
    ))
}

/// One poll. `None` = capture failed.
fn evaluate(grabber: &mut ScreenGrabber, rule: &PixelTriggerRule) -> Option<bool> {
    match rule {
        PixelTriggerRule::Line(line) => evaluate_line_rule(grabber, line).map(|e| e.matched),
        PixelTriggerRule::Points(points) => {
            for p in points {
                if !p.matches(grabber.pixel(p.x, p.y)?) {
                    return Some(false);
                }
            }
            Some(!points.is_empty())
        }
    }
}

/// Detailed single evaluation for the live preview in the UI.
pub fn probe(settings: &ClickerSettings) -> PixelTriggerProbe {
    let rule = match rule_from_settings(settings) {
        Ok(rule) => rule,
        Err(e) => {
            return PixelTriggerProbe {
                error: Some(e.to_string()),
                ..PixelTriggerProbe::default()
            }
        }
    };
    let Some(mut grabber) = ScreenGrabber::new() else {
        return PixelTriggerProbe {
            error: Some(String::from("Screen capture unavailable")),
            ..PixelTriggerProbe::default()
        };
    };
    match rule {
        PixelTriggerRule::Line(line) => match evaluate_line_rule(&mut grabber, &line) {
            Some(eval) => PixelTriggerProbe {
                matched: eval.matched,
                anchor_x: eval.anchor_offset.map(|o| line.x1 + o as i32),
                check_color: eval.check_color.map(Rgb::to_hex),
                ..PixelTriggerProbe::default()
            },
            None => PixelTriggerProbe {
                error: Some(String::from("Screen capture failed")),
                ..PixelTriggerProbe::default()
            },
        },
        PixelTriggerRule::Points(points) => {
            let results: Vec<PointProbe> = points
                .iter()
                .map(|p| {
                    let color = grabber.pixel(p.x, p.y);
                    PointProbe {
                        color: color.map(Rgb::to_hex),
                        matched: color.is_some_and(|c| p.matches(c)),
                    }
                })
                .collect();
            PixelTriggerProbe {
                matched: !results.is_empty() && results.iter().all(|r| r.matched),
                points: results,
                ..PixelTriggerProbe::default()
            }
        }
    }
}

#[derive(Clone, Debug, serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct PixelSample {
    pub x: i32,
    pub y: i32,
    pub color: String,
    /// The cursor was over Blur's own main window (capture almost certainly unintended).
    pub on_own_window: bool,
}

/// `true` when the screen point lies on Blur's visible main window. Clicking
/// there would operate Blur itself, e.g. its close button.
pub fn point_on_main_window(app: &tauri::AppHandle, x: i32, y: i32) -> bool {
    let Some(window) = app.get_webview_window("main") else {
        return false;
    };
    if !window.is_visible().unwrap_or(false) || window.is_minimized().unwrap_or(false) {
        return false;
    }
    let (Ok(pos), Ok(size)) = (window.outer_position(), window.outer_size()) else {
        return false;
    };
    let width = i32::try_from(size.width).unwrap_or(i32::MAX);
    let height = i32::try_from(size.height).unwrap_or(i32::MAX);
    (pos.x..pos.x.saturating_add(width)).contains(&x)
        && (pos.y..pos.y.saturating_add(height)).contains(&y)
}

/// Cursor position plus the colour underneath it.
pub fn sample_at_cursor(app: &tauri::AppHandle) -> Option<PixelSample> {
    let (x, y) = current_cursor_position()?;
    sample_at(app, x, y)
}

/// A screen position plus the colour there (right-click picker).
pub fn sample_at(app: &tauri::AppHandle, x: i32, y: i32) -> Option<PixelSample> {
    let color = ScreenGrabber::new()?.pixel(x, y)?;
    Some(PixelSample {
        x,
        y,
        color: color.to_hex(),
        on_own_window: point_on_main_window(app, x, y),
    })
}

// ---------------------------------------------------------------------------
// Run loop
// ---------------------------------------------------------------------------

fn click_at(x: i32, y: i32, down: u32, up: u32, hold: Duration) -> bool {
    move_mouse(x, y);
    std::thread::sleep(MOVE_SETTLE);
    if !send_mouse_event(down) {
        return false;
    }
    if !hold.is_zero() {
        std::thread::sleep(hold);
    }
    send_mouse_event(up)
}

#[inline]
fn pick(range: MsRange, rng: &mut SmallRng) -> Duration {
    Duration::from_millis(range.at(rng.next_f64()))
}

/// Clicks unless the point is on Blur's own window. `Err` carries the stop reason.
fn guarded_click(
    app: &tauri::AppHandle,
    label: &str,
    x: i32,
    y: i32,
    (down, up): (u32, u32),
    hold: Duration,
) -> Result<(), String> {
    if point_on_main_window(app, x, y) {
        log::warn!("[PixelTrigger] {label} at ({x}, {y}) is on the Blur window, stopping");
        return Err(format!(
            "{label} is on the Blur window. Move Blur away or capture it again."
        ));
    }
    log::info!(
        "[PixelTrigger] {label} at ({x}, {y}), hold {} ms",
        hold.as_millis()
    );
    if !click_at(x, y, down, up, hold) {
        return Err(String::from(
            "Windows rejected the simulated click (elevated or secure window?)",
        ));
    }
    Ok(())
}

/// Runs every recast step in order. `Ok` = number of clicks sent.
fn run_recast(
    app: &tauri::AppHandle,
    recast: &PixelTriggerRecast,
    control: &RunControl,
    rng: &mut SmallRng,
    hold: MsRange,
) -> Result<i64, String> {
    let no_abort = || false;
    let mut sent: i64 = 0;
    for (index, step) in recast.steps.iter().enumerate() {
        let wait = pick(step.delay, rng);
        if !wait.is_zero() {
            sleep_interruptible(wait, control, &no_abort);
        }
        if !control.is_active() {
            break;
        }
        let label = format!("Recast step {}", index + 1);
        let hold = pick(hold, rng);
        guarded_click(
            app,
            &label,
            step.x,
            step.y,
            get_button_flags(step.button),
            hold,
        )?;
        sent += 1;
    }
    Ok(sent)
}

/// Runs until stopped. Returns `(click_count, stop_reason)`.
pub fn run_trigger_loop(
    config: &ClickerConfig,
    trigger: &PixelTriggerConfig,
    control: &RunControl,
    start_time: Instant,
) -> (i64, String) {
    let Some(mut grabber) = ScreenGrabber::new() else {
        return (0, String::from("Pixel trigger: screen capture unavailable"));
    };
    let app = control.app();
    let state = app.state::<ClickerState>();
    let buttons = get_button_flags(config.button);
    let no_abort = || false;
    let mut rng = SmallRng::new();
    let mut clicks: i64 = 0;
    let mut capture_failures: u32 = 0;
    let mut matches_since_recast: u32 = 0;
    let mut last_activity = Instant::now();
    log::info!(
        "[PixelTrigger] started: {:?}, {} click action(s), cooldown {:?}, hold {:?}, recast {:?}",
        trigger.rule,
        trigger.actions.len(),
        trigger.cooldown,
        trigger.hold,
        trigger.recast
    );

    while control.is_active() {
        if config.limit > 0 && clicks >= i64::from(config.limit) {
            return (clicks, format!("Click limit reached ({})", config.limit));
        }
        if let Some(reason) = should_stop_for_failsafe(config) {
            return (clicks, reason);
        }
        if config.task_switcher_stop_enabled && process::is_task_switcher_active() {
            return (clicks, String::from("Blocked by Alt+Tab"));
        }
        if let Some(reason) = process::check_process_list(config) {
            return (clicks, reason);
        }
        if config.time_limit > 0.0 && start_time.elapsed().as_secs_f64() >= config.time_limit {
            return (
                clicks,
                format!("Time limit reached ({:.1}s)", config.time_limit),
            );
        }
        if state.paused.load(Ordering::SeqCst) {
            std::thread::sleep(Duration::from_millis(10));
            continue;
        }

        match evaluate(&mut grabber, &trigger.rule) {
            Some(true) => {
                capture_failures = 0;
                matches_since_recast += 1;
                log::info!(
                    "[PixelTrigger] match #{} after {:.2}s",
                    matches_since_recast,
                    start_time.elapsed().as_secs_f64()
                );
                for (index, action) in trigger.actions.iter().enumerate() {
                    let wait = pick(action.delay, &mut rng);
                    if !wait.is_zero() {
                        sleep_interruptible(wait, control, &no_abort);
                    }
                    if !control.is_active() {
                        break;
                    }
                    let label = format!("Click position {}", index + 1);
                    if let Err(reason) = guarded_click(
                        app,
                        &label,
                        action.x,
                        action.y,
                        buttons,
                        pick(trigger.hold, &mut rng),
                    ) {
                        return (clicks, reason);
                    }
                    clicks += 1;
                    CLICK_COUNT.store(clicks, Ordering::Relaxed);
                }

                if let Some(recast) = trigger.recast.as_ref() {
                    if recast.after_matches > 0 && matches_since_recast >= recast.after_matches {
                        log::info!(
                            "[PixelTrigger] recast after {} match(es), {} step(s)",
                            matches_since_recast,
                            recast.steps.len()
                        );
                        match run_recast(app, recast, control, &mut rng, trigger.hold) {
                            Ok(sent) => {
                                clicks += sent;
                                CLICK_COUNT.store(clicks, Ordering::Relaxed);
                            }
                            Err(reason) => return (clicks, reason),
                        }
                        matches_since_recast = 0;
                    }
                }

                last_activity = Instant::now();
                emit_status(app);
                let cooldown = pick(trigger.cooldown, &mut rng);
                if !cooldown.is_zero() {
                    sleep_interruptible(cooldown, control, &no_abort);
                }
            }
            Some(false) => {
                capture_failures = 0;
                if let Some(recast) = trigger.recast.as_ref() {
                    if recast.idle_timeout_ms > 0
                        && last_activity.elapsed() >= Duration::from_millis(recast.idle_timeout_ms)
                    {
                        log::info!(
                            "[PixelTrigger] no match for {} ms, recasting",
                            recast.idle_timeout_ms
                        );
                        match run_recast(app, recast, control, &mut rng, trigger.hold) {
                            Ok(sent) => {
                                clicks += sent;
                                CLICK_COUNT.store(clicks, Ordering::Relaxed);
                            }
                            Err(reason) => return (clicks, reason),
                        }
                        matches_since_recast = 0;
                        last_activity = Instant::now();
                        emit_status(app);
                    }
                }
            }
            None => {
                capture_failures += 1;
                if capture_failures >= MAX_CAPTURE_FAILURES {
                    return (clicks, String::from("Pixel trigger: screen capture failed"));
                }
            }
        }
        std::thread::sleep(POLL_SLEEP);
    }

    (clicks, String::from("Stopped"))
}
