import type { Observer } from "../types/observer";
import { invokeCommand } from "./invokeCommand";

export function getObserver(): Promise<Observer> {
  return invokeCommand<Observer>("get_observer");
}

export function setObserver(observer: Observer): Promise<Observer> {
  return invokeCommand<Observer>("set_observer", {
    lat: observer.latitudeDeg,
    lon: observer.longitudeDeg,
    altitudeM: observer.altitudeM,
    label: observer.label,
  });
}
