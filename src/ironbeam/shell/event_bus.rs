//! IronBeam Event Bus
//! 
//! Handles cross-component messaging and event distribution.

use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use crate::cell::CellId;

#[derive(Clone, Debug)]
pub enum IronBeamEvent {
    // Window Management
    WindowFocus(CellId),
    WindowClose(CellId),
    
    // Selection
    SelectionChanged(Vec<CellId>),
    
    // Staging
    ModelAdded(CellId),
    ModelRemoved(CellId),
    
    // Ontology
    RulesetChanged(String), // TODO: Use enum when available
    
    // IDE
    FileOpened(String),
    CodeChanged(String),
    
    // Generic
    Command(String),
}

pub type EventHandler = Box<dyn Fn(&IronBeamEvent) + Send + Sync>;

pub struct EventBus {
    subscribers: Arc<Mutex<HashMap<String, Vec<EventHandler>>>>,
}

impl EventBus {
    pub fn new() -> Self {
        Self {
            subscribers: Arc::new(Mutex::new(HashMap::new())),
        }
    }

    pub fn subscribe<F>(&self, topic: &str, handler: F)
    where
        F: Fn(&IronBeamEvent) + Send + Sync + 'static,
    {
        let mut subs = self.subscribers.lock().unwrap();
        subs.entry(topic.to_string())
            .or_insert_with(Vec::new)
            .push(Box::new(handler));
    }

    pub fn publish(&self, topic: &str, event: IronBeamEvent) {
        let subs = self.subscribers.lock().unwrap();
        if let Some(handlers) = subs.get(topic) {
            for handler in handlers {
                handler(&event);
            }
        }
    }
}
