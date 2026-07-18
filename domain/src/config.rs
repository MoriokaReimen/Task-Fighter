use serde::{Deserialize, Serialize};
use std::convert::TryFrom;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize, Serialize, Default)]
pub enum Locale {
    #[default]
    System,
    English,
    Japanese,

    German,
    Chinese,
    Vietnamese,
    Spanish,
}

impl From<Locale> for i32 {
    fn from(scheme: Locale) -> Self {
        match scheme {
            Locale::System => 0,
            Locale::English => 1,
            Locale::Japanese => 2,
            Locale::German => 3,
            Locale::Chinese => 4,
            Locale::Vietnamese => 5,
            Locale::Spanish => 6,
        }
    }
}

impl TryFrom<i32> for Locale {
    type Error = String;

    fn try_from(value: i32) -> Result<Self, Self::Error> {
        match value {
            0 => Ok(Self::System),
            1 => Ok(Self::English),
            2 => Ok(Self::Japanese),
            3 => Ok(Self::German),
            4 => Ok(Self::Chinese),
            5 => Ok(Self::Vietnamese),
            6 => Ok(Self::Spanish),
            _ => Err(format!("{value} Invalid value for Locale")),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize, Serialize, Default)]
pub enum ColorScheme {
    #[default]
    LightBlue,
    DarkOrange,
    WindowsLight,
    WindowsDark,
    Sakura,
    Violet,
    Chrome,
}

impl From<ColorScheme> for i32 {
    fn from(scheme: ColorScheme) -> Self {
        match scheme {
            ColorScheme::LightBlue => 0,
            ColorScheme::DarkOrange => 1,
            ColorScheme::WindowsLight => 2,
            ColorScheme::WindowsDark => 3,
            ColorScheme::Sakura => 4,
            ColorScheme::Violet => 5,
            ColorScheme::Chrome => 6,
        }
    }
}

impl TryFrom<i32> for ColorScheme {
    type Error = String;

    fn try_from(value: i32) -> Result<Self, Self::Error> {
        match value {
            0 => Ok(Self::LightBlue),
            1 => Ok(Self::DarkOrange),
            2 => Ok(Self::WindowsLight),
            3 => Ok(Self::WindowsDark),
            4 => Ok(Self::Sakura),
            5 => Ok(Self::Violet),
            6 => Ok(Self::Chrome),
            _ => Err(format!("{value} Invalid value for ColorScheme")),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Deserialize, Serialize, Default)]
pub struct Config {
    pub color_scheme: ColorScheme,
    pub locale: Locale,
}
