use std::{
    fmt::{Debug, Display},
    rc::Rc,
};

use serde::{Deserialize, Serialize};

/// Name of the Task
#[derive(Clone, Hash, PartialEq, Eq)]
pub struct TaskName(Rc<String>);

impl<'de> Deserialize<'de> for TaskName {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        Deserialize::deserialize(deserializer).map(TaskName)
    }
}

impl Serialize for TaskName {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        self.0.serialize(serializer)
    }
}

impl Display for TaskName {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        Display::fmt(&self.0, f)
    }
}

impl Debug for TaskName {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        Debug::fmt(&self.0, f)
    }
}

impl From<String> for TaskName {
    fn from(value: String) -> Self {
        Self(Rc::new(value))
    }
}

impl AsRef<String> for TaskName {
    fn as_ref(&self) -> &String {
        &self.0
    }
}
