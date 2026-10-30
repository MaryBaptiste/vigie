import type { Telemetry } from "../types/telemetry";
import { invokeCommand } from "./invokeCommand";

export function getTelemetry(noradId: number): Promise<Telemetry> {
  return invokeCommand<Telemetry>("get_telemetry", { noradId });
}
