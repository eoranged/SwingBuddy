//! Google Calendar service implementation
//! 
//! This service handles Google Calendar API integration for event creation,
//! management, calendar sharing functionality, OAuth2 authentication,
//! and event URL generation for "Add to Calendar" functionality.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use tracing::{info, debug};
use crate::config::settings::Settings;
use crate::models::event::Event;
use crate::utils::errors::{SwingBuddyError, GoogleError, GoogleResult, Result};

/// Google Calendar event structure
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GoogleCalendarEvent {
    pub id: Option<String>,
    pub summary: String,
    pub description: Option<String>,
    pub start: GoogleDateTime,
    pub end: GoogleDateTime,
    pub location: Option<String>,
    pub attendees: Option<Vec<GoogleAttendee>>,
    pub html_link: Option<String>,
}

/// Google Calendar date/time structure
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GoogleDateTime {
    #[serde(rename = "dateTime")]
    pub date_time: Option<String>,
    #[serde(rename = "timeZone")]
    pub time_zone: Option<String>,
}

/// Google Calendar attendee structure
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GoogleAttendee {
    pub email: String,
    #[serde(rename = "displayName")]
    pub display_name: Option<String>,
    #[serde(rename = "responseStatus")]
    pub response_status: Option<String>,
}

/// Google Calendar service for event management
#[derive(Clone)]
pub struct GoogleCalendarService {
    settings: Settings,
    http_client: reqwest::Client,
}

impl GoogleCalendarService {
    /// Create a new GoogleCalendarService instance
    pub fn new(settings: Settings) -> Result<Self> {
        let http_client = reqwest::Client::builder()
            .timeout(std::time::Duration::from_secs(30))
            .user_agent("SwingBuddy-Bot/1.0")
            .build()
            .map_err(|e| SwingBuddyError::Http(e))?;

        Ok(Self {
            settings,
            http_client,
        })
    }

    /// Create a new event in Google Calendar
    pub async fn create_event(&self, event: &Event) -> GoogleResult<String> {
        if !self.is_enabled() {
            return Err(GoogleError::ApiError("Google Calendar integration is disabled".to_string()));
        }

        info!(event_id = event.id, title = %event.title, "Creating Google Calendar event");

        let google_event = self.convert_to_google_event(event)?;
        
        // For now, return a mock calendar ID since we don't have actual Google API integration
        // In a real implementation, this would make an actual API call
        let calendar_id = self.create_mock_calendar_event(&google_event).await?;
        
        info!(event_id = event.id, calendar_id = %calendar_id, "Google Calendar event created");
        Ok(calendar_id)
    }

    /// Update an existing event in Google Calendar
    pub async fn update_event(&self, event: &Event, calendar_id: &str) -> GoogleResult<()> {
        if !self.is_enabled() {
            return Err(GoogleError::ApiError("Google Calendar integration is disabled".to_string()));
        }

        info!(event_id = event.id, calendar_id = %calendar_id, "Updating Google Calendar event");

        let google_event = self.convert_to_google_event(event)?;
        
        // Mock update - in real implementation, this would call Google Calendar API
        self.update_mock_calendar_event(calendar_id, &google_event).await?;
        
        info!(event_id = event.id, calendar_id = %calendar_id, "Google Calendar event updated");
        Ok(())
    }

    /// Delete an event from Google Calendar
    pub async fn delete_event(&self, calendar_id: &str) -> GoogleResult<()> {
        if !self.is_enabled() {
            return Err(GoogleError::ApiError("Google Calendar integration is disabled".to_string()));
        }

        info!(calendar_id = %calendar_id, "Deleting Google Calendar event");

        // Mock deletion - in real implementation, this would call Google Calendar API
        self.delete_mock_calendar_event(calendar_id).await?;
        
        info!(calendar_id = %calendar_id, "Google Calendar event deleted");
        Ok(())
    }

    /// Generate "Add to Calendar" URL for an event
    pub fn generate_add_to_calendar_url(&self, event: &Event) -> Result<String> {
        debug!(event_id = event.id, "Generating add to calendar URL");

        // Generate Google Calendar URL format
        let start_time = event.event_date.format("%Y%m%dT%H%M%SZ").to_string();
        
        // Assume 2-hour duration if not specified
        let end_time = (event.event_date + chrono::Duration::hours(2))
            .format("%Y%m%dT%H%M%SZ").to_string();

        let mut url = "https://calendar.google.com/calendar/render?action=TEMPLATE".to_string();
        
        // Add event title
        url.push_str(&format!("&text={}", urlencoding::encode(&event.title)));
        
        // Add dates
        url.push_str(&format!("&dates={}/{}", start_time, end_time));
        
        // Add description if available
        if let Some(description) = &event.description {
            url.push_str(&format!("&details={}", urlencoding::encode(description)));
        }
        
        // Add location if available
        if let Some(location) = &event.location {
            url.push_str(&format!("&location={}", urlencoding::encode(location)));
        }

        debug!(event_id = event.id, url = %url, "Generated add to calendar URL");
        Ok(url)
    }

