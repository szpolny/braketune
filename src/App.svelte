<script lang="ts">
  import { onMount } from "svelte";
  import {
    MAX_PERCENT,
    MIN_PERCENT,
    getAutostart,
    getStatus,
    onStatusChanged,
    setAutostart,
    setBrakeForce,
    type Status,
  } from "./api";

  let status = $state<Status | null>(null);
  let level = $state(MAX_PERCENT);
  let error = $state("");
  let autostart = $state(false);
  let timer: ReturnType<typeof setTimeout>;

  const connected = $derived(status?.device != null);
  const fill = $derived(((level - MIN_PERCENT) / (MAX_PERCENT - MIN_PERCENT)) * 100);

  function schedule() {
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
    const unlisten = onStatusChanged((next) => (status = next));
    return () => unlisten.then((fn) => fn());
  });
</script>

<main>
  <header>
    <h1>BrakeTune</h1>
    <div class="chip" class:on={connected}>
      <span class="dot"></span>
      {status?.device?.name ?? "No pedals detected"}
    </div>
  </header>

  <section class="card">
    <div class="label">Brake force</div>
    <div class="readout">
      <span class="value">{level}</span><span class="max">%</span>
    </div>

    <input
      type="range"
      min={MIN_PERCENT}
      max={MAX_PERCENT}
      step="1"
      bind:value={level}
      oninput={schedule}
      disabled={!connected}
      style="--fill: {fill}%"
    />
    <div class="hint"><span>Light</span><span>Full axis</span></div>
    {#if error}<p class="error">{error}</p>{/if}
  </section>

  <p class="note">
    Applied live. In your game or simulator, select the controller named
    <strong>BrakeTune Pedals</strong>. The original pedals are hidden while
    BrakeTune is running.
  </p>

  <label class="toggle">
    <input type="checkbox" checked={autostart} onchange={toggleAutostart} />
    <span>Start with system and keep running in the tray</span>
  </label>

  <footer>Unofficial. Not affiliated with Fanatec.</footer>
</main>

<style>
  main {
    display: flex;
    flex-direction: column;
    gap: 20px;
    padding: 28px 28px 20px;
    min-height: 100vh;
  }
  header { display: flex; flex-direction: column; gap: 12px; }
  h1 { font-size: 22px; font-weight: 700; letter-spacing: 0.02em; }
  .chip {
    align-self: flex-start;
    display: flex;
    align-items: center;
    gap: 8px;
    padding: 6px 12px;
    border: 1px solid var(--line);
    border-radius: 999px;
    font-size: 13px;
    color: var(--muted);
  }
  .chip.on { color: var(--text); }
  .dot { width: 8px; height: 8px; border-radius: 50%; background: #565b66; }
  .chip.on .dot { background: var(--ok); box-shadow: 0 0 8px var(--ok); }

  .card {
    background: var(--panel);
    border: 1px solid var(--line);
    border-radius: 16px;
    padding: 24px;
    display: flex;
    flex-direction: column;
    gap: 14px;
  }
  .label { font-size: 12px; text-transform: uppercase; letter-spacing: 0.12em; color: var(--muted); }
  .readout { display: flex; align-items: baseline; gap: 8px; }
  .value {
    font-size: 72px;
    font-weight: 800;
    line-height: 1;
    background: linear-gradient(180deg, var(--accent), var(--accent-2));
    -webkit-background-clip: text;
    background-clip: text;
    color: transparent;
  }
  .max { color: var(--muted); font-size: 20px; }

  input[type="range"] {
    -webkit-appearance: none;
    appearance: none;
    width: 100%;
    height: 8px;
    border-radius: 999px;
    background: linear-gradient(90deg, var(--accent) var(--fill), var(--line) var(--fill));
    outline: none;
    cursor: pointer;
  }
  input[type="range"]:disabled { opacity: 0.4; cursor: not-allowed; }
  input[type="range"]::-webkit-slider-thumb {
    -webkit-appearance: none;
    width: 22px;
    height: 22px;
    border-radius: 50%;
    background: #fff;
    border: 3px solid var(--accent);
  }
  .hint { display: flex; justify-content: space-between; font-size: 12px; color: var(--muted); }

  .error { color: #ff6b6b; font-size: 13px; user-select: text; }

  .note { font-size: 13px; line-height: 1.5; color: var(--muted); }
  .note strong { color: var(--text); }

  .toggle { display: flex; align-items: center; gap: 10px; font-size: 14px; color: var(--muted); cursor: pointer; }
  .toggle input { accent-color: var(--accent); width: 16px; height: 16px; }

  footer { margin-top: auto; font-size: 12px; color: var(--muted); text-align: center; }
</style>
