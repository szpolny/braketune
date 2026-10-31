<script lang="ts">
  import { onMount } from "svelte";
  import { getCurrentWindow } from "@tauri-apps/api/window";
  import logo from "../assets/icon.svg";
  import {
    MAX_PERCENT,
    MIN_PERCENT,
    getAutostart,
    getPedal,
    getStatus,
    onPedal,
    onStatusChanged,
    setAutostart,
    setBrakeForce,
    type Pedal,
    type Status,
  } from "./api";

  const appWindow = getCurrentWindow();
  const SCALE = [0, 25, 50, 75, 100];

  let status = $state<Status | null>(null);
  let pedal = $state<Pedal>({ input: 0, output: 0 });
  let level = $state(MAX_PERCENT);
  let error = $state("");
  let autostart = $state(false);
  let dragging = $state(false);
  let meter: HTMLDivElement;
  let timer: ReturnType<typeof setTimeout>;

  const connected = $derived(status?.device != null);
  const input = $derived(connected ? pedal.input * 100 : 0);
  const output = $derived(connected ? pedal.output * 100 : 0);

  function setLevel(next: number) {
    next = Math.round(Math.min(MAX_PERCENT, Math.max(MIN_PERCENT, next)));
    if (next === level) return;
    level = next;
    clearTimeout(timer);
    timer = setTimeout(async () => {
      error = "";
      try {
        status = await setBrakeForce(level);
      } catch (e) {
        error = String(e);
      }
    }, 120);
  }

  function levelAt(e: PointerEvent) {
    const rect = meter.getBoundingClientRect();
    return ((e.clientX - rect.left) / rect.width) * 100;
  }

  function pointerDown(e: PointerEvent) {
    meter.setPointerCapture(e.pointerId);
    dragging = true;
    setLevel(levelAt(e));
  }

  function pointerMove(e: PointerEvent) {
    if (dragging) setLevel(levelAt(e));
  }

  function keyDown(e: KeyboardEvent) {
    const step: Record<string, number> = {
      ArrowUp: 1,
      ArrowRight: 1,
      ArrowDown: -1,
      ArrowLeft: -1,
      PageUp: 10,
      PageDown: -10,
    };
    if (e.key === "Home") setLevel(MIN_PERCENT);
    else if (e.key === "End") setLevel(MAX_PERCENT);
    else if (e.key in step) setLevel(level + step[e.key]);
    else return;
    e.preventDefault();
  }

  async function toggleAutostart(event: Event) {
    const enabled = (event.currentTarget as HTMLInputElement).checked;
    try {
      autostart = await setAutostart(enabled);
    } catch (e) {
      error = String(e);
    }
  }

  onMount(() => {
    getAutostart().then((enabled) => (autostart = enabled));
    getStatus().then((next) => {
      status = next;
      level = next.settings.brake_force;
    });
    getPedal().then((next) => (pedal = next));
    const unlisten = [
      onStatusChanged((next) => (status = next)),
      onPedal((next) => (pedal = next)),
    ];
    return () => unlisten.forEach((p) => p.then((fn) => fn()));
  });
</script>

<div class="titlebar" data-tauri-drag-region>
  <img src={logo} alt="" data-tauri-drag-region />
  <span data-tauri-drag-region>BrakeTune</span>
  <button class="win" aria-label="Minimize" onclick={() => appWindow.minimize()}>
    <svg viewBox="0 0 10 10"><path d="M1 5h8" /></svg>
  </button>
  <button class="win close" aria-label="Close to tray" onclick={() => appWindow.close()}>
    <svg viewBox="0 0 10 10"><path d="M1 1l8 8M9 1l-8 8" /></svg>
  </button>
</div>

