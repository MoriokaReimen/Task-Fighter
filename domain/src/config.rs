use anyhow::{Context, anyhow};
use serde::{Deserialize, Serialize};
use std::convert::TryFrom;
use std::fmt;
use std::str::FromStr;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize, Serialize, Default)]
pub enum Locale {
    #[default]
    System,
    English,
    Japanese,
}

impl From<Locale> for i32 {
    fn from(scheme: Locale) -> Self {
        match scheme {
            Locale::System => 0,
            Locale::English => 1,
            Locale::Japanese => 2,
        }
    }
}

impl TryFrom<i32> for Locale {
    type Error = String;

    fn try_from(value: i32) -> Result<Self, Self::Error> {
        match value {
            0 => Ok(Locale::System),
            1 => Ok(Locale::English),
            2 => Ok(Locale::Japanese),
            _ => Err(format!("{value} Invalid value for Locale")),
        }
    }
}

impl Locale {
    #[must_use]
    pub const fn as_str(&self) -> &'static str {
        match self {
            Self::System => "System",
            Self::English => "English",
            Self::Japanese => "Japanese",
        }
    }
}

// 2. 標準の AsRef<str> トレイトを実装
impl AsRef<str> for Locale {
    fn as_ref(&self) -> &str {
        self.as_str()
    }
}

// 3. (オプション) 標準の Display トレイトを実装しておくと、format!("{}", scheme) などが可能に
impl fmt::Display for Locale {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.as_str())
    }
}

// 1. anyhow::Error を返す TryFrom<&str> の実装
impl<'a> TryFrom<&'a str> for Locale {
    type Error = anyhow::Error;

    fn try_from(value: &'a str) -> Result<Self, Self::Error> {
        match value {
            "System" => Ok(Self::System),
            "English" => Ok(Self::English),
            "Japanese" => Ok(Self::Japanese),
            _ => Err(anyhow!("Undefined Locale: '{value}'")),
        }
    }
}

impl FromStr for Locale {
    type Err = anyhow::Error;

    fn from_str(value: &str) -> Result<Self, Self::Err> {
        Self::try_from(value).with_context(|| format!("Failed convert string to Locale: {value}"))
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

impl ColorScheme {
    #[must_use]
    pub const fn as_str(&self) -> &'static str {
        match self {
            Self::LightBlue => "LightBlue",
            Self::DarkOrange => "DarkOrange",
            Self::WindowsLight => "WindowsLight",
            Self::WindowsDark => "WindowsDark",
            Self::Sakura => "Sakura",
            Self::Violet => "Violet",
            Self::Chrome => "Chrome",
        }
    }
}

// 2. 標準の AsRef<str> トレイトを実装
impl AsRef<str> for ColorScheme {
    fn as_ref(&self) -> &str {
        self.as_str()
    }
}

// 3. (オプション) 標準の Display トレイトを実装しておくと、format!("{}", scheme) などが可能に
impl fmt::Display for ColorScheme {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.as_str())
    }
}

// 1. anyhow::Error を返す TryFrom<&str> の実装
impl<'a> TryFrom<&'a str> for ColorScheme {
    type Error = anyhow::Error;

    fn try_from(value: &'a str) -> Result<Self, Self::Error> {
        match value {
            "Light Blue" | "LightBlue" => Ok(Self::LightBlue),
            "Dark Orange" | "DarkOrange" => Ok(Self::DarkOrange),
            "Windows Light" | "WindowsLight" => Ok(Self::WindowsLight),
            "Windows Dark" | "WindowsDark" => Ok(Self::WindowsDark),
            "Sakura" => Ok(Self::Sakura),
            "Violet" => Ok(Self::Violet),
            "Chrome" => Ok(Self::Chrome),
            _ => Err(anyhow!("Undefined ColorScheme: '{value}'")),
        }
    }
}

impl FromStr for ColorScheme {
    type Err = anyhow::Error;

    fn from_str(value: &str) -> Result<Self, Self::Err> {
        Self::try_from(value)
            .with_context(|| format!("Failed convert string to ColorScheme: {value}"))
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Deserialize, Serialize, Default)]
pub struct Config {
    pub color_scheme: ColorScheme,
    pub locale: Locale,
}
