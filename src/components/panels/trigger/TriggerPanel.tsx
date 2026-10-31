import { useCallback, useEffect, useRef, useState } from "react";
import { invoke } from "@tauri-apps/api/core";
import { listen } from "@tauri-apps/api/event";
import "../advanced/sections/shared.css";
import "./TriggerPanel.css";
import type { Settings } from "../../../store";
import {
  PIXEL_DELAY_LIMIT,
  PIXEL_HOLD_LIMIT,
  PIXEL_TOLERANCE_LIMIT,
  PIXEL_TRIGGER_MAX_ITEMS,
  MOUSE_BUTTON_OPTIONS,
  RECAST_AFTER_MATCHES_LIMIT,
  sanitizeHexColor,
  type PixelTriggerRecast,
  type PixelTriggerRecastStep,
  RECAST_MAX_STEPS,
  type PixelTriggerAction,
  type PixelTriggerLine,
  type PixelTriggerMode,
  type PixelTriggerPoint,
} from "../../../settingsSchema";
import { CardDivider, NumInput, ToggleBtn } from "../advanced/sections/shared";

interface Props {
  settings: Settings;
  update: (patch: Partial<Settings>) => void;
  running: boolean;
}

export interface PixelSample {
  x: number;
  y: number;
  color: string;
  onOwnWindow?: boolean;
}

export interface PixelProbe {
  matched: boolean;
  anchorX: number | null;
  checkColor: string | null;
  points: { color: string | null; matched: boolean }[];
  error: string | null;
}

const CURSOR_POLL_MS = 100;
const PROBE_POLL_MS = 120;
const COORD_MIN = -99999;
const COORD_MAX = 99999;

const COORD_STYLE = { width: "6ch", textAlign: "right" } as const;
const TOL_STYLE = { width: "4ch", textAlign: "right" } as const;
const MS_STYLE = { width: "7ch", textAlign: "right" } as const;

function newId(): string {
  return (
    globalThis.crypto?.randomUUID?.() ??
    `px-${Date.now()}-${Math.random().toString(36).slice(2, 8)}`
  );
}

function usePolling<T>(
  command: string,
  intervalMs: number,
  enabled = true,
): T | null {
  const [value, setValue] = useState<T | null>(null);
  useEffect(() => {
    if (!enabled) return;
    let alive = true;
    let busy = false;
    const tick = async () => {
      if (busy) return;
      busy = true;
      try {
        const next = await invoke<T | null>(command);
        if (alive) setValue(next ?? null);
      } catch {
        if (alive) setValue(null);
      } finally {
        busy = false;
      }
    };
    void tick();
    const id = window.setInterval(() => void tick(), intervalMs);
    return () => {
      alive = false;
      window.clearInterval(id);
    };
  }, [command, intervalMs, enabled]);
  return enabled ? value : null;
}

interface PendingCapture {
  label: string;
}

interface PickedPayload {
  x: number;
  y: number;
}

/**
 * Blur's right-click picker: move the mouse to the spot and right-click
 * (the right click is swallowed, it never reaches the game). Esc cancels.
 */
