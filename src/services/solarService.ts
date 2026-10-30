import type { SolarState } from "../types/solar";
import { invokeCommand } from "./invokeCommand";

export function getSolarState(): Promise<SolarState> {
  return invokeCommand<SolarState>("get_solar_state");
}
