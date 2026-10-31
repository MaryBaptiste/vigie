/** Colonne de `TuiTable`. */
export interface TuiColumn<T> {
  key: string;
  label: string;
  /** Largeur CSS, ex. `8ch` ; une colonne sans largeur prend l'espace restant. */
  width?: string;
  align?: "left" | "right" | "center";
  /** Valeur de tri ; colonne non triable si absente. */
  sortValue?: (row: T) => number | string;
  /** Texte de la cellule (sinon slot `cell-<key>`). */
  text?: (row: T) => string;
}

export interface TuiSort {
  key: string;
  direction: "asc" | "desc";
}
