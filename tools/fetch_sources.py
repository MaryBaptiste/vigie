"""Télécharge les données brutes nécessaires à build_catalogs.py dans tools/raw/.

Usage : python -I tools/fetch_sources.py
Les fichiers bruts ne sont pas versionnés ; seuls les fichiers générés le sont.
"""

from __future__ import annotations

import pathlib
import urllib.request

RAW_DIR = pathlib.Path(__file__).resolve().parent / "raw"

VSOP87_BASE = "https://ftp.imcce.fr/pub/ephem/planets/vsop87/"
VSOP87_PLANETS = ("mer", "ven", "ear", "mar", "jup", "sat", "ura", "nep")

SOURCES = {
    # Yale Bright Star Catalog, 5e édition révisée (Hoffleit & Warren 1991).
    "bsc5.dat.gz": "http://tdc-www.harvard.edu/catalogs/bsc5.dat.gz",
    # Limites IAU des constellations (Roman 1987, CDS VI/42), copie astropy (BSD-3).
    "roman87.dat": "https://raw.githubusercontent.com/astropy/astropy/main/astropy/coordinates/data/constellation_data_roman87.dat",
    # Catalogue IAU des noms d'étoiles (WGSN), CC BY 4.0.
    "IAU-CSN.txt": "https://www.pas.rochester.edu/~emamajek/WGSN/IAU-CSN.txt",
    # Tracés des constellations de d3-celestial (BSD-3, Olaf Frohn).
    "constellations.lines.json": "https://raw.githubusercontent.com/ofrohn/d3-celestial/master/data/constellations.lines.json",
    **{f"VSOP87D.{p}": f"{VSOP87_BASE}VSOP87D.{p}" for p in VSOP87_PLANETS},
}


def main() -> None:
    RAW_DIR.mkdir(exist_ok=True)
    for name, url in SOURCES.items():
        target = RAW_DIR / name
        if target.exists():
            print(f"déjà présent : {name}")
            continue
        print(f"téléchargement : {url}")
        request = urllib.request.Request(url, headers={"User-Agent": "vigie-tools"})
        with urllib.request.urlopen(request, timeout=120) as response:
            target.write_bytes(response.read())


if __name__ == "__main__":
    main()
