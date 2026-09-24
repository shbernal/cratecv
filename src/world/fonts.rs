//! The faces the binary carries.
//!
//! Four static instances of Noto Serif, built by `scripts/build-fonts.sh` and
//! committed under `assets/fonts/`. They all share the family name, so Typst
//! selects between them by weight class and italic bit.

use typst_library::foundations::Bytes;
use typst_library::text::{Font, FontBook};

const FACES: [&[u8]; 4] = [
    include_bytes!("../../assets/fonts/NotoSerif-Regular.ttf"),
    include_bytes!("../../assets/fonts/NotoSerif-SemiBold.ttf"),
    include_bytes!("../../assets/fonts/NotoSerif-ExtraBold.ttf"),
    include_bytes!("../../assets/fonts/NotoSerif-Italic.ttf"),
];

/// The family every built-in theme sets text in.
pub const FAMILY: &str = "Noto Serif";

/// Load the embedded faces and the book that indexes them.
pub fn embedded() -> (Vec<Font>, FontBook) {
    let fonts: Vec<Font> = FACES
        .iter()
        .map(|face| Font::new(Bytes::new(*face), 0).expect("an embedded face should always parse"))
        .collect();
    let book = FontBook::from_fonts(&fonts);
    (fonts, book)
}