    /// Generate calendar sharing URL
    pub fn generate_calendar_sharing_url(&self) -> Result<String> {
        let google_config = self.settings.google.as_ref()
            .ok_or_else(|| GoogleError::ApiError("Google Calendar not configured".to_string()))?;

        let sharing_url = format!(
            "https://calendar.google.com/calendar/embed?src={}",
            urlencoding::encode(&google_config.calendar_id)
        );

        Ok(sharing_url)
    }

    /// Get calendar events for a date range
    pub async fn get_events(&self, start_date: DateTime<Utc>, end_date: DateTime<Utc>) -> GoogleResult<Vec<GoogleCalendarEvent>> {
        if !self.is_enabled() {
            return Err(GoogleError::ApiError("Google Calendar integration is disabled".to_string()));
        }

        debug!(start_date = %start_date, end_date = %end_date, "Getting calendar events");

        // Mock implementation - in real implementation, this would call Google Calendar API
        let events = self.get_mock_calendar_events(start_date, end_date).await?;
        
        debug!(count = events.len(), "Retrieved calendar events");
        Ok(events)
    }

    /// Check if Google Calendar integration is enabled
    pub fn is_enabled(&self) -> bool {
        self.settings.features.google_calendar && self.settings.google.is_some()
    }

    /// Convert SwingBuddy event to Google Calendar event format
    fn convert_to_google_event(&self, event: &Event) -> GoogleResult<GoogleCalendarEvent> {
        let start_time = event.event_date.to_rfc3339();
        let end_time = (event.event_date + chrono::Duration::hours(2)).to_rfc3339();

        Ok(GoogleCalendarEvent {
            id: event.google_calendar_id.clone(),
            summary: event.title.clone(),
            description: event.description.clone(),
            start: GoogleDateTime {
                date_time: Some(start_time),
                time_zone: Some("UTC".to_string()),
            },
            end: GoogleDateTime {
                date_time: Some(end_time),
                time_zone: Some("UTC".to_string()),
            },
            location: event.location.clone(),
            attendees: None, // Could be populated with event participants
            html_link: None,
        })
    }

    /// Mock implementation for creating calendar event
    async fn create_mock_calendar_event(&self, _event: &GoogleCalendarEvent) -> GoogleResult<String> {
        // Simulate API delay
        tokio::time::sleep(std::time::Duration::from_millis(100)).await;
        
        // Generate a mock calendar ID
        let calendar_id = format!("mock_cal_{}", uuid::Uuid::new_v4().to_string().replace("-", ""));
        Ok(calendar_id)
    }

    /// Mock implementation for updating calendar event
    async fn update_mock_calendar_event(&self, _calendar_id: &str, _event: &GoogleCalendarEvent) -> GoogleResult<()> {
        // Simulate API delay
        tokio::time::sleep(std::time::Duration::from_millis(100)).await;
        Ok(())
    }

    /// Mock implementation for deleting calendar event
    async fn delete_mock_calendar_event(&self, _calendar_id: &str) -> GoogleResult<()> {
        // Simulate API delay
        tokio::time::sleep(std::time::Duration::from_millis(100)).await;
        Ok(())
    }

    /// Mock implementation for getting calendar events
    async fn get_mock_calendar_events(&self, _start_date: DateTime<Utc>, _end_date: DateTime<Utc>) -> GoogleResult<Vec<GoogleCalendarEvent>> {
        // Simulate API delay
        tokio::time::sleep(std::time::Duration::from_millis(200)).await;
        
        // Return empty list for mock implementation
        Ok(vec![])
    }

