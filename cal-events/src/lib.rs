use anyhow::{Result, anyhow};
use chrono::{DateTime, Local};
use std::collections::HashMap;
use uuid::Uuid;

#[derive(Debug, Clone)]
pub struct Event {
    pub id: Uuid,
    pub title: String,
    pub description: Option<String>,
    pub start_time: DateTime<Local>,
    pub end_time: DateTime<Local>,
}

impl Event {
    pub fn new(
        title: String,
        description: Option<String>,
        start_time: DateTime<Local>,
        end_time: DateTime<Local>,
    ) -> Result<Self> {
        if end_time <= start_time {
            return Err(anyhow!("End time must be after start time"));
        }

        Ok(Self {
            id: Uuid::new_v4(),
            title,
            description,
            start_time,
            end_time,
        })
    }
}

#[derive(Debug, Default)]
pub struct EventManager {
    events: HashMap<Uuid, Event>,
}

impl EventManager {
    pub fn new() -> Self {
        Self {
            events: HashMap::new(),
        }
    }

    pub fn add_event(&mut self, event: Event) -> Result<Uuid> {
        let id = event.id;
        self.events.insert(id, event);
        Ok(id)
    }

    pub fn delete_event(&mut self, id: Uuid) -> Result<()> {
        self.events.remove(&id)
            .ok_or_else(|| anyhow!("Event not found"))?;
        Ok(())
    }

    pub fn edit_event(&mut self, id: Uuid, mut updated_event: Event) -> Result<()> {
        if !self.events.contains_key(&id) {
            return Err(anyhow!("Event not found"));
        }
        updated_event.id = id; // Preserve the original ID
        self.events.insert(id, updated_event);
        Ok(())
    }

    pub fn get_event(&self, id: Uuid) -> Option<&Event> {
        self.events.get(&id)
    }

    pub fn list_events(&self) -> Vec<&Event> {
        self.events.values().collect()
    }

    pub fn list_events_for_day(&self, date: DateTime<Local>) -> Vec<&Event> {
        self.events.values()
            .filter(|event| {
                event.start_time.date_naive() == date.date_naive()
            })
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_event_creation() {
        let now = Local::now();
        let later = now + chrono::Duration::hours(1);
        
        let event = Event::new(
            "Test Event".to_string(),
            Some("Description".to_string()),
            now,
            later,
        );
        
        assert!(event.is_ok());
    }

    #[test]
    fn test_invalid_event_times() {
        let now = Local::now();
        let earlier = now - chrono::Duration::hours(1);
        
        let event = Event::new(
            "Test Event".to_string(),
            None,
            now,
            earlier,
        );
        
        assert!(event.is_err());
    }

    #[test]
    fn test_event_management() {
        let mut manager = EventManager::new();
        let now = Local::now();
        let later = now + chrono::Duration::hours(1);
        
        let event = Event::new(
            "Test Event".to_string(),
            None,
            now,
            later,
        ).unwrap();
        
        let id = manager.add_event(event.clone()).unwrap();
        assert_eq!(manager.list_events().len(), 1);
        
        manager.delete_event(id).unwrap();
        assert_eq!(manager.list_events().len(), 0);
    }
}