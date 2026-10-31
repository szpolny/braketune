import { invoke } from "@tauri-apps/api/core";
import { listen } from "@tauri-apps/api/event";

export interface Device {
  name: string;
  product_id: number;
}

export interface Settings {
  brake_force: number;
}

export interface Status {
  device: Device | null;
  settings: Settings;
}

/** Brake position as fractions (0–1) of the full axis. */
export interface Pedal {
  /** How far the physical pedal is pressed. */
  input: number;
  /** What games see after scaling. */
  output: number;
}

export const MIN_PERCENT = 1;
export const MAX_PERCENT = 100;

export const getStatus = () => invoke<Status>("get_status");
export const setBrakeForce = (percent: number) =>
  invoke<Status>("set_brake_force", { percent });
export const onStatusChanged = (handler: (status: Status) => void) =>
  listen<Status>("status-changed", (e) => handler(e.payload));
export const getPedal = () => invoke<Pedal>("get_pedal");
export const onPedal = (handler: (pedal: Pedal) => void) =>
  listen<Pedal>("pedal", (e) => handler(e.payload));
export const getAutostart = () => invoke<boolean>("get_autostart");
export const setAutostart = (enabled: boolean) =>
  invoke<boolean>("set_autostart", { enabled });
