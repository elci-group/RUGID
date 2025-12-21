/// Telemetry System - "The Observatory"
///
/// Structured logging and performance monitoring for RUGID.

use std::collections::{HashMap, VecDeque};
use std::time::Instant;

/// Log severity levels
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum LogLevel {
    ERROR = 0,
    WARN = 1,
    INFO = 2,
    DEBUG = 3,
    TRACE = 4,
}

impl std::fmt::Display for LogLevel {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            LogLevel::ERROR => write!(f, "ERROR"),
            LogLevel::WARN => write!(f, "WARN"),
            LogLevel::INFO => write!(f, "INFO"),
            LogLevel::DEBUG => write!(f, "DEBUG"),
            LogLevel::TRACE => write!(f, "TRACE"),
        }
    }
}

/// Event categories for filtering
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Category {
    Physics,
    Rendering,
    Assets,
    Input,
    Scripting,
    Network,
    System,
}

impl std::fmt::Display for Category {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Category::Physics => write!(f, "Physics"),
            Category::Rendering => write!(f, "Rendering"),
            Category::Assets => write!(f, "Assets"),
            Category::Input => write!(f, "Input"),
            Category::Scripting => write!(f, "Scripting"),
            Category::Network => write!(f, "Network"),
            Category::System => write!(f, "System"),
        }
    }
}

/// Metadata value types
#[derive(Debug, Clone)]
pub enum MetadataValue {
    String(String),
    Int(i64),
    Float(f64),
    Bool(bool),
}

impl std::fmt::Display for MetadataValue {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            MetadataValue::String(s) => write!(f, "{}", s),
            MetadataValue::Int(i) => write!(f, "{}", i),
            MetadataValue::Float(fl) => write!(f, "{:.2}", fl),
            MetadataValue::Bool(b) => write!(f, "{}", b),
        }
    }
}

/// A telemetry event
#[derive(Debug, Clone)]
pub struct TelemetryEvent {
    /// Microseconds since program start
    pub timestamp_us: u64,
    /// Log level
    pub level: LogLevel,
    /// Category 
    pub category: Category,
    /// Event name
    pub event: String,
    /// Structured metadata
    pub metadata: HashMap<String, MetadataValue>,
}

/// Telemetry collector with circular buffer
pub struct TelemetryCollector {
    /// Program start time (for timestamps)
    start_time: Instant,
    /// Circular buffer of events
    events: VecDeque<TelemetryEvent>,
    /// Maximum events to keep
    max_events: usize,
    /// Current log level filter (only log events at this level or higher priority)
    log_level: LogLevel,
    /// Event counts by category
    event_counts: HashMap<Category, u64>,
}

impl Default for TelemetryCollector {
    fn default() -> Self {
        Self::new()
    }
}

impl TelemetryCollector {
    pub fn new() -> Self {
        Self {
            start_time: Instant::now(),
            events: VecDeque::with_capacity(10000),
            max_events: 10000,
            log_level: LogLevel::INFO,
            event_counts: HashMap::new(),
        }
    }

    /// Set the log level filter
    pub fn set_log_level(&mut self, level: LogLevel) {
        self.log_level = level;
    }

    /// Record an event
    pub fn event(
        &mut self,
        level: LogLevel,
        category: Category,
        event: impl Into<String>,
        metadata: HashMap<String, MetadataValue>,
    ) {
        // Filter by log level
        if level > self.log_level {
            return;
        }

        let timestamp_us = self.start_time.elapsed().as_micros() as u64;

        let event = TelemetryEvent {
            timestamp_us,
            level,
            category,
            event: event.into(),
            metadata,
        };

        // Add to circular buffer
        if self.events.len() >= self.max_events {
            self.events.pop_front();
        }
        self.events.push_back(event);

        // Update category counts
        *self.event_counts.entry(category).or_insert(0) += 1;

        // Optional: Print to stdout for immediate visibility
        #[cfg(debug_assertions)]
        {
            if level <= LogLevel::WARN {
                eprintln!("[{:?}] [{}] {}", level, category, self.events.back().unwrap().event);
            }
        }
    }

    /// Convenience: log at INFO level
    pub fn info(&mut self, category: Category, event: impl Into<String>) {
        self.event(LogLevel::INFO, category, event, HashMap::new());
    }

    /// Convenience: log at WARN level
    pub fn warn(&mut self, category: Category, event: impl Into<String>) {
        self.event(LogLevel::WARN, category, event, HashMap::new());
    }

    /// Convenience: log at ERROR level
    pub fn error(&mut self, category: Category, event: impl Into<String>) {
        self.event(LogLevel::ERROR, category, event, HashMap::new());
    }

