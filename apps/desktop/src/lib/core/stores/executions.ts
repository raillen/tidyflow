import { atom, map } from "nanostores";
import type { ActiveExecution, ExecutionCompleted } from "$lib/contracts/job";

export const activeExecutions = map<Record<string, ActiveExecution>>({});
export const lastCompleted = atom<ExecutionCompleted | null>(null);

export function upsertExecution(progress: ActiveExecution): void {
  activeExecutions.setKey(progress.executionId, progress);
}

export function removeExecution(executionId: string): void {
  const next = { ...activeExecutions.get() };
  delete next[executionId];
  activeExecutions.set(next);
}

export function setActiveExecutions(items: ActiveExecution[]): void {
  const next: Record<string, ActiveExecution> = {};
  for (const item of items) {
    next[item.executionId] = item;
  }
  activeExecutions.set(next);
}

export function handleCompleted(completed: ExecutionCompleted): void {
  removeExecution(completed.executionId);
  lastCompleted.set(completed);
}
