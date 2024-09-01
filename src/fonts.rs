use super::*;
use web_sys::{Document, HtmlSpanElement};

pub static FONT_LIST: [&'static str; 89] = [
    "Arial",
    "Arial Black",
    "Arial Narrow",
    "Arial Rounded MT",
    "Arimo",
    "Archivo",
    "Barlow",
    "Bebas Neue",
    "Bitter",
    "Bookman",
    "Calibri",
    "Cabin",
    "Candara",
    "Century",
    "Century Gothic",
    "Comic Sans MS",
    "Constantia",
    "Courier",
    "Courier New",
    "Crimson Text",
    "DM Mono",
    "DM Sans",
    "DM Serif Display",
    "DM Serif Text",
    "Dosis",
    "Droid Sans",
    "Exo",
    "Fira Code",
    "Fira Sans",
    "Franklin Gothic Medium",
    "Garamond",
    "Geneva",
    "Georgia",
    "Gill Sans",
    "Helvetica",
    "Impact",
    "Inconsolata",
    "Indie Flower",
    "Inter",
    "Josefin Sans",
    "Karla",
    "Lato",
    "Lexend",
    "Lucida Bright",
    "Lucida Console",
    "Lucida Sans Unicode",
    "Manrope",
    "Merriweather",
    "Merriweather Sans",
    "Montserrat",
    "Myriad",
    "Noto Sans",
    "Nunito",
    "Nunito Sans",
    "Open Sans",
    "Optima",
    "Orbitron",
    "Oswald",
    "Pacifico",
    "Palatino",
    "Perpetua",
    "PT Sans",
    "PT Serif",
    "Poppins",
    "Prompt",
    "Public Sans",
    "Quicksand",
    "Rajdhani",
    "Recursive",
    "Roboto",
    "Roboto Condensed",
    "Rockwell",
    "Rubik",
    "Segoe Print",
    "Segoe Script",
    "Segoe UI",
    "Sora",
    "Source Sans Pro",
    "Space Mono",
    "Tahoma",
    "Taviraj",
    "Times",
    "Times New Roman",
    "Titillium Web",
    "Trebuchet MS",
    "Ubuntu",
    "Varela Round",
    "Verdana",
    "Work Sans",
];

pub fn detect_fonts(document: &Document) -> Option<Vec<bool>> {
    // a font will be compared against all the three default fonts.
    // and if it doesn't match all 3 then that font is not available.
    let base_fonts: [&'static str; 3] = ["monospace", "sans-serif", "serif"];
    //we use m or w because these two characters take up the maximum width.
    // And we use a LLi so that the same matching fonts can get separated
    let test_string = "mmmmmmmmmmlli";
    //we test using 72px font size, we may use any size. I guess larger the better.

    let text_size = "72px";
    let h = document
        .get_elements_by_tag_name("body")
        .get_with_index(0)?;
    // create a SPAN in the document to get the width of the text we use to test

    let s = document
        .create_element("span")
        .ok()?
        .dyn_into::<HtmlSpanElement>()
        .ok()?;
    s.style().set_property("font-size", text_size).ok()?;
    s.set_inner_html(test_string);
    let mut default_width = Vec::new();
    let mut default_height = Vec::new();
    for font in base_fonts {
        s.style().set_property("font-family", font).ok()?;
        h.append_child(&s).ok()?;
        default_width.push(s.offset_width());
        default_height.push(s.offset_height());
        h.remove_child(&s).ok()?;
    }
    let mut detect_font = Vec::new();

    for font in FONT_LIST {
        for (i, base_font) in base_fonts.into_iter().enumerate() {
            s.style()
                .set_property("font-family", &format!("{font},{base_font}"))
                .ok()?;
            h.append_child(&s).ok()?;
            let matched =
                s.offset_width() != default_width[i] || s.offset_height() != default_height[i];
            h.remove_child(&s).ok()?;
            detect_font.push(matched)
        }
    }
    Some(detect_font)
}
