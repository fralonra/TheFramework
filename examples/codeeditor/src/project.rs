use crate::prelude::*;

#[derive(Serialize, Deserialize, PartialEq, Clone, Debug)]
pub struct Project {
    pub name: String,
    pub codegrid: TheCodeGrid,

    #[serde(skip)]
    pub undo_stack: TheUndoStack,
}

impl Default for Project {
    fn default() -> Self {
        Self::new()
    }
}

impl Project {
    pub fn new() -> Self {
        Self {
            name: "Untitled".to_string(),
            codegrid: TheCodeGrid::default(),
            undo_stack: TheUndoStack::default(),
        }
    }
}