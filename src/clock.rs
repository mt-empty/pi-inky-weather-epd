//! Clock abstraction for time-dependent operations
//!
//! This module provides a trait-based abstraction for accessing the current time,
//! which allows for dependency injection and testing of time-dependent logic.

use chrono::{DateTime, Local, Utc};

/// Trait for accessing the current time
///
/// This trait allows for dependency injection of time sources, making it possible
/// to test time-dependent logic with fixed or controlled time values.
///
/// # Examples
///
/// Using the system clock in production:
///
/// ```ignore
/// let clock = SystemClock;
/// let now = clock.now_local();
/// ```
///
/// Using a fixed clock in tests:
///
/// ```ignore
/// let clock = FixedClock::new(Utc.with_ymd_and_hms(2025, 10, 9, 22, 0, 0).unwrap());
/// let now = clock.now_local();  // Always returns 10 PM Melbourne time
/// ```
pub trait Clock {
    /// Returns the current local time
    fn now_local(&self) -> DateTime<Local>;

    /// Returns the current UTC time
    fn now_utc(&self) -> DateTime<Utc>;
}

/// System clock implementation that returns actual current time
///
/// This is the production implementation that should be used in normal application flow.
#[derive(Debug, Clone, Copy)]
pub struct SystemClock;

impl Clock for SystemClock {
    fn now_local(&self) -> DateTime<Local> {
        Local::now()
    }

    fn now_utc(&self) -> DateTime<Utc> {
        Utc::now()
    }
}

/// Fixed clock implementation for testing
///
/// Returns a predetermined time, useful for testing time-dependent logic.
#[derive(Debug, Clone, Copy)]
pub struct FixedClock {
    fixed_time: DateTime<Utc>,
}

impl FixedClock {
    /// Create a new fixed clock with the specified UTC time
    ///
    /// # Arguments
    ///
    /// * `fixed_time` - The UTC time to always return
    ///
    /// # Examples
    ///
    /// ```ignore
    /// use chrono::{TimeZone, Utc};
    ///
    /// // Create a clock fixed at 10 PM UTC on Oct 9, 2025
    /// let clock = FixedClock::new(
    ///     Utc.with_ymd_and_hms(2025, 10, 9, 22, 0, 0).unwrap()
    /// );
    /// ```
    pub fn new(fixed_time: DateTime<Utc>) -> Self {
        Self { fixed_time }
    }

    /// Create a fixed clock from an RFC3339 timestamp string
    ///
    /// # Arguments
    ///
    /// * `timestamp` - RFC3339 formatted timestamp (e.g., "2025-10-09T22:00:00Z")
    ///
    /// # Returns
    ///
    /// * `Ok(FixedClock)` if the timestamp is valid
    /// * `Err` if the timestamp cannot be parsed
    ///
    /// # Examples
    ///
    /// ```ignore
    /// let clock = FixedClock::from_rfc3339("2025-10-09T22:00:00Z").unwrap();
    /// ```
    pub fn from_rfc3339(timestamp: &str) -> Result<Self, chrono::ParseError> {
        let fixed_time = DateTime::parse_from_rfc3339(timestamp)?
            .with_timezone(&Utc);
        Ok(Self { fixed_time })
    }
}

impl Clock for FixedClock {
    fn now_local(&self) -> DateTime<Local> {
        self.fixed_time.with_timezone(&Local)
    }

    fn now_utc(&self) -> DateTime<Utc> {
        self.fixed_time
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::{Datelike, TimeZone, Timelike};

    #[test]
    fn test_system_clock_returns_real_time() {
        let clock = SystemClock;
        let now_local = clock.now_local();
        let now_utc = clock.now_utc();

        // Just verify they return something (can't test exact values)
        assert!(now_local.year() >= 2025);
        assert!(now_utc.year() >= 2025);
    }

    #[test]
    fn test_fixed_clock_returns_fixed_time() {
        let fixed_time = Utc.with_ymd_and_hms(2025, 10, 9, 22, 0, 0).unwrap();
        let clock = FixedClock::new(fixed_time);

        let now_utc = clock.now_utc();
        assert_eq!(now_utc, fixed_time);

        let now_local = clock.now_local();
        assert_eq!(now_local.year(), 2025);
        assert_eq!(now_local.month(), 10);
        // Day could be 9 or 10 depending on timezone (e.g., AEDT is UTC+11)
        assert!(now_local.day() == 9 || now_local.day() == 10);
        // Verify the times represent the same instant
        assert_eq!(now_local.with_timezone(&Utc), fixed_time);
    }

    #[test]
    fn test_fixed_clock_from_rfc3339() {
        let clock = FixedClock::from_rfc3339("2025-10-09T22:00:00Z").unwrap();
        let now_utc = clock.now_utc();

        assert_eq!(now_utc.year(), 2025);
        assert_eq!(now_utc.month(), 10);
        assert_eq!(now_utc.day(), 9);
        assert_eq!(now_utc.hour(), 22);
        assert_eq!(now_utc.minute(), 0);
        assert_eq!(now_utc.second(), 0);
    }

    #[test]
    fn test_fixed_clock_multiple_calls_return_same_time() {
        let clock = FixedClock::from_rfc3339("2025-10-09T14:30:00Z").unwrap();

        let time1 = clock.now_utc();
        let time2 = clock.now_utc();
        let time3 = clock.now_utc();

        assert_eq!(time1, time2);
        assert_eq!(time2, time3);
    }
}
