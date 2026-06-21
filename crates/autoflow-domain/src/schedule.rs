use chrono::{DateTime, Local, Utc};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct ScheduleConfig {
    pub enabled: bool,
    #[serde(default = "default_timezone")]
    pub timezone: String,
    pub rule: ScheduleRule,
}

fn default_timezone() -> String {
    "local".into()
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(tag = "kind", rename_all = "camelCase")]
pub enum ScheduleRule {
    Interval { minutes: u32 },
    Daily { hour: u8, minute: u8 },
    Weekly { days: Vec<u8>, hour: u8, minute: u8 },
}

impl ScheduleConfig {
    pub fn validate(&self) -> Result<(), crate::DomainError> {
        if !self.enabled {
            return Ok(());
        }
        match &self.rule {
            ScheduleRule::Interval { minutes } if *minutes == 0 => Err(
                crate::DomainError::Validation("schedule interval minutes must be >= 1".into()),
            ),
            ScheduleRule::Daily { hour, minute } if *hour > 23 || *minute > 59 => Err(
                crate::DomainError::Validation("invalid daily schedule time".into()),
            ),
            ScheduleRule::Weekly { days, hour, minute } => {
                if days.is_empty() || days.iter().any(|d| *d > 6) {
                    return Err(crate::DomainError::Validation(
                        "weekly schedule days must be 0-6".into(),
                    ));
                }
                if *hour > 23 || *minute > 59 {
                    return Err(crate::DomainError::Validation(
                        "invalid weekly schedule time".into(),
                    ));
                }
                Ok(())
            }
            _ => Ok(()),
        }
    }

    pub fn compute_next_run(&self, from: DateTime<Utc>) -> Option<DateTime<Utc>> {
        if !self.enabled {
            return None;
        }
        next_run(from, self)
    }
}

fn next_run(from: DateTime<Utc>, config: &ScheduleConfig) -> Option<DateTime<Utc>> {
    use chrono::{Datelike, Duration, TimeZone};

    if config.timezone == "local" {
        let local = from.with_timezone(&Local);
        return match &config.rule {
            ScheduleRule::Interval { minutes } => Some(from + Duration::minutes(*minutes as i64)),
            ScheduleRule::Daily { hour, minute } => {
                let mut day = local.date_naive();
                let mut candidate = day.and_hms_opt(*hour as u32, *minute as u32, 0)?;
                let mut dt = Local.from_local_datetime(&candidate).latest()?;
                if dt <= local {
                    day = day + Duration::days(1);
                    candidate = day.and_hms_opt(*hour as u32, *minute as u32, 0)?;
                    dt = Local.from_local_datetime(&candidate).latest()?;
                }
                Some(dt.with_timezone(&Utc))
            }
            ScheduleRule::Weekly { days, hour, minute } => {
                for offset in 0..8i64 {
                    let day = local.date_naive() + Duration::days(offset);
                    let weekday = day.weekday().num_days_from_sunday() as u8;
                    if !days.contains(&weekday) {
                        continue;
                    }
                    let candidate = day.and_hms_opt(*hour as u32, *minute as u32, 0)?;
                    if let Some(dt) = Local.from_local_datetime(&candidate).latest() {
                        if dt > local {
                            return Some(dt.with_timezone(&Utc));
                        }
                    }
                }
                None
            }
        };
    }

    let tz: chrono_tz::Tz = config.timezone.parse().unwrap_or(chrono_tz::UTC);
    let local = from.with_timezone(&tz);
    match &config.rule {
        ScheduleRule::Interval { minutes } => Some(from + Duration::minutes(*minutes as i64)),
        ScheduleRule::Daily { hour, minute } => {
            let mut day = local.date_naive();
            let mut candidate = day.and_hms_opt(*hour as u32, *minute as u32, 0)?;
            let mut dt = tz.from_local_datetime(&candidate).latest()?;
            if dt <= local {
                day = day + Duration::days(1);
                candidate = day.and_hms_opt(*hour as u32, *minute as u32, 0)?;
                dt = tz.from_local_datetime(&candidate).latest()?;
            }
            Some(dt.with_timezone(&Utc))
        }
        ScheduleRule::Weekly { days, hour, minute } => {
            for offset in 0..8i64 {
                let day = local.date_naive() + Duration::days(offset);
                let weekday = day.weekday().num_days_from_sunday() as u8;
                if !days.contains(&weekday) {
                    continue;
                }
                let candidate = day.and_hms_opt(*hour as u32, *minute as u32, 0)?;
                if let Some(dt) = tz.from_local_datetime(&candidate).latest() {
                    if dt > local {
                        return Some(dt.with_timezone(&Utc));
                    }
                }
            }
            None
        }
    }
}
