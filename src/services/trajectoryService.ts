import type { Trajectory } from "../types/trajectory";
import { invokeCommand } from "./invokeCommand";

export function getTrajectory(noradId: number): Promise<Trajectory> {
  return invokeCommand<Trajectory>("get_trajectory", { noradId });
}
