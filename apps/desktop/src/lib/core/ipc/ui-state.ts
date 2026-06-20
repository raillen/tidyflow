import { invoke } from "@tauri-apps/api/core";

export type ModalUiState = {
  width: number;
  height: number;
  navCollapsed?: boolean;
};

const MODAL_DEFAULT: ModalUiState = {
  width: Math.round(window.innerWidth * 0.78),
  height: Math.round(window.innerHeight * 0.82),
  navCollapsed: false,
};

export async function loadModalState(key: string): Promise<ModalUiState> {
  try {
    const raw: unknown = await invoke("ui_state_get", { key });
    if (raw && typeof raw === "object") {
      const state = raw as Partial<ModalUiState>;
      return {
        width: state.width ?? MODAL_DEFAULT.width,
        height: state.height ?? MODAL_DEFAULT.height,
        navCollapsed: state.navCollapsed ?? false,
      };
    }
  } catch {
    /* browser preview */
  }
  return { ...MODAL_DEFAULT };
}

export async function saveModalState(key: string, state: ModalUiState): Promise<void> {
  try {
    await invoke("ui_state_save", { key, payload: state });
  } catch {
    /* ignore */
  }
}

export type MissedScheduleEntry = {
  id: number;
  jobId: string;
  jobName: string;
  scheduledAt: string;
  detectedAt: string;
};

export async function listMissedSchedules(): Promise<MissedScheduleEntry[]> {
  const raw: unknown = await invoke("jobs_list_missed_schedules");
  return raw as MissedScheduleEntry[];
}

export async function clearMissedSchedules(): Promise<void> {
  await invoke("jobs_clear_missed_schedules");
}
