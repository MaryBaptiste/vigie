import type { BodyType } from "../types/sky";

const PLANET_SYMBOLS: Readonly<Record<string, string>> = {
  mercury: "☿",
  venus: "♀",
  mars: "♂",
  jupiter: "♃",
  saturn: "♄",
  uranus: "⛢",
  neptune: "♆",
};

/** Symbole astronomique d'un objet du système solaire. */
export function bodySymbol(id: string, bodyType: BodyType): string {
  switch (bodyType) {
    case "SUN":
      return "☉";
    case "MOON":
      return "☾";
    case "COMET":
      return "☄";
    case "PLANET":
      return PLANET_SYMBOLS[id] ?? "●";
  }
}

const TYPE_LABELS: Readonly<Record<BodyType, string>> = {
  SUN: "STAR",
  MOON: "SATELLITE",
  PLANET: "PLANET",
  COMET: "COMET",
};

export function bodyTypeLabel(bodyType: BodyType): string {
  return TYPE_LABELS[bodyType];
}

/** Libellé court d'une comète : sa désignation, sans le nom entre parenthèses. */
export function shortBodyName(name: string): string {
  return name.split(" (")[0] ?? name;
}
