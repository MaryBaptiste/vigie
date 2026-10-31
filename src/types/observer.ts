/** Miroir de `ObserverDto`. */
export interface Observer {
  latitudeDeg: number;
  /** Positive vers l'est. */
  longitudeDeg: number;
  altitudeM: number;
  label: string;
}
