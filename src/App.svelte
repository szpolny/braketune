<script lang="ts">
  import { onMount } from "svelte";
  import {
    MAX_PERCENT,
    MIN_PERCENT,
    getStatus,
    onStatusChanged,
    setApplyOnConnect,
    setBrakeForce,
    type Status,
  } from "./api";

  let status = $state<Status | null>(null);
  let level = $state(50);
  let error = $state("");
  let saved = $state(false);
  let savedTimer: ReturnType<typeof setTimeout>;

  const connected = $derived(status?.device != null);
  const fill = $derived(((level - MIN_PERCENT) / (MAX_PERCENT - MIN_PERCENT)) * 100);

  function sync(next: Status) {
    status = next;
    level = next.settings.brake_force;
  }

  async function apply() {
    error = "";
    try {
      sync(await setBrakeForce(level));
      saved = true;
      clearTimeout(savedTimer);
      savedTimer = setTimeout(() => (saved = false), 1500);
    } catch (e) {
      error = String(e);
    }
  }

  async function toggleAuto(event: Event) {
    const enabled = (event.currentTarget as HTMLInputElement).checked;
    sync(await setApplyOnConnect(enabled));
  }

  onMount(() => {
    getStatus().then(sync);
    const unlisten = onStatusChanged((next) => {
      status = next;
      if (!next.device) saved = false;
    });
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
      disabled={!connected}
      style="--fill: {fill}%"
    />
    <div class="hint"><span>Softer</span><span>Stiffer</span></div>

    <button class="apply" onclick={apply} disabled={!connected}>
      {saved ? "Applied ✓" : "Apply to pedals"}
    </button>
    {#if error}<p class="error">{error}</p>{/if}
  </section>

  <label class="toggle">
    <input
      type="checkbox"
      checked={status?.settings.apply_on_connect ?? true}
      onchange={toggleAuto}
    />
    <span>Re-apply automatically when pedals connect</span>
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

  .apply {
    margin-top: 6px;
    padding: 14px;
    border: 0;
    border-radius: 12px;
    font: inherit;
    font-weight: 600;
    color: #fff;
    background: linear-gradient(90deg, var(--accent), var(--accent-2));
    cursor: pointer;
  }
  .apply:disabled { opacity: 0.35; cursor: not-allowed; }
  .error { color: #ff6b6b; font-size: 13px; user-select: text; }

  .toggle { display: flex; align-items: center; gap: 10px; font-size: 14px; color: var(--muted); cursor: pointer; }
  .toggle input { accent-color: var(--accent); width: 16px; height: 16px; }

  footer { margin-top: auto; font-size: 12px; color: var(--muted); text-align: center; }
</style>