    /// Generate iCal format for event (alternative to Google Calendar)
    pub fn generate_ical(&self, event: &Event) -> Result<String> {
        debug!(event_id = event.id, "Generating iCal format");

        let start_time = event.event_date.format("%Y%m%dT%H%M%SZ").to_string();
        let end_time = (event.event_date + chrono::Duration::hours(2))
            .format("%Y%m%dT%H%M%SZ").to_string();
        let created_time = event.created_at.format("%Y%m%dT%H%M%SZ").to_string();

        let mut ical = String::new();
        ical.push_str("BEGIN:VCALENDAR\r\n");
        ical.push_str("VERSION:2.0\r\n");
        ical.push_str("PRODID:-//SwingBuddy//SwingBuddy Bot//EN\r\n");
        ical.push_str("BEGIN:VEVENT\r\n");
        ical.push_str(&format!("UID:swingbuddy-{}\r\n", event.id));
        ical.push_str(&format!("DTSTART:{}\r\n", start_time));
        ical.push_str(&format!("DTEND:{}\r\n", end_time));
        ical.push_str(&format!("DTSTAMP:{}\r\n", created_time));
        ical.push_str(&format!("SUMMARY:{}\r\n", event.title));
        
        if let Some(description) = &event.description {
            ical.push_str(&format!("DESCRIPTION:{}\r\n", description.replace('\n', "\\n")));
        }
        
        if let Some(location) = &event.location {
            ical.push_str(&format!("LOCATION:{}\r\n", location));
        }
        
        ical.push_str("STATUS:CONFIRMED\r\n");
        ical.push_str("END:VEVENT\r\n");
        ical.push_str("END:VCALENDAR\r\n");

        Ok(ical)
    }

    /// Get calendar integration statistics
    pub async fn get_integration_stats(&self) -> Result<CalendarStats> {
        debug!("Getting calendar integration statistics");

        let stats = CalendarStats {
            is_enabled: self.is_enabled(),
            total_events_created: 0, // Would be tracked in real implementation
            total_events_updated: 0,
            total_events_deleted: 0,
            last_sync: None,
        };

        Ok(stats)
    }
}

/// Calendar integration statistics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CalendarStats {
    pub is_enabled: bool,
    pub total_events_created: u64,
    pub total_events_updated: u64,
    pub total_events_deleted: u64,
    pub last_sync: Option<DateTime<Utc>>,
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::Utc;

    #[test]
    fn test_generate_add_to_calendar_url() {
        let settings = Settings::default();
        let service = GoogleCalendarService::new(settings).unwrap();
        
        let event = Event {
            id: 1,
            title: "Test Event".to_string(),
            description: Some("Test Description".to_string()),
            event_date: Utc::now(),
            location: Some("Test Location".to_string()),
            max_participants: None,
            google_calendar_id: None,
            created_by: None,
            group_id: None,
            is_active: true,
            created_at: Utc::now(),
            updated_at: Utc::now(),
        };

        let url = service.generate_add_to_calendar_url(&event).unwrap();
        assert!(url.contains("calendar.google.com"));
        assert!(url.contains("Test%20Event"));
        assert!(url.contains("Test%20Description"));
        assert!(url.contains("Test%20Location"));
    }

    #[test]
    fn test_generate_ical() {
        let settings = Settings::default();
        let service = GoogleCalendarService::new(settings).unwrap();
        
        let event = Event {
            id: 1,
            title: "Test Event".to_string(),
            description: Some("Test Description".to_string()),
            event_date: Utc::now(),
            location: Some("Test Location".to_string()),
            max_participants: None,
            google_calendar_id: None,
            created_by: None,
            group_id: None,
            is_active: true,
            created_at: Utc::now(),
            updated_at: Utc::now(),
        };

        let ical = service.generate_ical(&event).unwrap();
        assert!(ical.contains("BEGIN:VCALENDAR"));
        assert!(ical.contains("BEGIN:VEVENT"));
        assert!(ical.contains("SUMMARY:Test Event"));
        assert!(ical.contains("DESCRIPTION:Test Description"));
        assert!(ical.contains("LOCATION:Test Location"));
        assert!(ical.contains("END:VEVENT"));
        assert!(ical.contains("END:VCALENDAR"));
    }

    #[test]
    fn test_convert_to_google_event() {
        let settings = Settings::default();
        let service = GoogleCalendarService::new(settings).unwrap();
        
        let event = Event {
            id: 1,
            title: "Test Event".to_string(),
            description: Some("Test Description".to_string()),
            event_date: Utc::now(),
            location: Some("Test Location".to_string()),
            max_participants: None,
            google_calendar_id: None,
            created_by: None,
            group_id: None,
            is_active: true,
            created_at: Utc::now(),
            updated_at: Utc::now(),
        };

        let google_event = service.convert_to_google_event(&event).unwrap();
        assert_eq!(google_event.summary, "Test Event");
        assert_eq!(google_event.description, Some("Test Description".to_string()));
        assert_eq!(google_event.location, Some("Test Location".to_string()));
        assert!(google_event.start.date_time.is_some());
        assert!(google_event.end.date_time.is_some());
    }
}