use crate::dto::SpaceObjectDto;
use crate::models::SpaceObject;

pub struct SpaceObjectMapper;

impl SpaceObjectMapper {
    pub fn to_dto(object: &SpaceObject) -> SpaceObjectDto {
        SpaceObjectDto {
            norad_id: object.norad_id,
            name: object.name.to_owned(),
            short_name: object.short_name.to_owned(),
            description: object.description.to_owned(),
        }
    }

    pub fn to_dtos(objects: &[SpaceObject]) -> Vec<SpaceObjectDto> {
        objects.iter().map(Self::to_dto).collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const ISS: SpaceObject = SpaceObject {
        norad_id: 25544,
        name: "ISS (ZARYA)",
        short_name: "ISS",
        description: "International Space Station",
    };

    #[test]
    fn maps_every_field() {
        let dto = SpaceObjectMapper::to_dto(&ISS);
        assert_eq!(dto.norad_id, 25544);
        assert_eq!(dto.name, "ISS (ZARYA)");
        assert_eq!(dto.short_name, "ISS");
        assert_eq!(dto.description, "International Space Station");
    }

    #[test]
    fn serializes_in_camel_case() {
        let json = serde_json::to_value(SpaceObjectMapper::to_dto(&ISS)).unwrap();
        assert_eq!(json["noradId"], 25544);
        assert_eq!(json["shortName"], "ISS");
    }

    #[test]
    fn preserves_order() {
        let other = SpaceObject { norad_id: 1, ..ISS };
        let dtos = SpaceObjectMapper::to_dtos(&[ISS, other]);
        let ids: Vec<u32> = dtos.iter().map(|d| d.norad_id).collect();
        assert_eq!(ids, [25544, 1]);
    }
}
