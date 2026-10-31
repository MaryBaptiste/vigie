/** Objet survolé ou épinglé sur la voûte céleste. */
export interface SkyPick {
  kind: "star" | "body";
  /** Numéro HR pour une étoile, identifiant backend pour un objet. */
  id: string;
  name: string;
  /** Nature de l'objet, ex. `PLANÈTE`, `HR 7001`. */
  subtitle: string;
  magnitude: number;
  azimuthDeg: number;
  altitudeDeg: number;
  /** Lune, Mercure et Vénus : fraction éclairée. */
  illumination: number | null;
}
