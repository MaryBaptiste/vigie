import { Vector3 } from "three";

/** Rayon équatorial WGS84 (km), pris comme rayon unitaire du globe. */
export const EARTH_RADIUS_KM = 6378.137;

/**
 * Position géodésique → coordonnées scène (globe de rayon 1, repère lié à la Terre).
 * Convention : +Y = pôle Nord, longitude 0° vers +Z, longitudes Est vers +X.
 * Destinée au futur marqueur de l'objet sélectionné et à sa trace.
 */
export function latLonToVector3(latDeg: number, lonDeg: number, altitudeKm = 0, target = new Vector3()): Vector3 {
  const r = 1 + altitudeKm / EARTH_RADIUS_KM;
  const lat = (latDeg * Math.PI) / 180;
  const lon = (lonDeg * Math.PI) / 180;
  return target.set(
    r * Math.cos(lat) * Math.sin(lon),
    r * Math.sin(lat),
    r * Math.cos(lat) * Math.cos(lon),
  );
}

/** Inverse de `latLonToVector3` (le rayon est ignoré). */
export function vector3ToLatLon(point: Vector3): [latDeg: number, lonDeg: number] {
  const p = point.clone().normalize();
  return [(Math.asin(p.y) * 180) / Math.PI, (Math.atan2(p.x, p.z) * 180) / Math.PI];
}