    /// Convenience: log at DEBUG level
    pub fn debug(&mut self, category: Category, event: impl Into<String>) {
        self.event(LogLevel::DEBUG, category, event, HashMap::new());
    }

    /// Get all events
    pub fn get_events(&self) -> &VecDeque<TelemetryEvent> {
        &self.events
    }

    /// Get events by category
    pub fn get_events_by_category(&self, category: Category) -> Vec<&TelemetryEvent> {
        self.events.iter().filter(|e| e.category == category).collect()
    }

    /// Get events by level
    pub fn get_events_by_level(&self, level: LogLevel) -> Vec<&TelemetryEvent> {
        self.events.iter().filter(|e| e.level == level).collect()
    }

    /// Get event count for a category
    pub fn get_event_count(&self, category: Category) -> u64 {
        *self.event_counts.get(&category).unwrap_or(&0)
    }

    /// Clear all events
    pub fn clear(&mut self) {
        self.events.clear();
        self.event_counts.clear();
    }

    /// Get the number of events currently stored
    pub fn len(&self) -> usize {
        self.events.len()
    }

    /// Check if the event buffer is empty
    pub fn is_empty(&self) -> bool {
        self.events.is_empty()
    }
}

/// Helper macro for creating metadata HashMap
#[macro_export]
macro_rules! metadata {
    ($($key:expr => $val:expr),* $(,)?) => {{
        let mut map = std::collections::HashMap::new();
        $(
            map.insert($key.to_string(), $crate::telemetry::MetadataValue::from($val));
        )*
        map
    }};
}

// Conversions for MetadataValue
impl From<String> for MetadataValue {
    fn from(s: String) -> Self {
        MetadataValue::String(s)
    }
}

impl From<&str> for MetadataValue {
    fn from(s: &str) -> Self {
        MetadataValue::String(s.to_string())
    }
}

impl From<i32> for MetadataValue {
    fn from(i: i32) -> Self {
        MetadataValue::Int(i as i64)
    }
}

impl From<i64> for MetadataValue {
    fn from(i: i64) -> Self {
        MetadataValue::Int(i)
    }
}

impl From<f32> for MetadataValue {
    fn from(f: f32) -> Self {
        MetadataValue::Float(f as f64)
    }
}

impl From<f64> for MetadataValue {
    fn from(f: f64) -> Self {
        MetadataValue::Float(f)
    }
}

impl From<bool> for MetadataValue {
    fn from(b: bool) -> Self {
        MetadataValue::Bool(b)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_event_logging() {
        let mut telemetry = TelemetryCollector::new();

        telemetry.info(Category::System, "Test event");

        assert_eq!(telemetry.len(), 1);
        let events = telemetry.get_events();
        assert_eq!(events[0].event, "Test event");
        assert_eq!(events[0].level, LogLevel::INFO);
    }

    #[test]
    fn test_log_level_filtering() {
        let mut telemetry = TelemetryCollector::new();
        telemetry.set_log_level(LogLevel::WARN);

        telemetry.debug(Category::System, "Debug message");
        telemetry.info(Category::System, "Info message");
        telemetry.warn(Category::System, "Warning message");

        // Only WARN and higher should be logged
        assert_eq!(telemetry.len(), 1);
        assert_eq!(telemetry.get_events()[0].level, LogLevel::WARN);
    }

    #[test]
    fn test_circular_buffer() {
        let mut telemetry = TelemetryCollector::new();
        telemetry.max_events = 5;

        for i in 0..10 {
            telemetry.info(Category::System, format!("Event {}", i));
        }

        // Should only keep last 5
        assert_eq!(telemetry.len(), 5);
        assert_eq!(telemetry.get_events()[0].event, "Event 5");
    }

    #[test]
    fn test_category_filtering() {
        let mut telemetry = TelemetryCollector::new();

        telemetry.info(Category::Physics, "Physics event");
        telemetry.info(Category::Rendering, "Rendering event");
        telemetry.info(Category::Physics, "Another physics event");

        let physics_events = telemetry.get_events_by_category(Category::Physics);
        assert_eq!(physics_events.len(), 2);
    }

    #[test]
    fn test_event_counts() {
        let mut telemetry = TelemetryCollector::new();

        telemetry.info(Category::Physics, "Event 1");
        telemetry.info(Category::Physics, "Event 2");
        telemetry.info(Category::Rendering, "Event 3");

        assert_eq!(telemetry.get_event_count(Category::Physics), 2);
        assert_eq!(telemetry.get_event_count(Category::Rendering), 1);
    }
}
