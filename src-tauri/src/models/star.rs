use crate::astronomy::stars::CatalogPosition;

/// Étoile du Yale Bright Star Catalog.
#[derive(Debug, Clone, PartialEq)]
pub struct Star {
    /// Numéro HR du catalogue.
    pub id: u32,
    /// Nom propre IAU (WGSN), francisé pour les plus connues.
    pub name: Option<String>,
    pub position: CatalogPosition,
    /// Magnitude visuelle V.
    pub magnitude: f64,
    /// Indice de couleur B−V.
    pub color_index: Option<f64>,
}
