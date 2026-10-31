import type { CelestialBody, CometCatalogStatus } from "../types/celestialBody";
import { invokeCommand } from "./invokeCommand";

export function getVisibleBodies(datetimeUtc?: string): Promise<CelestialBody[]> {
  return invokeCommand<CelestialBody[]>("get_visible_bodies", { datetimeUtc });
}

export function getCometCatalogStatus(): Promise<CometCatalogStatus> {
  return invokeCommand<CometCatalogStatus>("get_comet_catalog_status");
}