function usePickerCapture() {
  const [pending, setPending] = useState<PendingCapture | null>(null);
  const [warning, setWarning] = useState<string | null>(null);
  const callbackRef = useRef<((sample: PixelSample) => void) | null>(null);
  const labelRef = useRef("");

  useEffect(() => {
    let disposed = false;
    const unlisteners: Array<() => void> = [];
    const keep = (promise: Promise<() => void>) =>
      void promise
        .then((unlisten) => {
          if (disposed) unlisten();
          else unlisteners.push(unlisten);
        })
        .catch(() => undefined);

    keep(
      listen<PickedPayload>("click-point-picked", (event) => {
        const done = callbackRef.current;
        if (!done) return;
        callbackRef.current = null;
        setPending(null);
        const { x, y } = event.payload;
        void invoke<PixelSample | null>("pixel_trigger_sample_at", { x, y })
          .then((sample) => {
            if (!sample) return;
            if (sample.onOwnWindow) {
              setWarning(
                `Not captured: that spot is on the Blur window. Pick ${labelRef.current} again.`,
              );
              return;
            }
            done(sample);
          })
          .catch(() => undefined);
      }),
    );
    keep(
      listen("click-pick-ended", () => {
        callbackRef.current = null;
        setPending(null);
      }),
    );

    return () => {
      disposed = true;
      unlisteners.forEach((unlisten) => unlisten());
      if (callbackRef.current) {
        callbackRef.current = null;
        void invoke("cancel_click_point_pick").catch(() => undefined);
      }
    };
  }, []);

  const cancel = useCallback(() => {
    callbackRef.current = null;
    setPending(null);
    void invoke("cancel_click_point_pick").catch(() => undefined);
  }, []);

  const start = useCallback(
    (label: string, onDone: (sample: PixelSample) => void) => {
      setWarning(null);
      callbackRef.current = onDone;
      labelRef.current = label;
      setPending({ label });
      void invoke("start_click_point_pick").catch((error) => {
        callbackRef.current = null;
        setPending(null);
        setWarning(`Picker could not start: ${String(error)}`);
      });
    },
    [],
  );

  return {
    pending,
    warning,
    start,
    cancel,
    dismissWarning: () => setWarning(null),
  };
}

/** min ms, plus "to max" when randomization is on. */
function MsRangeInput({
  min,
  max,
  randomize,
  onMin,
  onMax,
  limit = PIXEL_DELAY_LIMIT,
}: {
  min: number;
  max: number;
  randomize: boolean;
  onMin: (v: number) => void;
  onMax: (v: number) => void;
  limit?: { min: number; max: number };
}) {
  return (
    <>
      <div className="adv-numbox-sm">
        <NumInput
          value={min}
          onChange={onMin}
          min={limit.min}
          max={limit.max}
          hoverWheel={false}
          style={MS_STYLE}
        />
        {!randomize && <span className="adv-unit">ms</span>}
      </div>
      {randomize && (
        <>
          <span className="trig-dim">to</span>
          <div className="adv-numbox-sm">
            <NumInput
              value={Math.max(max, min)}
              onChange={onMax}
              min={limit.min}
              max={limit.max}
              hoverWheel={false}
              style={MS_STYLE}
            />
            <span className="adv-unit">ms</span>
          </div>
        </>
      )}
    </>
  );
}

function Swatch({ color }: { color: string | null | undefined }) {
  return (
    <span
      className="trig-swatch"
      style={{ background: color ?? "transparent" }}
      title={color ?? ""}
    />
  );
}

function ColorField({
  value,
  onChange,
}: {
  value: string;
  onChange: (hex: string) => void;
}) {
  const [draft, setDraft] = useState<string | null>(null);
  const commit = () => {
    if (draft === null) return;
    onChange(sanitizeHexColor(draft, value));
    setDraft(null);
  };
  return (
    <span className="trig-color">
      <Swatch color={value} />
      <input
        className="trig-hex"
        value={draft ?? value}
        spellCheck={false}
        maxLength={7}
        onChange={(e) => setDraft(e.target.value)}
        onBlur={commit}
        onKeyDown={(e) => {
          if (e.key === "Enter") commit();
          if (e.key === "Escape") setDraft(null);
        }}
      />
    </span>
  );
}

function Btn({
  children,
  onClick,
  disabled,
  title,
}: {
  children: React.ReactNode;
  onClick: () => void;
  disabled?: boolean;
  title?: string;
}) {
  return (
    <button
      type="button"
      className="adv-hotkey-edit-btn trig-btn"
      onClick={onClick}
      disabled={disabled}
      title={title}
    >
      {children}
    </button>
  );
}

function Label({ children }: { children: React.ReactNode }) {
  return <span className="trig-label">{children}</span>;
}

