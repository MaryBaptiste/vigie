/** Projection équirectangulaire : 1 unité SVG = 1 degré. */
export const MAP_WIDTH = 360;
export const MAP_HEIGHT = 180;

export interface LatLonPoint {
  latitudeDeg: number;
  longitudeDeg: number;
}

export function project(latDeg: number, lonDeg: number): [x: number, y: number] {
  return [lonDeg + 180, 90 - latDeg];
}

function format(latDeg: number, lonDeg: number): string {
  const [x, y] = project(latDeg, lonDeg);
  return `${x.toFixed(2)},${y.toFixed(2)}`;
}

/**
 * Convertit une polyligne géographique en une ou plusieurs listes de points SVG
 * (attribut `points` de <polyline>). Elle est coupée au passage de l'antiméridien,
 * avec un point interpolé sur chaque bord de la carte pour que le tracé touche les bords.
 */
export function toMapPolylines(points: readonly LatLonPoint[]): string[] {
  const polylines: string[] = [];
  let current: string[] = [];

  const flush = () => {
    if (current.length > 1) polylines.push(current.join(" "));
    current = [];
  };

  points.forEach((point, i) => {
    const previous = points[i - 1];
    if (previous) {
      const deltaLon = point.longitudeDeg - previous.longitudeDeg;
      if (Math.abs(deltaLon) > 180) {
        // Vers l'est (179° → −179°) on sort par +180°, vers l'ouest par −180°.
        const edge = deltaLon < 0 ? 180 : -180;
        const unwrappedLon = point.longitudeDeg + (deltaLon < 0 ? 360 : -360);
        const t = (edge - previous.longitudeDeg) / (unwrappedLon - previous.longitudeDeg);
        const edgeLat = previous.latitudeDeg + t * (point.latitudeDeg - previous.latitudeDeg);
        current.push(format(edgeLat, edge));
        flush();
        current.push(format(edgeLat, -edge));
      }
    }
    current.push(format(point.latitudeDeg, point.longitudeDeg));
  });
  flush();

  return polylines;
}
