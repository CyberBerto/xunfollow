use chrono::{DateTime, Timelike, Utc};
use rand::Rng;
use serde::{Deserialize, Serialize};
use sqlx::SqlitePool;

#[derive(Debug, Clone, PartialEq)]
pub enum LimitStatus {
    Ok,
    HourlyWait(i64), // minutes to wait
    SessionBreak,
    DailyStop,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RateLimiter {
    pub daily_count: u32,
    pub hourly_count: u32,
    pub session_count: u32,
    pub daily_limit: u32,
    pub hourly_limit: u32,
    pub session_limit: u32,
    pub min_delay: u32,
    pub max_delay: u32,
    #[serde(with = "chrono::serde::ts_seconds")]
    pub hour_start: DateTime<Utc>,
    #[serde(with = "chrono::serde::ts_seconds")]
    pub day_start: DateTime<Utc>,
}

impl Default for RateLimiter {
    fn default() -> Self {
        let now = Utc::now();
        RateLimiter {
            daily_count: 0,
            hourly_count: 0,
            session_count: 0,
            daily_limit: 50,
            hourly_limit: 30,
            session_limit: 25,
            min_delay: 30,
            max_delay: 60,
            hour_start: now,
            day_start: now,
        }
    }
}

impl RateLimiter {
    pub fn new(
        daily_limit: u32,
        hourly_limit: u32,
        session_limit: u32,
        min_delay: u32,
        max_delay: u32,
    ) -> Self {
        let now = Utc::now();
        RateLimiter {
            daily_count: 0,
            hourly_count: 0,
            session_count: 0,
            daily_limit,
            hourly_limit,
            session_limit,
            min_delay,
            max_delay,
            hour_start: now,
            day_start: now,
        }
    }

    pub fn check_limits(&mut self) -> LimitStatus {
        let now = Utc::now();

        // Check if it's a new day (reset at midnight UTC)
        if self.is_new_day(now) {
            self.reset_daily();
        }

        // Check if it's a new hour
        if self.is_new_hour(now) {
            self.reset_hourly();
        }

        // Check daily limit
        if self.daily_count >= self.daily_limit {
            return LimitStatus::DailyStop;
        }

        // Check hourly limit
        if self.hourly_count >= self.hourly_limit {
            let minutes_remaining = self.minutes_until_next_hour(now);
            return LimitStatus::HourlyWait(minutes_remaining);
        }

        // Check session limit
        if self.session_count >= self.session_limit {
            return LimitStatus::SessionBreak;
        }

        LimitStatus::Ok
    }

    pub fn increment(&mut self) {
        self.daily_count += 1;
        self.hourly_count += 1;
        self.session_count += 1;
    }

    pub fn reset_session(&mut self) {
        self.session_count = 0;
    }

    pub fn reset_hourly(&mut self) {
        self.hourly_count = 0;
        self.hour_start = Utc::now();
    }

    pub fn reset_daily(&mut self) {
        self.daily_count = 0;
        self.hourly_count = 0;
        self.session_count = 0;
        self.day_start = Utc::now();
    }

    pub fn get_random_delay(&self) -> std::time::Duration {
        let mut rng = rand::thread_rng();
        let delay_secs = rng.gen_range(self.min_delay..=self.max_delay);
        std::time::Duration::from_secs(delay_secs as u64)
    }

    fn is_new_day(&self, now: DateTime<Utc>) -> bool {
        now.date_naive() != self.day_start.date_naive()
    }

    fn is_new_hour(&self, now: DateTime<Utc>) -> bool {
        now.hour() != self.hour_start.hour() || self.is_new_day(now)
    }

    fn minutes_until_next_hour(&self, now: DateTime<Utc>) -> i64 {
        60 - now.minute() as i64
    }

    pub async fn save_to_database(&self, db: &SqlitePool) -> Result<(), sqlx::Error> {
        let pairs = [
            ("daily_count", self.daily_count.to_string()),
            ("hourly_count", self.hourly_count.to_string()),
            ("session_count", self.session_count.to_string()),
            ("hour_start", self.hour_start.timestamp().to_string()),
            ("day_start", self.day_start.timestamp().to_string()),
        ];

        for (key, value) in pairs {
            sqlx::query(
                "INSERT OR REPLACE INTO rate_limits (key, value) VALUES (?, ?)"
            )
            .bind(key)
            .bind(value)
            .execute(db)
            .await?;
        }

        Ok(())
    }

    pub async fn load_from_database(db: &SqlitePool, settings: &crate::commands::Settings) -> Result<Self, sqlx::Error> {
        let rows: Vec<(String, String)> = sqlx::query_as(
            "SELECT key, value FROM rate_limits"
        )
        .fetch_all(db)
        .await?;

        let mut limiter = RateLimiter::new(
            settings.daily_limit,
            settings.hourly_limit,
            settings.session_limit,
            settings.min_delay,
            settings.max_delay,
        );

        for (key, value) in rows {
            match key.as_str() {
                "daily_count" => limiter.daily_count = value.parse().unwrap_or(0),
                "hourly_count" => limiter.hourly_count = value.parse().unwrap_or(0),
                "session_count" => limiter.session_count = value.parse().unwrap_or(0),
                "hour_start" => {
                    if let Ok(ts) = value.parse::<i64>() {
                        if let Some(dt) = DateTime::from_timestamp(ts, 0) {
                            limiter.hour_start = dt;
                        }
                    }
                }
                "day_start" => {
                    if let Ok(ts) = value.parse::<i64>() {
                        if let Some(dt) = DateTime::from_timestamp(ts, 0) {
                            limiter.day_start = dt;
                        }
                    }
                }
                _ => {}
            }
        }

        // Check if we need to reset based on current time
        let now = Utc::now();
        if limiter.is_new_day(now) {
            limiter.reset_daily();
        } else if limiter.is_new_hour(now) {
            limiter.reset_hourly();
        }

        Ok(limiter)
    }
}
