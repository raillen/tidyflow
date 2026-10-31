/**
 * Extracts a human-readable message from anything the Tauri IPC layer throws.
 *
 * `invoke` rejects with a plain `string` for Rust `Result::Err(String)` and for
 * `#[tauri::command]` errors, so `error instanceof Error` is false for the most
 * common failure path. Reading `.message` first and falling back to the raw
 * value keeps the backend's wording instead of a generic label.
 *
 * @param error  The rejection value, of unknown shape.
 * @param fallback  Text used when the value carries no usable message.
 */
export function errorMessage(error: unknown, fallback: string): string {
  if (typeof error === "string" && error.trim()) {
    return error;
  }
  if (error instanceof Error && error.message.trim()) {
    return error.message;
  }
  if (error && typeof error === "object") {
    const message = (error as { message?: unknown }).message;
    if (typeof message === "string" && message.trim()) {
      return message;
    }
  }
  return fallback;
}
