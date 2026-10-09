use chrono::{DateTime, Datelike, NaiveDate, NaiveDateTime, NaiveTime, Utc};

/// Result of solar calculation for a specific day
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SolarTimes {
    Normal {
        sunrise_utc: DateTime<Utc>,
        sunset_utc: DateTime<Utc>,
    },
    PolarDay,   // Sun never sets
    PolarNight, // Sun never rises
}

/// Solar calculator based on NOAA Solar Calculation algorithm
pub struct SolarCalculator {
    pub latitude: f64,
    pub longitude: f64,
    /// Zenith angle in degrees:
    /// 90.8333° = Official sunrise/sunset (accounting for atmospheric refraction and solar disc diameter)
    /// 96.0° = Civil twilight (dawn / dusk)
    pub zenith: f64,
    pub sunrise_offset_minutes: i64,
    pub sunset_offset_minutes: i64,
}

impl SolarCalculator {
    pub const ZENITH_OFFICIAL: f64 = 90.83333333333333;
    pub const ZENITH_CIVIL: f64 = 96.0;

    pub fn new(latitude: f64, longitude: f64) -> Self {
        Self {
            latitude,
            longitude,
            zenith: Self::ZENITH_OFFICIAL,
            sunrise_offset_minutes: 0,
            sunset_offset_minutes: 0,
        }
    }

    pub fn with_zenith(mut self, zenith: f64) -> Self {
        self.zenith = zenith;
        self
    }

    pub fn with_offsets(mut self, sunrise_offset: i64, sunset_offset: i64) -> Self {
        self.sunrise_offset_minutes = sunrise_offset;
        self.sunset_offset_minutes = sunset_offset;
        self
    }

    /// Calculate sunrise and sunset for a given UTC date
    pub fn calculate(&self, date: NaiveDate) -> SolarTimes {
        let day_of_year = date.ordinal() as f64;
        let lng_hour = self.longitude / 15.0;

        // 1. Calculate sunrise
        let sunrise_hour = self.calc_event_hour(day_of_year, lng_hour, true);
        // 2. Calculate sunset
        let sunset_hour = self.calc_event_hour(day_of_year, lng_hour, false);

        match (sunrise_hour, sunset_hour) {
            (Some(sr), Some(ss)) => {
                let sr_time = self.hour_to_datetime(date, sr);
                let ss_time = self.hour_to_datetime(date, ss);

                let sr_adjusted = sr_time + chrono::Duration::minutes(self.sunrise_offset_minutes);
                let ss_adjusted = ss_time + chrono::Duration::minutes(self.sunset_offset_minutes);

                SolarTimes::Normal {
                    sunrise_utc: sr_adjusted,
                    sunset_utc: ss_adjusted,
                }
            }
            _ => {
                // If polar condition: check solar declination to determine polar day or night
                let decl = self.approx_declination(day_of_year);
                let lat_rad = self.latitude.to_radians();
                let sin_alt = lat_rad.sin() * decl.sin();
                if sin_alt > 0.0 {
                    SolarTimes::PolarDay
                } else {
                    SolarTimes::PolarNight
                }
            }
        }
    }

    fn approx_declination(&self, day_of_year: f64) -> f64 {
        let t = day_of_year + ((12.0 - (self.longitude / 15.0)) / 24.0);
        let m = ((0.9856 * t) - 3.289).to_radians();
        let l = (m.to_degrees()
            + (1.916 * m.sin())
            + (0.020 * (2.0 * m).sin())
            + 282.634)
            .rem_euclid(360.0)
            .to_radians();
        let sin_dec = 0.39782 * l.sin();
        sin_dec.asin()
    }

