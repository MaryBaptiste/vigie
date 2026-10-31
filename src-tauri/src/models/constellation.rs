use crate::astronomy::Equatorial;

/// Segment de tracé entre deux étoiles (J2000).
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct ConstellationSegment {
    /// Identifiant unique sur tout le catalogue, stable d'un calcul à l'autre.
    pub id: u32,
    pub from: Equatorial,
    pub to: Equatorial,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Constellation {
    /// Abréviation IAU (« UMa »).
    pub abbreviation: String,
    pub latin_name: String,
    pub french_name: String,
    pub segments: Vec<ConstellationSegment>,
}
