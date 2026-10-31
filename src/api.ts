import { invoke } from "@tauri-apps/api/core";
import { listen } from "@tauri-apps/api/event";

export interface Device {
  name: string;
  product_id: number;
}

export interface Settings {
  brake_force: number;
  apply_on_connect: boolean;
}

export interface Status {
  device: Device | null;
  settings: Settings;
}

export const MIN_LEVEL = 1;
export const MAX_LEVEL = 10;

export const getStatus = () => invoke<Status>("get_status");
export const setBrakeForce = (level: number) =>
  invoke<Status>("set_brake_force", { level });
export const setApplyOnConnect = (enabled: boolean) =>
  invoke<Status>("set_apply_on_connect", { enabled });
export const onStatusChanged = (handler: (status: Status) => void) =>
  listen<Status>("status-changed", (e) => handler(e.payload));
