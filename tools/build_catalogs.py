"""Génère les données embarquées du planétarium de Vigie à partir des fichiers de tools/raw/.

Usage :
    python -I tools/fetch_sources.py
    python -I tools/build_catalogs.py

Sorties :
    src-tauri/data/stars.json            étoiles BSC5 jusqu'à V = 6.0, avec nom propre IAU
    src-tauri/data/constellations.json   noms (latin, français, IAU) et tracés
    src-tauri/data/boundaries.json       limites IAU (Roman 1987), équinoxe B1875
    src-tauri/src/astronomy/vsop87_data.rs  séries VSOP87D tronquées
"""

from __future__ import annotations

import csv
import gzip
import json
import math
import pathlib
import sys

TOOLS_DIR = pathlib.Path(__file__).resolve().parent
RAW_DIR = TOOLS_DIR / "raw"
ROOT = TOOLS_DIR.parent
DATA_DIR = ROOT / "src-tauri" / "data"
VSOP_OUT = ROOT / "src-tauri" / "src" / "astronomy" / "vsop87_data.rs"

MAX_MAGNITUDE = 6.0

# Troncature VSOP87 : on garde un terme si son amplitude, multipliée par
# τ^α sur l'intervalle utile (|τ| ≤ 0.2 millénaire, soit 1800–2200),
# dépasse le seuil. Erreur résultante < 1″ sur les longitudes héliocentriques.
VSOP_THRESHOLD = 1e-7
VSOP_TAU_MAX = 0.2
VSOP_PLANETS = (
    ("MERCURY", "mer"),
    ("VENUS", "ven"),
    ("EARTH", "ear"),
    ("MARS", "mar"),
    ("JUPITER", "jup"),
    ("SATURN", "sat"),
    ("URANUS", "ura"),
    ("NEPTUNE", "nep"),
)


def write_json(path: pathlib.Path, payload: object) -> None:
    path.write_text(json.dumps(payload, ensure_ascii=False, separators=(",", ":")), encoding="utf-8")
    print(f"{path.relative_to(ROOT)} : {path.stat().st_size // 1024} Kio")


def read_semicolon_csv(path: pathlib.Path) -> list[dict[str, str]]:
    with path.open(encoding="utf-8", newline="") as handle:
        return list(csv.DictReader(handle, delimiter=";"))


# ── Étoiles ────────────────────────────────────────────────────────────


def parse_float(text: str) -> float | None:
    text = text.strip()
    return float(text) if text else None


def load_star_names() -> dict[int, str]:
    """Nom IAU (avec diacritiques) indexé par numéro HR."""
    names: dict[int, str] = {}
    for line in (RAW_DIR / "IAU-CSN.txt").read_text(encoding="utf-8").splitlines():
        if line.startswith(("#", "$")) or not line.strip():
            continue
        designation = line[36:49].strip()
        if not designation.startswith("HR "):
            continue
        names[int(designation[3:])] = line[18:36].strip()
    return names


def build_stars() -> None:
    names = load_star_names()
    stars = []
    raw = gzip.decompress((RAW_DIR / "bsc5.dat.gz").read_bytes()).decode("latin-1")
    for line in raw.splitlines():
        line = line.ljust(197)
        vmag = parse_float(line[102:107])
        ra_h = line[75:77].strip()
        if vmag is None or vmag > MAX_MAGNITUDE or not ra_h:
            continue  # objets retirés du catalogue (novae, amas) ou trop faibles
        hr = int(line[0:4])
        ra_deg = 15 * (int(ra_h) + int(line[77:79]) / 60 + float(line[79:83]) / 3600)
        sign = -1 if line[83] == "-" else 1
        dec_deg = sign * (int(line[84:86]) + int(line[86:88]) / 60 + int(line[88:90]) / 3600)
        pm_ra = parse_float(line[148:154]) or 0.0  # μα·cos δ, ″/an
        pm_dec = parse_float(line[154:160]) or 0.0
        bv = parse_float(line[109:114])
        stars.append([hr, round(ra_deg, 5), round(dec_deg, 5), pm_ra, pm_dec, vmag, bv, names.get(hr)])
    write_json(DATA_DIR / "stars.json", stars)
    print(f"  {len(stars)} étoiles, {sum(1 for s in stars if s[7])} nommées")


# ── Constellations ─────────────────────────────────────────────────────


