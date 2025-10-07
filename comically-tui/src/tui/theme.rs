use ratatui::style::{palette, Color};
use supports_color::Stream;

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum ThemeMode {
    Dark,
    Light,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum ColorCapability {
    TrueColor,
    Colors256,
}

impl ColorCapability {
    #[inline]
    fn adapt_color(&self, color: Color) -> Color {
        match self {
            ColorCapability::TrueColor => color,
            ColorCapability::Colors256 => match color {
                Color::Rgb(r, g, b) => {
                    let index = ansi_colours::ansi256_from_rgb((r, g, b));
                    Color::Indexed(index)
                }
                _ => color,
            },
        }
    }
}

impl ColorCapability {
    fn detect() -> Self {
        match supports_color::on(Stream::Stdout) {
            None => ColorCapability::Colors256,
            Some(color_info) => {
                if color_info.has_16m {
                    ColorCapability::TrueColor
                } else {
                    ColorCapability::Colors256
                }
            }
        }
    }
}

#[derive(Debug, Clone, Copy)]
pub struct Theme {
    pub mode: ThemeMode,
    pub color_capability: ColorCapability,

    pub border: Color,
    pub content: Color,
    pub background: Color,
    pub accent: Color,

    pub primary: Color,
    pub primary_bg: Color,
    pub primary_pressed: Color,

    pub secondary: Color,
    pub secondary_bg: Color,
    pub secondary_pressed: Color,

    pub error_bg: Color,
    pub scrollbar_thumb: Color,
    pub stage_colors: StageColors,

    // for text on progress bars
    pub gauge_label: Color,
    pub muted: Color,
}

#[derive(Debug, Clone, Copy)]
pub struct StageColors {
    pub process: Color,
    pub mobi: Color,
    pub epub: Color,
}

impl Theme {
    fn dark_with_capability(color_capability: ColorCapability) -> Self {
        let mut theme = match color_capability {
            ColorCapability::TrueColor => Self::dark_true_color(),
            ColorCapability::Colors256 => Self::dark_256(),
        };
        theme.color_capability = color_capability;
        theme
    }

    fn light_with_capability(color_capability: ColorCapability) -> Self {
        match color_capability {
            ColorCapability::TrueColor => Self::light_true_color(),
            ColorCapability::Colors256 => Self::light_256(),
        }
    }

    fn dark_true_color() -> Self {
        Self {
            mode: ThemeMode::Dark,
            color_capability: ColorCapability::TrueColor, // Will be overwritten
            border: palette::tailwind::SLATE.c400,
            content: palette::tailwind::SLATE.c200,
            background: palette::tailwind::SLATE.c950,
            accent: palette::tailwind::AMBER.c400,
            primary: palette::tailwind::CYAN.c200,
            primary_bg: palette::tailwind::CYAN.c600,
            primary_pressed: palette::tailwind::CYAN.c500,
            secondary: palette::tailwind::FUCHSIA.c300,
            secondary_bg: palette::tailwind::FUCHSIA.c600,
            secondary_pressed: palette::tailwind::FUCHSIA.c500,
            error_bg: palette::tailwind::RED.c800,
            scrollbar_thumb: palette::tailwind::CYAN.c500,
            gauge_label: palette::tailwind::SLATE.c200,
            muted: palette::tailwind::SLATE.c600,
            stage_colors: StageColors {
                process: palette::tailwind::PURPLE.c700,
                mobi: palette::tailwind::PINK.c700,
                epub: palette::tailwind::EMERALD.c700,
            },
        }
    }

    fn dark_256() -> Self {
        let mut t =
            Self::dark_true_color().map_colors(|c| ColorCapability::Colors256.adapt_color(c));
        t.color_capability = ColorCapability::Colors256;
        t
    }

    fn light_true_color() -> Self {
        // paper-like theme inspired by solarized light and old books
        Self {
            mode: ThemeMode::Light,
            color_capability: ColorCapability::TrueColor,
            border: palette::tailwind::STONE.c400, // Solarized base00 - darker for better contrast
            content: Color::Rgb(88, 110, 117),     // Solarized base01 - better contrast
            background: Color::Rgb(253, 246, 227), // Warm paper color (Solarized base3)
            accent: palette::tailwind::AMBER.c500,
            primary: palette::tailwind::SKY.c500,
            primary_bg: palette::tailwind::SKY.c200,
            primary_pressed: palette::tailwind::SKY.c100,
            secondary: palette::tailwind::VIOLET.c500,
            secondary_bg: palette::tailwind::VIOLET.c200,
            secondary_pressed: palette::tailwind::VIOLET.c100,
            error_bg: palette::tailwind::RED.c100,
            scrollbar_thumb: palette::tailwind::STONE.c400,
            gauge_label: palette::tailwind::STONE.c700,
            muted: palette::tailwind::STONE.c300,
            stage_colors: StageColors {
                process: palette::tailwind::PURPLE.c300,
                mobi: palette::tailwind::PINK.c300,
                epub: palette::tailwind::EMERALD.c300,
            },
        }
    }

    fn light_256() -> Self {
        let mut t =
            Self::light_true_color().map_colors(|c| ColorCapability::Colors256.adapt_color(c));
        t.color_capability = ColorCapability::Colors256;
        t
    }

    fn map_colors(&self, f: impl Fn(Color) -> Color) -> Self {
        Self {
            mode: self.mode,
            color_capability: self.color_capability,

            border: f(self.border),
            content: f(self.content),
            background: f(self.background),
            accent: f(self.accent),
            primary: f(self.primary),
            primary_bg: f(self.primary_bg),
            primary_pressed: f(self.primary_pressed),
            secondary: f(self.secondary),
            secondary_bg: f(self.secondary_bg),
            secondary_pressed: f(self.secondary_pressed),
            error_bg: f(self.error_bg),
            scrollbar_thumb: f(self.scrollbar_thumb),
            gauge_label: f(self.gauge_label),
            muted: f(self.muted),
            stage_colors: StageColors {
                process: f(self.stage_colors.process),
                mobi: f(self.stage_colors.mobi),
                epub: f(self.stage_colors.epub),
            },
        }
    }
}

impl Theme {
    pub fn is_dark(&self) -> bool {
        self.mode == ThemeMode::Dark
    }

    pub fn toggle(&mut self) {
        *self = match self.mode {
            ThemeMode::Dark => Self::light_with_capability(self.color_capability),
            ThemeMode::Light => Self::dark_with_capability(self.color_capability),
        };
    }

    pub fn adapt_color(&self, color: Color) -> Color {
        self.color_capability.adapt_color(color)
    }

    /// Convert an RGB color to the appropriate color based on terminal capabilities
    pub fn adapt_rgb(&self, r: u8, g: u8, b: u8) -> Color {
        match self.color_capability {
            ColorCapability::TrueColor => Color::Rgb(r, g, b),
            ColorCapability::Colors256 => {
                let index = ansi_colours::ansi256_from_rgb((r, g, b));
                Color::Indexed(index)
            }
        }
    }

    /// Detect terminal background and return appropriate theme using termbg
    /// Falls back to dark theme if detection fails or times out
    pub fn detect() -> Self {
        use std::time::Duration;

        let color_capability = ColorCapability::detect();

        // Try to detect terminal background with a 100ms timeout
        match termbg::theme(Duration::from_millis(100)) {
            Ok(termbg::Theme::Light) => Self::light_with_capability(color_capability),
            Ok(termbg::Theme::Dark) => Self::dark_with_capability(color_capability),
            Err(_) => Self::dark_with_capability(color_capability),
        }
    }
}
