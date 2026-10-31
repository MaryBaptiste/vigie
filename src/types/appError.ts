/** Miroir de la sérialisation de `AppError` (src-tauri/src/error.rs). */
export type AppErrorKind =
  | "network"
  | "invalidResponse"
  | "notFound"
  | "propagation"
  | "invalidInput"
  | "storage"
  | "catalog"
  | "unknown";

export interface AppError {
  kind: AppErrorKind;
  message: string;
}