def build_constellations() -> None:
    names = read_semicolon_csv(TOOLS_DIR / "constellation_names.csv")
    lines: dict[str, list[list[list[float]]]] = {}
    features = json.loads((RAW_DIR / "constellations.lines.json").read_text(encoding="utf-8"))["features"]
    for feature in features:
        # Serpens apparaît deux fois (Caput et Cauda) : on fusionne.
        polylines = lines.setdefault(feature["id"], [])
        for polyline in feature["geometry"]["coordinates"]:
            polylines.append([[round(ra % 360, 4), round(dec, 4)] for ra, dec in polyline])

    constellations = []
    for row in names:
        abbr = row["abbreviation"]
        if abbr not in lines:
            raise SystemExit(f"tracé manquant pour {abbr}")
        constellations.append({"abbr": abbr, "latin": row["latin"], "french": row["french"], "lines": lines[abbr]})
    if len(constellations) != 88:
        raise SystemExit(f"88 constellations attendues, {len(constellations)} trouvées")
    write_json(DATA_DIR / "constellations.json", constellations)


def build_boundaries() -> None:
    known = {row["abbreviation"] for row in read_semicolon_csv(TOOLS_DIR / "constellation_names.csv")}
    rows = []
    for line in (RAW_DIR / "roman87.dat").read_text(encoding="utf-8").splitlines():
        if line.startswith("#") or not line.strip():
            continue
        ra_low, ra_high, dec_low, abbr = line.split()
        if abbr not in known:
            raise SystemExit(f"abréviation inconnue dans Roman 1987 : {abbr}")
        rows.append([float(ra_low), float(ra_high), float(dec_low), abbr])
    write_json(DATA_DIR / "boundaries.json", rows)


# ── VSOP87D ────────────────────────────────────────────────────────────


def parse_vsop(path: pathlib.Path) -> list[list[list[tuple[float, float, float]]]]:
    """series[variable][puissance] = [(A, B, C)], variables L, B, R."""
    series: list[list[list[tuple[float, float, float]]]] = [[], [], []]
    current: list[tuple[float, float, float]] | None = None
    for line in path.read_text(encoding="ascii").splitlines():
        if line.startswith(" VSOP87"):
            variable = int(line[41]) - 1
            power = int(line[59])
            current = []
            while len(series[variable]) <= power:
                series[variable].append([])
            series[variable][power] = current
        elif line.strip() and current is not None:
            # Format fixe (1x,4i1,i5,12i3,f15.11,2f18.11,f14.11,f20.11).
            current.append((float(line[79:97]), float(line[97:111]), float(line[111:131])))
    return series


def truncate(series: list[list[list[tuple[float, float, float]]]]) -> list[list[list[tuple[float, float, float]]]]:
    return [
        [[t for t in terms if t[0] * VSOP_TAU_MAX**power >= VSOP_THRESHOLD] for power, terms in enumerate(variable)]
        for variable in series
    ]


def evaluate(variable: list[list[tuple[float, float, float]]], tau: float) -> float:
    return sum(sum(a * math.cos(b + c * tau) for a, b, c in terms) * tau**power for power, terms in enumerate(variable))


def rust_terms(terms: list[tuple[float, float, float]]) -> str:
    return "&[" + ",".join(f"({a!r},{b!r},{c!r})" for a, b, c in terms) + "]"


def build_vsop87() -> None:
    out = [
        "// @generated par tools/build_catalogs.py. Ne pas modifier à la main.",
        "#![allow(clippy::excessive_precision, clippy::approx_constant)]",
        "// Séries VSOP87D (Bretagnon & Francou 1988, IMCCE), tronquées :",
        f"// amplitude × {VSOP_TAU_MAX}^α ≥ {VSOP_THRESHOLD}.",
        "",
        "use super::vsop87::Series;",
        "",
    ]
    total = 0
    for const_name, code in VSOP_PLANETS:
        full = parse_vsop(RAW_DIR / f"VSOP87D.{code}")
        cut = truncate(full)
        count = sum(len(t) for v in cut for t in v)
        total += count
        # Contrôle : écart entre séries complètes et tronquées, 1800–2200.
        worst = [0.0, 0.0, 0.0]
        for step in range(41):
            tau = -VSOP_TAU_MAX + step * (2 * VSOP_TAU_MAX / 40)
            for i in range(3):
                worst[i] = max(worst[i], abs(evaluate(full[i], tau) - evaluate(cut[i], tau)))
        print(
            f"  {const_name:<8} {count:5} termes, écart max L {worst[0] * 206265:.2f}″ "
            f"B {worst[1] * 206265:.2f}″ R {worst[2]:.2e} UA"
        )
        for label, variable in zip("LBR", cut):
            powers = ",\n    ".join(rust_terms(terms) for terms in variable)
            out.append(f"pub const {const_name}_{label}: Series = &[\n    {powers},\n];")
        out.append("")
    VSOP_OUT.write_text("\n".join(out), encoding="utf-8")
    print(f"{VSOP_OUT.relative_to(ROOT)} : {total} termes, {VSOP_OUT.stat().st_size // 1024} Kio")


def main() -> None:
    sys.stdout.reconfigure(encoding="utf-8")
    DATA_DIR.mkdir(parents=True, exist_ok=True)
    build_stars()
    build_constellations()
    build_boundaries()
    build_vsop87()


if __name__ == "__main__":
    main()
