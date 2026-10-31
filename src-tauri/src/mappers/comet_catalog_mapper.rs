use chrono::{DateTime, TimeDelta, Utc};

use crate::dto::{CometCatalogStatusDto, CometSourceDto};
use crate::mappers::iso;
use crate::models::{CometCatalogStatus, CometDataSource};

/// Au-delà, le front affiche `DATA STALE`.
const STALE_AFTER: TimeDelta = TimeDelta::days(14);

pub struct CometCatalogMapper;

impl CometCatalogMapper {
    pub fn to_dto(status: &CometCatalogStatus, now: DateTime<Utc>) -> CometCatalogStatusDto {
        CometCatalogStatusDto {
            updated_at: status.updated_at.as_ref().map(iso),
            source: match status.source {
                CometDataSource::Network => CometSourceDto::Network,
                CometDataSource::Cache => CometSourceDto::Cache,
                CometDataSource::Offline => CometSourceDto::Offline,
                CometDataSource::Loading => CometSourceDto::Loading,
            },
            stale: status.updated_at.is_some_and(|at| now - at > STALE_AFTER),
            comet_count: status.comet_count,
        }
    }
}

#[cfg(test)]
mod tests {
    use chrono::TimeZone;

    use super::*;

    #[test]
    fn flags_stale_elements() {
        let updated = Utc.with_ymd_and_hms(2026, 10, 1, 0, 0, 0).unwrap();
        let status = CometCatalogStatus { updated_at: Some(updated), source: CometDataSource::Offline, comet_count: 900 };
        let fresh = CometCatalogMapper::to_dto(&status, updated + TimeDelta::days(3));
        assert!(!fresh.stale);
        let stale = CometCatalogMapper::to_dto(&status, updated + TimeDelta::days(15));
        assert!(stale.stale);
        let json = serde_json::to_value(stale).unwrap();
        assert_eq!(json["source"], "OFFLINE");
        assert_eq!(json["updatedAt"], "2026-10-01T00:00:00Z");
    }
}
