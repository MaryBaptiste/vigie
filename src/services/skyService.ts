import type { Sky } from "../types/sky";
import { invokeCommand } from "./invokeCommand";

/** `datetimeUtc` absent : instant présent côté backend. */
export function getSky(datetimeUtc?: string): Promise<Sky> {
  return invokeCommand<Sky>("get_sky", { datetimeUtc });
}