<main>
  <p class="device" class:on={connected}>
    <span class="led"></span>
    {status?.device?.name ?? "No pedals connected"}
  </p>

  <section class="force">
    <div>
      <div class="label">Brake force</div>
      <div class="big">{level}<small>%</small></div>
    </div>
    <div class="stepper">
      <button aria-label="Decrease" onclick={() => setLevel(level - 1)}>−</button>
      <button aria-label="Increase" onclick={() => setLevel(level + 1)}>+</button>
    </div>
  </section>

  <section class="instrument">
    <div
      class="meter"
      class:dragging
      bind:this={meter}
      role="slider"
      tabindex="0"
      aria-label="Brake force"
      aria-valuemin={MIN_PERCENT}
      aria-valuemax={MAX_PERCENT}
      aria-valuenow={level}
      onpointerdown={pointerDown}
      onpointermove={pointerMove}
      onpointerup={() => (dragging = false)}
      onpointercancel={() => (dragging = false)}
      onkeydown={keyDown}
      style="--cap: {level}%; --in: {input}%"
    >
      <div class="maxed"></div>
      <div class="travel"></div>
      <div class="cap"></div>
    </div>
    <div class="game" style="--out: {output}%"><div></div></div>
    <div class="scale" aria-hidden="true">
      {#each SCALE as mark}<span style="left: {mark}%">{mark}</span>{/each}
    </div>
  </section>

  <dl>
    <div>
      <dt><i class="key game"></i>Game input</dt>
      <dd class="hot">{#if connected}{Math.round(output)}<small>%</small>{:else}—{/if}</dd>
    </div>
    <div>
      <dt><i class="key travel"></i>Pedal input</dt>
      <dd>{#if connected}{Math.round(input)}<small>%</small>{:else}—{/if}</dd>
    </div>
  </dl>

  <p class="help">
    Drag the white marker to set brake force: pressing the pedal up to the
    marker gives full braking in game. Lower values need less pressure.
  </p>

  {#if error}<p class="error">{error}</p>{/if}

  <label class="setting">
    <span>
      Start at login
      <small>Stays in the tray when the window is closed</small>
    </span>
    <input type="checkbox" checked={autostart} onchange={toggleAutostart} />
    <span class="switch"></span>
  </label>

  <p class="note">
    In your game, select the controller <strong>BrakeTune Pedals</strong>. The
    original pedals are hidden while BrakeTune runs.
  </p>

  <footer>Unofficial. Not affiliated with Fanatec.</footer>
</main>

<style>
  .titlebar {
    display: flex;
    align-items: center;
    gap: 8px;
    height: 36px;
    padding-left: 14px;
    border-bottom: 1px solid var(--line);
    font-size: 12px;
    font-weight: 600;
    color: var(--muted);
  }
  .titlebar img { width: 16px; height: 16px; }
  .titlebar span { flex: 1; }
  .win {
    width: 44px;
    height: 100%;
    display: grid;
    place-items: center;
    border: 0;
    background: none;
    color: var(--muted);
    cursor: pointer;
  }
  .win svg { width: 10px; height: 10px; stroke: currentColor; stroke-width: 1.2; fill: none; }
  .win:hover { background: var(--raised); color: var(--text); }
  .win.close:hover { background: #c42b1c; color: #fff; }

  main {
    display: flex;
    flex-direction: column;
    gap: 16px;
    padding: 16px 20px 12px;
    width: 100%;
    max-width: 400px;
    min-height: calc(100vh - 36px);
    margin: 0 auto;
  }

  .device { display: flex; align-items: center; gap: 8px; font-size: 13px; color: var(--muted); }
  .device.on { color: var(--text); }
  .led { width: 6px; height: 6px; border-radius: 50%; background: var(--line-strong); }
  .device.on .led { background: var(--ok); }

  .force { display: flex; align-items: flex-end; justify-content: space-between; }
  .label, dt { font-size: 12px; color: var(--muted); }
  .big {
    font: 600 56px/1.05 var(--mono);
    letter-spacing: -0.04em;
    font-variant-numeric: tabular-nums;
  }
  small { font-size: 0.4em; color: var(--muted); margin-left: 2px; letter-spacing: 0; }

  .stepper { display: flex; gap: 6px; margin-bottom: 8px; }
  .stepper button {
    width: 40px;
    height: 32px;
    border: 1px solid var(--line);
    border-radius: 3px;
    background: var(--surface);
    color: var(--text);
    font: 15px var(--mono);
    cursor: pointer;
  }
  .stepper button:hover { border-color: var(--line-strong); background: var(--raised); }

  .meter {
    position: relative;
    height: 56px;
    background: var(--surface);
    border: 1px solid var(--line);
    border-radius: 3px;
    overflow: hidden;
    cursor: ew-resize;
    touch-action: none;
    outline: none;
  }
  .meter:focus-visible { border-color: var(--text); }
  .meter > div { position: absolute; top: 0; bottom: 0; left: 0; }
  .meter::after {
    content: "";
    position: absolute;
    inset: 0;
    background: repeating-linear-gradient(90deg, transparent 0 calc(10% - 1px), rgba(255, 255, 255, 0.05) calc(10% - 1px) 10%);
    pointer-events: none;
  }
  .maxed {
    left: var(--cap) !important;
    right: 0;
    background: repeating-linear-gradient(-45deg, var(--line-strong) 0 1px, transparent 1px 7px), var(--well);
  }
  .travel { width: var(--in); background: var(--accent-dim); }
  .game { height: 4px; margin-top: 6px; background: var(--well); border-radius: 2px; overflow: hidden; }
  .game div { width: var(--out); height: 100%; background: var(--accent); }
  .cap {
    left: var(--cap) !important;
    width: 2px;
    margin-left: -1px;
    background: var(--text);
  }
  .cap::before,
  .cap::after {
    content: "";
    position: absolute;
    left: -4px;
    border: 5px solid transparent;
  }
  .cap::before { top: 0; border-top-color: var(--text); }
  .cap::after { bottom: 0; border-bottom-color: var(--text); }

  .scale { position: relative; height: 14px; margin-top: 6px; font: 10px var(--mono); color: var(--faint); }
  .scale span { position: absolute; transform: translateX(-50%); }
  .scale span:first-child { transform: none; }
  .scale span:last-child { transform: translateX(-100%); }

  dl { display: grid; grid-template-columns: 1fr 1fr; gap: 16px; border-top: 1px solid var(--line); padding-top: 12px; }
  dl > div { display: flex; flex-direction: column; gap: 4px; }
  dt { display: flex; align-items: center; gap: 8px; }
  dd { font: 500 24px var(--mono); font-variant-numeric: tabular-nums; }
  dd.hot { color: var(--accent); }
  .key { width: 8px; height: 8px; }
  .key.game { background: var(--accent); }
  .key.travel { background: var(--accent-dim); }
  .help { font-size: 12px; line-height: 1.45; color: var(--faint); }

  .error { color: #ff6b5b; font-size: 13px; user-select: text; }

  .setting {
    display: flex;
    align-items: center;
    justify-content: space-between;
    gap: 16px;
    padding: 12px 0;
    border-top: 1px solid var(--line);
    border-bottom: 1px solid var(--line);
    font-size: 14px;
    cursor: pointer;
  }
  .setting small { display: block; margin: 2px 0 0; font-size: 12px; }
  .setting input { position: absolute; opacity: 0; pointer-events: none; }
  .switch {
    flex: none;
    position: relative;
    width: 34px;
    height: 18px;
    border-radius: 9px;
    background: var(--line-strong);
    transition: background 0.15s;
  }
  .switch::after {
    content: "";
    position: absolute;
    top: 2px;
    left: 2px;
    width: 14px;
    height: 14px;
    border-radius: 50%;
    background: var(--text);
    transition: transform 0.15s;
  }
  .setting input:checked + .switch { background: var(--accent); }
  .setting input:checked + .switch::after { transform: translateX(16px); }
  .setting input:focus-visible + .switch { outline: 1px solid var(--text); outline-offset: 2px; }

  .note { font-size: 12px; line-height: 1.5; color: var(--muted); }
  .note strong { color: var(--text); font-weight: 600; }

  footer { margin-top: auto; font-size: 11px; color: var(--faint); }
</style>
