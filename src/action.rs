use std::{fmt, string::ToString};

use serde::{
    Deserialize, Serialize,
    de::{self, Deserializer, Visitor},
};
use strum::Display;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Display, Deserialize)]
pub enum Action {
    Tick,
    Render,
    Resize(u16, u16),
    Suspend,
    Resume,
    Quit,
    Refresh,
    Error(String),
    Help,
}
