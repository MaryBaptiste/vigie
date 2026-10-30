/// Objet suivi par Vigie (catalogue interne).
#[derive(Debug, Clone, PartialEq)]
pub struct SpaceObject {
    pub norad_id: u32,
    pub name: &'static str,
    pub short_name: &'static str,
    pub description: &'static str,
}
