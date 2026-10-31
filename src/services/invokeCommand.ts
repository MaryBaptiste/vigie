import { invoke, type InvokeArgs } from "@tauri-apps/api/core";
import type { AppError, AppErrorKind } from "../types/appError";

const KNOWN_KINDS: readonly AppErrorKind[] = [
  "network",
  "invalidResponse",
  "notFound",
  "propagation",
  "invalidInput",
  "storage",
  "catalog",
];

function isRecord(value: unknown): value is Record<string, unknown> {
  return typeof value === "object" && value !== null;
}

/** Normalise toute erreur levée par `invoke()` en `AppError`. */
export function toAppError(error: unknown): AppError {
  if (isRecord(error) && typeof error.message === "string") {
    const kind = KNOWN_KINDS.find((k) => k === error.kind) ?? "unknown";
    return { kind, message: error.message };
  }
  return { kind: "unknown", message: String(error) };
}

/** Seul point d'appel à `invoke()` de l'application. Rejette toujours un `AppError`. */
export async function invokeCommand<T>(command: string, args?: InvokeArgs): Promise<T> {
  try {
    return await invoke<T>(command, args);
  } catch (error) {
    throw toAppError(error);
  }
}
