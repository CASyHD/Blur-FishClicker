# Blur FishClicker

> **Unofficial fork** of [Blur-AutoClicker](https://github.com/Blur009/Blur-AutoClicker) by [Blur009](https://github.com/Blur009).
> All core features (Simple/Advanced mode, hotkeys, click points, zones, presets …) come from the original –
> see the original repo for its documentation, releases and support. This fork adds a **pixel trigger** and a few extras.
> Please do **not** report issues that only affect this fork to the original project.

---

## What this fork adds

### 🎯 Pixel Trigger (new "Trigger" tab)
Watches screen pixels and clicks as soon as a colour condition is met – e.g. timing/fishing mini-games where a marker
slides over a target zone. While enabled it replaces the normal click speed; start/stop with your usual hotkey.

- **Line match** – line A (marker, e.g. a white arrow) and line B (target zone, e.g. a green area).
  Blur finds the marker on line A and clicks when line B has the target colour **in the same column**.
  Works even when the target zone moves.
- **Pixel match** – up to 16 fixed pixels that must all have (or, with "not", must *not* have) their colour at the same time.
- Colour + tolerance per line/pixel, millisecond polling (GDI capture of only the strip that is needed).
- Live **MATCH / no match** status including the detected marker position.

### 🖱️ Click sequence on match ("On Match")
- Several click positions in a row, each with its own wait before it.
- **Cooldown** after the sequence and an adjustable mouse-button **hold time**.

### 🔁 Recast sequence / anti-disruption
- Recasts after **N matches** – or when **nothing matched for X ms** (idle fallback, also fires right after starting).
- Up to **8 steps**, each with its own position, mouse button (L/R/M) and wait, reorderable.
  Use it to click away notifications/popups first and then cast.

### 🎲 Random timings
- Optionally use a **range** instead of a fixed value for every wait, the cooldown, the recast steps and the hold time (e.g. 50–100 ms).

### 🧭 Usability & display
- **Right-click picker** to capture position + colour (the right click is swallowed, Esc cancels).
- **On-screen overlay** (can be turned off): lines A/B, pixels, click and recast positions with labels.
  Markers are drawn *next to* the sampled pixels so they never affect detection.
- Optional **live colour readout** under the cursor (off by default).
- **Self-click protection:** positions on the Blur window are rejected when capturing and blocked (with a stop message) when clicking.
- All trigger settings are stored in **presets**.

### ⏱️ Start delay
- *Settings → Behaviour → Start Delay*: waits X ms after the hotkey before the first click (press the hotkey again to cancel).

---

## Quick start (fishing mini-game example)

1. Open the **Trigger** tab, mode **Line match**.
2. **Line A → Capture**: right-click at the height of the marker (set the colour to `#ffffff` if needed).
3. **Line B → Capture**: right-click inside the target zone.
4. **Capture left / right**: left and right end of the bar.
5. Check: status/overlay shows **MATCH** while the marker is inside the zone.
6. **On Match**: capture your click position(s). **Recast**: capture the steps (e.g. close popup → right-click to cast) and set *After matches* and/or *Recast if idle for*.
7. Turn the trigger **On** and press your hotkey.

Windowed or borderless mode is recommended – reading pixels may fail in exclusive fullscreen.

---

## Building

Requirements (Windows): Node.js 20+, Rust (MSVC toolchain), Visual Studio Build Tools (C++), WebView2.
See [BUILDING.md](BUILDING.md) for details.

```powershell
npm ci
npm run dev      # development
npm run build    # installer -> src-tauri\target\release\bundle\nsis\
```

---

## Notes

- Use at your own risk. Automation may violate the terms of service of individual games or websites.
- Not an official Blur009 product.

## License

Like the original, licensed under the [GNU General Public License v3.0](LICENSE).
This fork is a **modified version** of Blur-AutoClicker (© Blur009); the changes (pixel trigger, recast, random timings, overlay, start delay) were made in 2026 by CASyHD.
Original: https://github.com/Blur009/Blur-AutoClicker
