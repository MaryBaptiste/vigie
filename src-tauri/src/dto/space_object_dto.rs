use serde::Serialize;

#[derive(Debug, Clone, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SpaceObjectDto {
    pub norad_id: u32,
    pub name: String,
    pub short_name: String,
    pub description: String,
}
