use ratatui::style::Color;
#[derive(Clone, Copy)]
pub struct Theme {
    pub name: &'static str,
    pub panel_style: usize,
    pub graph_style: usize,
    pub bg: Color,
    pub panel: Color,
    pub fg: Color,
    pub muted: Color,
    pub accent: Color,
    pub secondary: Color,
    pub error: Color,
}
impl Theme {
    // Reset is a terminal-dependent foreground; use a defined dark ink on colored selections.
    pub fn on_accent(self) -> Color {
        if self.bg == Color::Reset {
            Color::Black
        } else {
            self.bg
        }
    }
}
const fn rgb(v: u32) -> Color {
    Color::Rgb((v >> 16) as u8, (v >> 8) as u8, v as u8)
}
const fn make(name: &'static str, colors: [u32; 7]) -> Theme {
    Theme {
        name,
        panel_style: 0,
        graph_style: 0,
        bg: rgb(colors[0]),
        panel: rgb(colors[1]),
        fg: rgb(colors[2]),
        muted: rgb(colors[3]),
        accent: rgb(colors[4]),
        secondary: rgb(colors[5]),
        error: rgb(colors[6]),
    }
}
// Existing IDs preserve profiles; new palettes are appended. Colors are curated here, not imported assets.
pub const THEMES: [Theme; 28] = [
    make(
        "fern",
        [
            0x10171d, 0x19232b, 0xd7e2e1, 0x80969f, 0xa2e097, 0x88c9d6, 0xff7984,
        ],
    ),
    make(
        "paper",
        [
            0xefebe1, 0xe3ddcf, 0x303734, 0x6b7268, 0x26745c, 0x526ea3, 0xb73443,
        ],
    ),
    make(
        "moon",
        [
            0x191724, 0x242033, 0xe0def4, 0x9790b0, 0xc4a7e7, 0x9ccfd8, 0xeb6f92,
        ],
    ),
    make(
        "carbon",
        [
            0x181a1d, 0x23262b, 0xe4e4dd, 0x929698, 0xe8bd55, 0x7bbbd1, 0xf0757f,
        ],
    ),
    make(
        "tide",
        [
            0x0e2029, 0x18323d, 0xd7eef2, 0x7b9ca8, 0x67d7d0, 0x83b7ff, 0xff8792,
        ],
    ),
    make(
        "ember",
        [
            0x211b19, 0x302723, 0xeee0ce, 0xa49182, 0xffae72, 0xd2c284, 0xf37c87,
        ],
    ),
    make(
        "orchid",
        [
            0x201a2a, 0x2e253c, 0xede3f5, 0xa697b3, 0xe8a0cd, 0xa9b9f8, 0xff8585,
        ],
    ),
    make(
        "moss",
        [
            0x20251d, 0x2b3327, 0xe1e5d6, 0x97a28c, 0xb6ce87, 0xd7bd7a, 0xef9b8e,
        ],
    ),
    make(
        "glacier",
        [
            0xe8f0f4, 0xdce7ee, 0x263c4d, 0x657e8b, 0x276eab, 0x7569b7, 0xbd4157,
        ],
    ),
    make(
        "ink",
        [
            0x101010, 0x1d1d1d, 0xf0f0f0, 0x969696, 0xffffff, 0xb5c4d2, 0xff9393,
        ],
    ),
    make(
        "solar",
        [
            0x002b36, 0x073642, 0xc6d5d4, 0x81999b, 0xe1b954, 0x6cc9b9, 0xf27b75,
        ],
    ),
    Theme {
        name: "terminal",
        panel_style: 0,
        graph_style: 0,
        bg: Color::Reset,
        panel: Color::Reset,
        fg: Color::Reset,
        muted: Color::DarkGray,
        accent: Color::Green,
        secondary: Color::Cyan,
        error: Color::Red,
    },
    make(
        "aurora",
        [
            0x102321, 0x18332e, 0xdaeee6, 0x82a89b, 0x78e6b1, 0xb7a1ed, 0xff8a9b,
        ],
    ),
    make(
        "cobalt",
        [
            0x0d1730, 0x162747, 0xe0e8fc, 0x8a9dc7, 0x72b5ff, 0xffd282, 0xff8399,
        ],
    ),
    make(
        "rose",
        [
            0x261a24, 0x382634, 0xf4e0ea, 0xbc93aa, 0xf5a5c7, 0xc6b3ef, 0xff9f87,
        ],
    ),
    make(
        "dune",
        [
            0x282219, 0x393024, 0xefe2c9, 0xb4a183, 0xe9c382, 0xb5c99a, 0xf69a85,
        ],
    ),
    make(
        "lavender",
        [
            0xf0eaf5, 0xe3daec, 0x392e4a, 0x7e6b91, 0x7448a2, 0x346d85, 0xb33762,
        ],
    ),
    make(
        "matcha",
        [
            0xeaf0df, 0xdde5cf, 0x354331, 0x6a7b60, 0x497234, 0x876c25, 0xb23c50,
        ],
    ),
    make(
        "candy",
        [
            0x20192e, 0x302440, 0xf3e8ff, 0xaa92c5, 0xfca6e2, 0x8cdef2, 0xff929d,
        ],
    ),
    make(
        "ocean",
        [
            0x081e2b, 0x102f40, 0xd9edf3, 0x79a4b9, 0x5cdbed, 0xa8c7ff, 0xff9d9d,
        ],
    ),
    make(
        "espresso",
        [
            0x211917, 0x332723, 0xeddfd2, 0xb29a89, 0xd9b596, 0xb9cba7, 0xf49f94,
        ],
    ),
    make(
        "copper",
        [
            0x241d1b, 0x382b26, 0xf2e0d3, 0xb59889, 0xf3ad81, 0x8fc9bf, 0xff8194,
        ],
    ),
    make(
        "arcade",
        [
            0x16162b, 0x22223b, 0xf2efff, 0xa09cc4, 0xf1ed86, 0x8cf5d0, 0xff8eb2,
        ],
    ),
    make(
        "blueprint",
        [
            0xe6edf6, 0xd7e2f0, 0x263e5d, 0x657e9f, 0x2864a6, 0x794ea5, 0xb8304c,
        ],
    ),
    make(
        "cherry",
        [
            0x29151c, 0x3d202a, 0xffe5e7, 0xbc929b, 0xff8d9d, 0xf1ca9b, 0xffb58b,
        ],
    ),
    make(
        "mint",
        [
            0xe3f1ec, 0xd2e5dc, 0x24463c, 0x5e8276, 0x216e57, 0x316d9c, 0xb03859,
        ],
    ),
    make(
        "amethyst",
        [
            0x1d1831, 0x2d2447, 0xece2fc, 0xa598c7, 0xc1a0ff, 0xf0c191, 0xff8fa8,
        ],
    ),
    make(
        "mono",
        [
            0xededed, 0xdedede, 0x252525, 0x6c6c6c, 0x303030, 0x505f76, 0xb12848,
        ],
    ),
];
pub fn get(id: usize) -> Theme {
    THEMES[id % THEMES.len()]
}

pub fn customized(id: usize, s: &crate::engine::Settings) -> Theme {
    let mut p = get(id);
    p.panel_style = s.panel_style;
    p.graph_style = s.graph_style;
    if s.accent > 0 {
        let light = matches!(p.bg,Color::Rgb(r,g,b) if (r as u32+g as u32+b as u32)>500);
        let colors = if light {
            [0x26745c, 0x166b8a, 0x876014, 0xa9394d, 0x6b4798]
        } else {
            [0xa2e097, 0x71d7eb, 0xeec76c, 0xff9394, 0xc5a6ef]
        };
        p.accent = rgb(colors[(s.accent - 1) % 5]);
    }
    p
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn theme_expansion_keeps_saved_ids_and_unique_names() {
        assert!(THEMES.len() >= 25);
        for (id, name) in [
            "fern", "paper", "moon", "carbon", "tide", "ember", "orchid", "moss", "glacier", "ink",
            "solar", "terminal",
        ]
        .iter()
        .enumerate()
        {
            assert_eq!(get(id).name, *name);
        }
        let names: std::collections::BTreeSet<_> = THEMES.iter().map(|t| t.name).collect();
        assert_eq!(names.len(), THEMES.len());
    }
}
