import type { SpaceObject } from "../types/spaceObject";
import { invokeCommand } from "./invokeCommand";

export function getSpaceObjects(): Promise<SpaceObject[]> {
  return invokeCommand<SpaceObject[]>("get_space_objects");
}
