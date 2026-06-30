// SPDX-FileCopyrightText: 2026 RAprogramm <andrey.rozanov.vl@gmail.com>
// SPDX-License-Identifier: MIT

//! Terminal color themes.

use ratatui::style::Color;
use serde::{Deserialize, Serialize};

/// Available color themes.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "snake_case")]
pub enum Theme {
    #[default]
    GruvboxDark,
    GruvboxLight,
    CatppuccinMocha,
    CatppuccinLatte
}

/// Color palette for a theme.
#[derive(Debug, Clone, Copy)]
#[allow(dead_code)]
pub struct Palette {
    pub bg:           Color,
    pub fg:           Color,
    pub border:       Color,
    pub title:        Color,
    pub header:       Color,
    pub selected:     Color,
    pub accent:       Color,
    pub success:      Color,
    pub warning:      Color,
    pub error:        Color,
    pub dim:          Color,
    pub tab_active:   Color,
    pub tab_inactive: Color
}

impl Theme {
    /// All selectable themes, in display order.
    pub const ALL: [Self; 4] = [
        Self::GruvboxDark,
        Self::GruvboxLight,
        Self::CatppuccinMocha,
        Self::CatppuccinLatte
    ];

    /// Returns the human-readable theme name.
    #[must_use]
    pub const fn label(self) -> &'static str {
        match self {
            Self::GruvboxDark => "Gruvbox Dark",
            Self::GruvboxLight => "Gruvbox Light",
            Self::CatppuccinMocha => "Catppuccin Mocha",
            Self::CatppuccinLatte => "Catppuccin Latte"
        }
    }

    /// Returns the stable identifier used in command ids and config.
    #[must_use]
    pub const fn id(self) -> &'static str {
        match self {
            Self::GruvboxDark => "gruvbox_dark",
            Self::GruvboxLight => "gruvbox_light",
            Self::CatppuccinMocha => "catppuccin_mocha",
            Self::CatppuccinLatte => "catppuccin_latte"
        }
    }

    /// Returns the palette for this theme.
    #[must_use]
    pub const fn palette(self) -> Palette {
        match self {
            Self::GruvboxDark => Palette {
                bg:           Color::Rgb(40, 40, 40),
                fg:           Color::Rgb(250, 241, 242),
                border:       Color::Rgb(101, 123, 131),
                title:        Color::Rgb(196, 160, 109),
                header:       Color::Rgb(250, 241, 242),
                selected:     Color::Rgb(250, 241, 242),
                accent:       Color::Rgb(137, 180, 250),
                success:      Color::Rgb(166, 227, 161),
                warning:      Color::Rgb(249, 226, 175),
                error:        Color::Rgb(243, 113, 113),
                dim:          Color::Rgb(146, 131, 116),
                tab_active:   Color::Rgb(250, 241, 242),
                tab_inactive: Color::Rgb(146, 131, 116)
            },
            Self::GruvboxLight => Palette {
                bg:           Color::Rgb(251, 241, 210),
                fg:           Color::Rgb(65, 56, 48),
                border:       Color::Rgb(146, 131, 116),
                title:        Color::Rgb(136, 91, 27),
                header:       Color::Rgb(65, 56, 48),
                selected:     Color::Rgb(65, 56, 48),
                accent:       Color::Rgb(69, 133, 136),
                success:      Color::Rgb(74, 128, 64),
                warning:      Color::Rgb(180, 111, 0),
                error:        Color::Rgb(184, 54, 54),
                dim:          Color::Rgb(146, 131, 116),
                tab_active:   Color::Rgb(65, 56, 48),
                tab_inactive: Color::Rgb(146, 131, 116)
            },
            Self::CatppuccinMocha => Palette {
                bg:           Color::Rgb(30, 30, 46),
                fg:           Color::Rgb(205, 214, 244),
                border:       Color::Rgb(102, 100, 125),
                title:        Color::Rgb(249, 226, 175),
                header:       Color::Rgb(205, 214, 244),
                selected:     Color::Rgb(205, 214, 244),
                accent:       Color::Rgb(137, 180, 250),
                success:      Color::Rgb(166, 227, 161),
                warning:      Color::Rgb(249, 226, 175),
                error:        Color::Rgb(243, 113, 113),
                dim:          Color::Rgb(141, 141, 162),
                tab_active:   Color::Rgb(205, 214, 244),
                tab_inactive: Color::Rgb(141, 141, 162)
            },
            Self::CatppuccinLatte => Palette {
                bg:           Color::Rgb(242, 239, 235),
                fg:           Color::Rgb(76, 79, 105),
                border:       Color::Rgb(161, 160, 170),
                title:        Color::Rgb(183, 108, 7),
                header:       Color::Rgb(76, 79, 105),
                selected:     Color::Rgb(76, 79, 105),
                accent:       Color::Rgb(92, 134, 233),
                success:      Color::Rgb(52, 161, 93),
                warning:      Color::Rgb(202, 137, 0),
                error:        Color::Rgb(210, 67, 67),
                dim:          Color::Rgb(161, 160, 170),
                tab_active:   Color::Rgb(76, 79, 105),
                tab_inactive: Color::Rgb(161, 160, 170)
            }
        }
    }
}
