use chrono::{NaiveDate, NaiveTime, Timelike};

#[derive(Debug, Clone)]
pub struct MoonInfo {
    pub phase_name: &'static str,
    pub emoji: &'static str,
    pub illumination_pct: u8,
    pub age_days: f64,
}

pub struct MoonCalculator;

impl MoonCalculator {
    const SYNODIC_MONTH: f64 = 29.530588853; // Average lunar cycle in days

    /// Calculate lunar phase for a given date
    pub fn calculate_phase(date: NaiveDate) -> MoonInfo {
        // Reference known New Moon: 2000-01-06 18:14:00 UTC
        let ref_date = NaiveDate::from_ymd_opt(2000, 1, 6).unwrap();
        let ref_time = NaiveTime::from_hms_opt(18, 14, 0).unwrap();
        let ref_dt = ref_date.and_time(ref_time).and_utc();

        let target_dt = date.and_time(NaiveTime::from_hms_opt(12, 0, 0).unwrap()).and_utc();
        let diff_seconds = (target_dt - ref_dt).num_seconds() as f64;
        let days_since_ref = diff_seconds / 86400.0;

        let cycles = days_since_ref / Self::SYNODIC_MONTH;
        let mut age_days = (cycles.fract() * Self::SYNODIC_MONTH).rem_euclid(Self::SYNODIC_MONTH);
        if age_days < 0.0 {
            age_days += Self::SYNODIC_MONTH;
        }

        // Illumination fraction formula: (1 - cos(angle)) / 2
        let phase_angle = (age_days / Self::SYNODIC_MONTH) * 2.0 * std::f64::consts::PI;
        let illumination = (((1.0 - phase_angle.cos()) / 2.0) * 100.0).round() as u8;

        let (phase_name, emoji) = match age_days {
            a if a < 1.84 => ("Новолуние", "🌑"),
            a if a < 5.53 => ("Молодая луна", "🌒"),
            a if a < 9.23 => ("1-я четверть", "🌓"),
            a if a < 12.92 => ("Растущая луна", "🌔"),
            a if a < 16.61 => ("Полнолуние", "🌕"),
            a if a < 20.30 => ("Убывающая луна", "🌖"),
            a if a < 23.99 => ("Посл. четверть", "🌗"),
            a if a < 27.68 => ("Старая луна", "🌘"),
            _ => ("Новолуние", "🌑"),
        };

        MoonInfo {
            phase_name,
            emoji,
            illumination_pct: illumination,
            age_days,
        }
    }

    /// Calculate approximate Moonset time for a given date, sunset time, and coordinates
    pub fn calculate_moonset(date: NaiveDate, sunset_local: NaiveTime) -> NaiveTime {
        let phase = Self::calculate_phase(date);

        // At New Moon (age 0), Moon sets at sunset.
        // As the Moon ages, it lags the Sun by ~24 hours over the 29.53-day cycle (~49 minutes/day).
        let offset_hours = (phase.age_days / Self::SYNODIC_MONTH) * 24.0;
        let sunset_secs = sunset_local.num_seconds_from_midnight() as f64;

        let moonset_secs = (sunset_secs + (offset_hours * 3600.0)).rem_euclid(86400.0) as u32;

        let h = (moonset_secs / 3600) % 24;
        let m = (moonset_secs % 3600) / 60;
        let s = moonset_secs % 60;

        NaiveTime::from_hms_opt(h, m, s).unwrap_or(sunset_local)
    }
}