    fn calc_event_hour(&self, day_of_year: f64, lng_hour: f64, is_sunrise: bool) -> Option<f64> {
        let t = if is_sunrise {
            day_of_year + ((6.0 - lng_hour) / 24.0)
        } else {
            day_of_year + ((18.0 - lng_hour) / 24.0)
        };

        // Sun's mean anomaly
        let m = ((0.9856 * t) - 3.289).to_radians();

        // Sun's true longitude
        let l = (m.to_degrees()
            + (1.916 * m.sin())
            + (0.020 * (2.0 * m).sin())
            + 282.634)
            .rem_euclid(360.0);
        let l_rad = l.to_radians();

        // Right ascension
        let mut ra = (0.91764 * l_rad.tan()).atan().to_degrees();
        ra = ra.rem_euclid(360.0);

        // Adjust RA quadrant to match L
        let l_quadrant = (l / 90.0).floor() * 90.0;
        let ra_quadrant = (ra / 90.0).floor() * 90.0;
        ra += l_quadrant - ra_quadrant;
        ra /= 15.0; // hours

        // Sun's declination
        let sin_dec = 0.39782 * l_rad.sin();
        let cos_dec = sin_dec.asin().cos();

        // Local hour angle
        let cos_h = (self.zenith.to_radians().cos() - (sin_dec * self.latitude.to_radians().sin()))
            / (cos_dec * self.latitude.to_radians().cos());

        // Check if sun rises or sets
        if cos_h > 1.0 || cos_h < -1.0 {
            return None;
        }

        let h = if is_sunrise {
            360.0 - cos_h.acos().to_degrees()
        } else {
            cos_h.acos().to_degrees()
        } / 15.0;

        // Local mean time
        let t_mean = h + ra - (0.06571 * t) - 6.622;

        // Universal time (UTC)
        let ut = (t_mean - lng_hour).rem_euclid(24.0);
        Some(ut)
    }

    fn hour_to_datetime(&self, date: NaiveDate, hour_float: f64) -> DateTime<Utc> {
        let total_seconds = (hour_float * 3600.0).round() as u32;
        let clamped_secs = total_seconds.min(86399);
        let hour = clamped_secs / 3600;
        let minute = (clamped_secs % 3600) / 60;
        let second = clamped_secs % 60;

        let naive_time = NaiveTime::from_hms_opt(hour, minute, second).unwrap_or_default();
        let naive_dt = NaiveDateTime::new(date, naive_time);
        DateTime::from_naive_utc_and_offset(naive_dt, Utc)
    }

    /// Determines if the given UTC timestamp should be Light (Day) or Dark (Night)
    pub fn is_daytime(&self, now_utc: DateTime<Utc>) -> bool {
        let date = now_utc.date_naive();
        match self.calculate(date) {
            SolarTimes::Normal {
                sunrise_utc,
                sunset_utc,
            } => {
                if sunrise_utc <= sunset_utc {
                    now_utc >= sunrise_utc && now_utc < sunset_utc
                } else {
                    now_utc >= sunrise_utc || now_utc < sunset_utc
                }
            }
            SolarTimes::PolarDay => true,
            SolarTimes::PolarNight => false,
        }
    }

    /// Get the next transition time and the target state (true = Light, false = Dark)
    pub fn next_transition(&self, now_utc: DateTime<Utc>) -> (DateTime<Utc>, bool) {
        let date = now_utc.date_naive();
        let is_day = self.is_daytime(now_utc);
        let tomorrow = date + chrono::Duration::days(1);

        if is_day {
            // Next event is sunset
            if let SolarTimes::Normal { sunset_utc, .. } = self.calculate(date) {
                if sunset_utc > now_utc {
                    return (sunset_utc, false);
                }
            }
            // Check tomorrow's sunset if today's already passed
            if let SolarTimes::Normal { sunset_utc, .. } = self.calculate(tomorrow) {
                return (sunset_utc, false);
            }
        } else {
            // Next event is sunrise
            if let SolarTimes::Normal { sunrise_utc, .. } = self.calculate(date) {
                if sunrise_utc > now_utc {
                    return (sunrise_utc, true);
                }
            }
            // Check tomorrow's sunrise
            if let SolarTimes::Normal { sunrise_utc, .. } = self.calculate(tomorrow) {
                return (sunrise_utc, true);
            }
        }

        // Fallback for polar days: check in 1 hour
        (now_utc + chrono::Duration::hours(1), is_day)
    }
}
