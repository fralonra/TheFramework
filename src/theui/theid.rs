pub use crate::prelude::*;

/// Defines the identifier for a widget, its name and Uuid.
#[derive(Clone, Debug)]
pub struct TheId {
    pub name: String,
    pub uuid: Uuid,
}

impl TheId {
    pub fn new(name: String) -> Self {
        Self {
            name,
            uuid: Uuid::new_v4(),
        }
    }

    /// Matches the id against optional names and uuids.
    pub fn matches(&self, name: Option<&String>, uuid: Option<&Uuid>) -> bool {
        if name.is_none() && uuid.is_none() {
            return false;
        }

        name == Some(&self.name) || uuid == Some(&self.uuid)
    }

    /// Checks if the ids are equal (reference the same widget).
    pub fn equals(&self, other: &Option<TheId>) -> bool {
        if let Some(other) = other {
            if self.uuid == other.uuid {
                return true;
            }
        }
        false
    }
}