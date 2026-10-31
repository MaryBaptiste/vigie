use crate::dto::ObserverDto;
use crate::models::Observer;

pub struct ObserverMapper;

impl ObserverMapper {
    pub fn to_dto(observer: &Observer) -> ObserverDto {
        ObserverDto {
            latitude_deg: observer.latitude_deg,
            longitude_deg: observer.longitude_deg,
            altitude_m: observer.altitude_m,
            label: observer.label.clone(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn serializes_in_camel_case() {
        let json = serde_json::to_value(ObserverMapper::to_dto(&Observer::paris())).unwrap();
        assert_eq!(json["latitudeDeg"], 48.8566);
        assert_eq!(json["longitudeDeg"], 2.3522);
        assert_eq!(json["altitudeM"], 35.0);
        assert_eq!(json["label"], "Paris");
    }
}
