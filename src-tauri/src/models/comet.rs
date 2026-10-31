use chrono::{DateTime, Utc};

use crate::astronomy::comet::OrbitalElements;

/// Comète du catalogue MPC.
#[derive(Debug, Clone, PartialEq)]
pub struct Comet {
    /// Désignation (« C/2023 A3 », « 2P »), utilisée comme identifiant.
    pub designation: String,
    /// Désignation et nom (« C/2023 A3 (Tsuchinshan-ATLAS) »).
    pub name: String,
    pub elements: OrbitalElements,
    /// Paramètres photométriques : m = H + 5·log Δ + K·log r.
    pub absolute_magnitude: f64,
    pub slope_k: f64,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CometDataSource {
    /// Téléchargé pendant cette session.
    Network,
    /// Chargé depuis le cache disque, sans échec réseau.
    Cache,
    /// Le réseau a échoué : cache éventuel réutilisé.
    Offline,
    /// Premier téléchargement en cours, aucun cache.
    Loading,
}

#[derive(Debug, Clone, PartialEq)]
pub struct CometCatalogStatus {
    pub updated_at: Option<DateTime<Utc>>,
    pub source: CometDataSource,
    pub comet_count: usize,
}