export default function TriggerPanel({ settings, update, running }: Props) {
  const liveColor = settings.pixelTriggerLiveColor;
  const randomize = settings.pixelTriggerRandomize;
  const cursor = usePolling<PixelSample>(
    "pixel_trigger_sample_cursor",
    CURSOR_POLL_MS,
    liveColor,
  );
  const probe = usePolling<PixelProbe>("pixel_trigger_probe", PROBE_POLL_MS);
  const capture = usePickerCapture();

  const line = settings.pixelTriggerLine;
  const points = settings.pixelTriggerPoints;
  const actions = settings.pixelTriggerActions;
  const mode = settings.pixelTriggerMode;

  const patchLine = useCallback(
    (patch: Partial<PixelTriggerLine>) =>
      update({ pixelTriggerLine: { ...line, ...patch } }),
    [line, update],
  );
  const patchPoint = (id: string, patch: Partial<PixelTriggerPoint>) =>
    update({
      pixelTriggerPoints: points.map((p) =>
        p.id === id ? { ...p, ...patch } : p,
      ),
    });
  const patchAction = (id: string, patch: Partial<PixelTriggerAction>) =>
    update({
      pixelTriggerActions: actions.map((a) =>
        a.id === id ? { ...a, ...patch } : a,
      ),
    });

  // Captures are applied against the settings at the time they finish.
  const settingsRef = useRef(settings);
  useEffect(() => {
    settingsRef.current = settings;
  });

  const captureLine = (
    label: string,
    apply: (s: PixelSample) => Partial<PixelTriggerLine>,
  ) =>
    capture.start(label, (s) =>
      update({
        pixelTriggerLine: {
          ...settingsRef.current.pixelTriggerLine,
          ...apply(s),
        },
      }),
    );

  const addPoint = () =>
    capture.start("the pixel to watch", (s) =>
      update({
        pixelTriggerPoints: [
          ...settingsRef.current.pixelTriggerPoints,
          {
            id: newId(),
            x: s.x,
            y: s.y,
            color: s.color,
            tolerance: 30,
            negate: false,
          },
        ].slice(0, PIXEL_TRIGGER_MAX_ITEMS),
      }),
    );

  const addAction = () =>
    capture.start("the click position", (s) => {
      const current = settingsRef.current.pixelTriggerActions;
      update({
        pixelTriggerActions: [
          ...current,
          {
            id: newId(),
            x: s.x,
            y: s.y,
            delayMs: current.length === 0 ? 0 : 1500,
            delayMaxMs: current.length === 0 ? 0 : 2000,
          },
        ].slice(0, PIXEL_TRIGGER_MAX_ITEMS),
      });
    });

  const recapturePoint = (id: string) =>
    capture.start("the pixel to watch", (s) =>
      update({
        pixelTriggerPoints: settingsRef.current.pixelTriggerPoints.map((p) =>
          p.id === id ? { ...p, x: s.x, y: s.y, color: s.color } : p,
        ),
      }),
    );

  const recaptureAction = (id: string) =>
    capture.start("the click position", (s) =>
      update({
        pixelTriggerActions: settingsRef.current.pixelTriggerActions.map((a) =>
          a.id === id ? { ...a, x: s.x, y: s.y } : a,
        ),
      }),
    );

  const recast = settings.pixelTriggerRecast;
  const patchRecast = (patch: Partial<PixelTriggerRecast>) =>
    update({ pixelTriggerRecast: { ...recast, ...patch } });
  const patchRecastStep = (
    id: string,
    patch: Partial<PixelTriggerRecastStep>,
  ) =>
    patchRecast({
      steps: recast.steps.map((step) =>
        step.id === id ? { ...step, ...patch } : step,
      ),
    });
  const addRecastStep = () =>
    capture.start("the next recast click", (s) => {
      const current = settingsRef.current.pixelTriggerRecast;
      const first = current.steps.length === 0;
      update({
        pixelTriggerRecast: {
          ...current,
          steps: [
            ...current.steps,
            {
              id: newId(),
              x: s.x,
              y: s.y,
              // First step: the cast itself (right click, after a pause).
              // Further steps: quick left clicks, e.g. closing a popup.
              button: first ? ("Right" as const) : ("Left" as const),
              delayMs: first ? 1500 : 300,
              delayMaxMs: first ? 2500 : 600,
            },
          ].slice(0, RECAST_MAX_STEPS),
        },
      });
    });
  const recaptureRecastStep = (id: string) =>
    capture.start("the recast click", (s) => {
      const current = settingsRef.current.pixelTriggerRecast;
      update({
        pixelTriggerRecast: {
          ...current,
          steps: current.steps.map((step) =>
            step.id === id ? { ...step, x: s.x, y: s.y } : step,
          ),
        },
      });
    });
  const moveRecastStep = (index: number, delta: number) => {
    const target = index + delta;
    if (target < 0 || target >= recast.steps.length) return;
    const steps = [...recast.steps];
    [steps[index], steps[target]] = [steps[target], steps[index]];
    patchRecast({ steps });
  };

  // On-screen preview of lines, pixels and click positions (overlay window).
  const showOnScreen = settings.pixelTriggerOverlay;
  useEffect(() => {
    if (!showOnScreen) return;
    void invoke("set_pixel_trigger_overlay", {
      data: {
        mode,
        line,
        points: points.map((p, i) => ({
          x: p.x,
          y: p.y,
          color: p.color,
          negate: p.negate,
          matched: probe?.points[i]?.matched ?? false,
        })),
        actions: actions.map((a) => ({ x: a.x, y: a.y, delayMs: a.delayMs })),
        matched: probe?.matched ?? false,
        anchorX: probe?.anchorX ?? null,
        recast: recast.enabled
          ? recast.steps
              .filter((step) => step.x !== 0 || step.y !== 0)
              .map((step, i) => ({
                index: i + 1,
                x: step.x,
                y: step.y,
                button: step.button,
                delayMs: step.delayMs,
              }))
          : [],
      },
    }).catch(() => undefined);
  }, [showOnScreen, mode, line, points, actions, probe, recast]);
  useEffect(() => {
    if (!showOnScreen) return;
    return () => {
      void invoke("set_pixel_trigger_overlay", { data: null }).catch(
        () => undefined,
      );
    };
  }, [showOnScreen]);

  const statusText = probe?.error
    ? probe.error
    : mode === "line"
      ? probe?.anchorX != null
        ? `Marker at x ${probe.anchorX}`
        : "Marker not found on line A"
      : `${probe?.points.filter((p) => p.matched).length ?? 0} / ${points.length} pixels match`;

  return (
    <div className="adv-panel adv-panel-text trig-panel">
      <div className="adv-sectioncontainer">
        <div className="adv-card-header">
          <span className="adv-card-title">Pixel Trigger</span>
          <ToggleBtn
            value={settings.pixelTriggerEnabled}
            onChange={(v) => update({ pixelTriggerEnabled: v })}
          />
        </div>
        <CardDivider />
        <div className="adv-card-desc">
          Watches screen pixels and clicks when they match. Start and stop with
          your hotkey. While enabled it replaces the normal click speed.
        </div>

        <div className="trig-status-row">
          <span
            className={`trig-pill ${probe?.matched ? "trig-pill--on" : ""}`}
          >
            {probe?.matched ? "MATCH" : "no match"}
          </span>
          <span className="trig-status-text">{statusText}</span>
          {running && settings.pixelTriggerEnabled && (
            <span className="trig-running">running</span>
          )}
          {liveColor && (
            <span className="trig-cursor">
              <Swatch color={cursor?.color} />
              <span>{cursor?.color ?? "—"}</span>
              <span className="trig-dim">
                {cursor ? `${cursor.x}, ${cursor.y}` : ""}
              </span>
            </span>
          )}
        </div>

        {capture.warning && !capture.pending && (
          <div className="trig-capture trig-capture--warn">
            <span>{capture.warning}</span>
            <Btn onClick={capture.dismissWarning}>OK</Btn>
          </div>
        )}

        {capture.pending && (
          <div className="trig-capture">
            <span>
              <b>Right-click</b> on {capture.pending.label}. Esc cancels.
            </span>
            <Btn onClick={capture.cancel}>Cancel</Btn>
          </div>
        )}

        <div className="trig-row">
          <Label>Live colour under cursor</Label>
          <ToggleBtn
            value={liveColor}
            onChange={(v) => update({ pixelTriggerLiveColor: v })}
          />
        </div>

        <div className="trig-row">
          <Label>Show on screen</Label>
          <ToggleBtn
            value={showOnScreen}
            onChange={(v) => update({ pixelTriggerOverlay: v })}
          />
          <span className="trig-dim">
            Lines, pixels and click positions as an overlay.
          </span>
        </div>

        <div className="trig-row">
          <Label>Mode</Label>
          <div className="adv-seg-group">
            {(
              [
                ["line", "Line match"],
                ["points", "Pixel match"],
              ] as [PixelTriggerMode, string][]
            ).map(([value, label]) => (
              <button
                key={value}
                type="button"
                className={`adv-seg-btn ${mode === value ? "active" : ""}`}
                onClick={() => update({ pixelTriggerMode: value })}
              >
                {label}
              </button>
            ))}
          </div>
        </div>
      </div>

      {mode === "line" ? (
        <div className="adv-sectioncontainer">
          <div className="adv-card-header">
            <span className="adv-card-title">Line Match</span>
          </div>
          <CardDivider />
          <div className="adv-card-desc">
            Finds the colour of line A (e.g. a white marker) and clicks when
            line B has its colour in the same column (e.g. the green zone).
          </div>

          <div className="trig-row">
            <Label>Line A · marker</Label>
            <ColorField
              value={line.anchorColor}
              onChange={(anchorColor) => patchLine({ anchorColor })}
            />
            <span className="trig-dim">±</span>
            <div className="adv-numbox-sm">
              <NumInput
                value={line.anchorTolerance}
                onChange={(anchorTolerance) => patchLine({ anchorTolerance })}
                min={PIXEL_TOLERANCE_LIMIT.min}
                max={PIXEL_TOLERANCE_LIMIT.max}
                hoverWheel={false}
                style={TOL_STYLE}
              />
            </div>
            <span className="trig-dim">y</span>
            <div className="adv-numbox-sm">
              <NumInput
                value={line.anchorY}
                onChange={(anchorY) => patchLine({ anchorY })}
                min={COORD_MIN}
                max={COORD_MAX}
                hoverWheel={false}
                style={COORD_STYLE}
              />
            </div>
            <Btn
              onClick={() =>
                captureLine("the marker (line A)", (s) => ({
                  anchorY: s.y,
                  anchorColor: s.color,
                }))
              }
            >
              Capture
            </Btn>
          </div>

          <div className="trig-row">
            <Label>Line B · target zone</Label>
            <ColorField
              value={line.checkColor}
              onChange={(checkColor) => patchLine({ checkColor })}
            />
            <span className="trig-dim">±</span>
            <div className="adv-numbox-sm">
              <NumInput
                value={line.checkTolerance}
                onChange={(checkTolerance) => patchLine({ checkTolerance })}
                min={PIXEL_TOLERANCE_LIMIT.min}
                max={PIXEL_TOLERANCE_LIMIT.max}
                hoverWheel={false}
                style={TOL_STYLE}
              />
            </div>
            <span className="trig-dim">y</span>
            <div className="adv-numbox-sm">
              <NumInput
                value={line.checkY}
                onChange={(checkY) => patchLine({ checkY })}
                min={COORD_MIN}
                max={COORD_MAX}
                hoverWheel={false}
                style={COORD_STYLE}
              />
            </div>
            <Btn
              onClick={() =>
                captureLine("the target zone (line B)", (s) => ({
                  checkY: s.y,
                  checkColor: s.color,
                }))
              }
            >
              Capture
            </Btn>
          </div>

          <div className="trig-row">
            <Label>Range x</Label>
            <div className="adv-numbox-sm">
              <NumInput
                value={line.x1}
                onChange={(x1) => patchLine({ x1 })}
                min={COORD_MIN}
                max={COORD_MAX}
                hoverWheel={false}
                style={COORD_STYLE}
              />
            </div>
            <span className="trig-dim">to</span>
            <div className="adv-numbox-sm">
              <NumInput
                value={line.x2}
                onChange={(x2) => patchLine({ x2 })}
                min={COORD_MIN}
                max={COORD_MAX}
                hoverWheel={false}
                style={COORD_STYLE}
              />
            </div>
            <Btn
              onClick={() =>
                captureLine("the LEFT end of the bar", (s) => ({ x1: s.x }))
              }
            >
              Capture left
            </Btn>
            <Btn
              onClick={() =>
                captureLine("the RIGHT end of the bar", (s) => ({ x2: s.x }))
              }
            >
              Capture right
            </Btn>
          </div>
          {mode === "line" && probe?.checkColor && (
            <div className="trig-row trig-dim">
              Line B under marker: <Swatch color={probe.checkColor} />{" "}
              {probe.checkColor}
            </div>
          )}
        </div>
      ) : (
        <div className="adv-sectioncontainer">
          <div className="adv-card-header">
            <span className="adv-card-title">Pixel Match</span>
            <Btn
              onClick={addPoint}
              disabled={points.length >= PIXEL_TRIGGER_MAX_ITEMS}
            >
              + Capture pixel
            </Btn>
          </div>
          <CardDivider />
          <div className="adv-card-desc">
            All pixels must have their colour at the same time (or must not,
            when “not” is on).
          </div>
          {points.length === 0 && (
            <div className="trig-empty">No pixels yet.</div>
          )}
          {points.map((p, i) => (
            <div className="trig-row" key={p.id}>
              <span
                className={`trig-index ${probe?.points[i]?.matched ? "trig-index--on" : ""}`}
              >
                {i + 1}
              </span>
              <span className="trig-dim">x</span>
              <div className="adv-numbox-sm">
                <NumInput
                  value={p.x}
                  onChange={(x) => patchPoint(p.id, { x })}
                  min={COORD_MIN}
                  max={COORD_MAX}
                  hoverWheel={false}
                  style={COORD_STYLE}
                />
              </div>
              <span className="trig-dim">y</span>
              <div className="adv-numbox-sm">
                <NumInput
                  value={p.y}
                  onChange={(y) => patchPoint(p.id, { y })}
                  min={COORD_MIN}
                  max={COORD_MAX}
                  hoverWheel={false}
                  style={COORD_STYLE}
                />
              </div>
              <ColorField
                value={p.color}
                onChange={(color) => patchPoint(p.id, { color })}
              />
              <span className="trig-dim">±</span>
              <div className="adv-numbox-sm">
                <NumInput
                  value={p.tolerance}
                  onChange={(tolerance) => patchPoint(p.id, { tolerance })}
                  min={PIXEL_TOLERANCE_LIMIT.min}
                  max={PIXEL_TOLERANCE_LIMIT.max}
                  hoverWheel={false}
                  style={TOL_STYLE}
                />
              </div>
              <button
                type="button"
                className={`adv-seg-btn trig-not ${p.negate ? "active" : ""}`}
                onClick={() => patchPoint(p.id, { negate: !p.negate })}
                title="Match when the pixel does NOT have this colour"
              >
                not
              </button>
              <Btn onClick={() => recapturePoint(p.id)}>Capture</Btn>
              <Btn
                onClick={() =>
                  update({
                    pixelTriggerPoints: points.filter((x) => x.id !== p.id),
                  })
                }
                title="Remove"
              >
                ×
              </Btn>
            </div>
          ))}
        </div>
      )}

      <div className="adv-sectioncontainer">
        <div className="adv-card-header">
          <span className="adv-card-title">On Match</span>
          <Btn
            onClick={addAction}
            disabled={actions.length >= PIXEL_TRIGGER_MAX_ITEMS}
          >
            + Capture click position
          </Btn>
        </div>
        <CardDivider />
        <div className="adv-card-desc">
          Clicks these positions in order with the mouse button from Simple /
          Advanced. “Wait” is the pause before that click, e.g. step 2 recasts
          after 1500 ms.
        </div>
        {actions.length === 0 && (
          <div className="trig-empty">No click positions yet.</div>
        )}
        {actions.map((a, i) => (
          <div className="trig-row" key={a.id}>
            <span className="trig-index">{i + 1}</span>
            <span className="trig-dim">wait</span>
            <MsRangeInput
              min={a.delayMs}
              max={a.delayMaxMs}
              randomize={randomize}
              onMin={(delayMs) => patchAction(a.id, { delayMs })}
              onMax={(delayMaxMs) => patchAction(a.id, { delayMaxMs })}
            />
            <span className="trig-dim">x</span>
            <div className="adv-numbox-sm">
              <NumInput
                value={a.x}
                onChange={(x) => patchAction(a.id, { x })}
                min={COORD_MIN}
                max={COORD_MAX}
                hoverWheel={false}
                style={COORD_STYLE}
              />
            </div>
            <span className="trig-dim">y</span>
            <div className="adv-numbox-sm">
              <NumInput
                value={a.y}
                onChange={(y) => patchAction(a.id, { y })}
                min={COORD_MIN}
                max={COORD_MAX}
                hoverWheel={false}
                style={COORD_STYLE}
              />
            </div>
            <Btn onClick={() => recaptureAction(a.id)}>Capture</Btn>
            <Btn
              onClick={() =>
                update({
                  pixelTriggerActions: actions.filter((x) => x.id !== a.id),
                })
              }
              title="Remove"
            >
              ×
            </Btn>
          </div>
        ))}
        <div className="trig-row">
          <Label>Cooldown after clicks</Label>
          <MsRangeInput
            min={settings.pixelTriggerCooldownMs}
            max={settings.pixelTriggerCooldownMaxMs}
            randomize={randomize}
            onMin={(pixelTriggerCooldownMs) =>
              update({ pixelTriggerCooldownMs })
            }
            onMax={(pixelTriggerCooldownMaxMs) =>
              update({ pixelTriggerCooldownMaxMs })
            }
          />
        </div>
        <div className="trig-row">
          <Label>Button held for</Label>
          <MsRangeInput
            min={settings.pixelTriggerHoldMs}
            max={settings.pixelTriggerHoldMaxMs}
            randomize={randomize}
            limit={PIXEL_HOLD_LIMIT}
            onMin={(pixelTriggerHoldMs) => update({ pixelTriggerHoldMs })}
            onMax={(pixelTriggerHoldMaxMs) => update({ pixelTriggerHoldMaxMs })}
          />
        </div>
        <div className="trig-row">
          <Label>Random timings</Label>
          <ToggleBtn
            value={randomize}
            onChange={(v) => update({ pixelTriggerRandomize: v })}
          />
          <span className="trig-dim">
            Every wait, cooldown and hold picks a random value in its range
            (e.g. 50 to 100 ms).
          </span>
        </div>
      </div>

      <div className="adv-sectioncontainer">
        <div className="adv-card-header">
          <span className="adv-card-title">Recast</span>
          <ToggleBtn
            value={recast.enabled}
            onChange={(enabled) => patchRecast({ enabled })}
          />
        </div>
        <CardDivider />
        <div className="adv-card-desc">
          Clicks to throw the line again, after a number of matches and/or when
          nothing matched for a while. Anti-disruption: put a step before the
          cast that clicks away notifications or popups.
        </div>
        <div className="trig-row">
          <Label>After matches</Label>
          <div className="adv-numbox-sm">
            <NumInput
              value={recast.afterMatches}
              onChange={(afterMatches) => patchRecast({ afterMatches })}
              min={RECAST_AFTER_MATCHES_LIMIT.min}
              max={RECAST_AFTER_MATCHES_LIMIT.max}
              hoverWheel={false}
              style={TOL_STYLE}
            />
          </div>
          <span className="trig-dim">(0 = only when idle)</span>
        </div>
        {recast.steps.length === 0 && (
          <div className="trig-empty">No recast clicks yet.</div>
        )}
        {recast.steps.map((step, i) => (
          <div className="trig-row" key={step.id}>
            <span className="trig-index trig-index--recast">{i + 1}</span>
            <span className="trig-dim">wait</span>
            <MsRangeInput
              min={step.delayMs}
              max={step.delayMaxMs}
              randomize={randomize}
              onMin={(delayMs) => patchRecastStep(step.id, { delayMs })}
              onMax={(delayMaxMs) => patchRecastStep(step.id, { delayMaxMs })}
            />
            <div className="adv-seg-group">
              {MOUSE_BUTTON_OPTIONS.map((button) => (
                <button
                  key={button}
                  type="button"
                  className={`adv-seg-btn ${step.button === button ? "active" : ""}`}
                  onClick={() => patchRecastStep(step.id, { button })}
                  title={`${button} click`}
                >
                  {button[0]}
                </button>
              ))}
            </div>
            <span className="trig-dim">x</span>
            <div className="adv-numbox-sm">
              <NumInput
                value={step.x}
                onChange={(x) => patchRecastStep(step.id, { x })}
                min={COORD_MIN}
                max={COORD_MAX}
                hoverWheel={false}
                style={COORD_STYLE}
              />
            </div>
            <span className="trig-dim">y</span>
            <div className="adv-numbox-sm">
              <NumInput
                value={step.y}
                onChange={(y) => patchRecastStep(step.id, { y })}
                min={COORD_MIN}
                max={COORD_MAX}
                hoverWheel={false}
                style={COORD_STYLE}
              />
            </div>
            <Btn onClick={() => recaptureRecastStep(step.id)}>Capture</Btn>
            <Btn
              onClick={() => moveRecastStep(i, -1)}
              disabled={i === 0}
              title="Move up"
            >
              ↑
            </Btn>
            <Btn
              onClick={() => moveRecastStep(i, 1)}
              disabled={i === recast.steps.length - 1}
              title="Move down"
            >
              ↓
            </Btn>
            <Btn
              onClick={() =>
                patchRecast({
                  steps: recast.steps.filter((x) => x.id !== step.id),
                })
              }
              title="Remove"
            >
              ×
            </Btn>
          </div>
        ))}
        <div className="trig-row">
          <Btn
            onClick={addRecastStep}
            disabled={recast.steps.length >= RECAST_MAX_STEPS}
          >
            + Capture recast click
          </Btn>
          <span className="trig-dim">
            Steps run top to bottom, each after its wait. L / R / M = mouse
            button.
          </span>
        </div>
        <div className="trig-row">
          <Label>Recast if idle for</Label>
          <div className="adv-numbox-sm">
            <NumInput
              value={recast.idleTimeoutMs}
              onChange={(idleTimeoutMs) => patchRecast({ idleTimeoutMs })}
              min={PIXEL_DELAY_LIMIT.min}
              max={PIXEL_DELAY_LIMIT.max}
              hoverWheel={false}
              style={MS_STYLE}
            />
            <span className="adv-unit">ms</span>
          </div>
          <span className="trig-dim">
            No match for this long → recast (0 = off). Also casts after start.
          </span>
        </div>
      </div>
    </div>
  );
}
