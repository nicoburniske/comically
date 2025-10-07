use num_enum::TryFromPrimitive;
use serde::{Deserialize, Serialize};
use strum::{EnumCount, EnumIter, EnumTryAs};

use std::str::FromStr;

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, EnumTryAs)]
pub enum Device {
    Preset(Preset),
    Custom { width: u32, height: u32 },
}

impl Device {
    pub fn name(&self) -> &str {
        match self {
            Device::Preset(preset) => preset.name(),
            Device::Custom { .. } => "Custom",
        }
    }

    pub fn dimensions(&self) -> (u32, u32) {
        match self {
            Device::Preset(preset) => preset.dimensions(),
            Device::Custom { width, height } => (*width, *height),
        }
    }
}

impl From<Preset> for Device {
    fn from(preset: Preset) -> Self {
        Device::Preset(preset)
    }
}

#[derive(
    Debug, Copy, Clone, PartialEq, Serialize, Deserialize, TryFromPrimitive, EnumCount, EnumIter,
)]
#[repr(u8)]
pub enum Preset {
    KindlePw11 = 0,
    KindlePw12 = 1,
    KindleOasis = 2,
    KindleScribe = 3,
    KindleBasic = 4,
    Kindle11 = 5,
    KoboClaraHd = 6,
    KoboClara2e = 7,
    KoboLibra2 = 8,
    KoboSage = 9,
    KoboElipsa = 10,
    Remarkable2 = 11,
    IpadMini = 12,
    Ipad109 = 13,
    IpadPro11 = 14,
    OnyxBooxNova = 15,
    OnyxBooxNote = 16,
    PocketbookEra = 17,
}

impl Preset {
    pub fn len() -> usize {
        Self::COUNT
    }

    pub fn iter() -> impl Iterator<Item = Self> {
        <Self as strum::IntoEnumIterator>::iter()
    }

    pub fn name(&self) -> &str {
        match self {
            Preset::KindlePw11 => "Kindle PW 11",
            Preset::KindlePw12 => "Kindle PW 12",
            Preset::KindleOasis => "Kindle Oasis",
            Preset::KindleScribe => "Kindle Scribe",
            Preset::KindleBasic => "Kindle Basic",
            Preset::Kindle11 => "Kindle 11",
            Preset::KoboClaraHd => "Kobo Clara HD",
            Preset::KoboClara2e => "Kobo Clara 2E",
            Preset::KoboLibra2 => "Kobo Libra 2",
            Preset::KoboSage => "Kobo Sage",
            Preset::KoboElipsa => "Kobo Elipsa",
            Preset::Remarkable2 => "reMarkable 2",
            Preset::IpadMini => "iPad Mini",
            Preset::Ipad109 => "iPad 10.9",
            Preset::IpadPro11 => "iPad Pro 11",
            Preset::OnyxBooxNova => "Onyx Boox Nova",
            Preset::OnyxBooxNote => "Onyx Boox Note",
            Preset::PocketbookEra => "PocketBook Era",
        }
    }

    pub fn dimensions(&self) -> (u32, u32) {
        match self {
            Preset::KindlePw11 => (1236, 1648),
            Preset::KindlePw12 => (1264, 1680),
            Preset::KindleOasis => (1264, 1680),
            Preset::KindleScribe => (1860, 2480),
            Preset::KindleBasic => (600, 800),
            Preset::Kindle11 => (1072, 1448),
            Preset::KoboClaraHd => (1072, 1448),
            Preset::KoboClara2e => (1072, 1448),
            Preset::KoboLibra2 => (1264, 1680),
            Preset::KoboSage => (1440, 1920),
            Preset::KoboElipsa => (1404, 1872),
            Preset::Remarkable2 => (1404, 1872),
            Preset::IpadMini => (1488, 2266),
            Preset::Ipad109 => (1640, 2360),
            Preset::IpadPro11 => (1668, 2388),
            Preset::OnyxBooxNova => (1200, 1600),
            Preset::OnyxBooxNote => (1404, 1872),
            Preset::PocketbookEra => (1200, 1600),
        }
    }
}

#[derive(Debug)]
pub struct ParseError(String);

impl std::fmt::Display for ParseError {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        f.write_str("Invalid device preset: ")?;
        f.write_str(&self.0)
    }
}

impl std::error::Error for ParseError {}

impl TryFrom<&str> for Preset {
    type Error = ParseError;

    fn try_from(s: &str) -> Result<Self, Self::Error> {
        let normalized = s.to_lowercase().replace([' ', '_'], "-");
        match normalized.as_str() {
            "kindle-pw-11" => Ok(Preset::KindlePw11),
            "kindle-pw-12" => Ok(Preset::KindlePw12),
            "kindle-oasis" => Ok(Preset::KindleOasis),
            "kindle-scribe" => Ok(Preset::KindleScribe),
            "kindle-basic" => Ok(Preset::KindleBasic),
            "kindle-11" => Ok(Preset::Kindle11),
            "kobo-clara-hd" => Ok(Preset::KoboClaraHd),
            "kobo-clara-2e" => Ok(Preset::KoboClara2e),
            "kobo-libra-2" => Ok(Preset::KoboLibra2),
            "kobo-sage" => Ok(Preset::KoboSage),
            "kobo-elipsa" => Ok(Preset::KoboElipsa),
            "remarkable-2" => Ok(Preset::Remarkable2),
            "ipad-mini" => Ok(Preset::IpadMini),
            "ipad-109" => Ok(Preset::Ipad109),
            "ipad-pro-11" => Ok(Preset::IpadPro11),
            "onyx-boox-nova" => Ok(Preset::OnyxBooxNova),
            "onyx-boox-note" => Ok(Preset::OnyxBooxNote),
            "pocketbook-era" => Ok(Preset::PocketbookEra),
            _ => Err(ParseError(s.to_string())),
        }
    }
}

impl FromStr for Preset {
    type Err = ParseError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Self::try_from(s)
    }
}
