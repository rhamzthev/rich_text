/**
 * RHAMSEZ, REMEMBER:
 * 8-BIT = 1 BYTE
 * 16-BIT = 2 BYTES
 * 32-BIT = 4 BYTES
 */
use serde::Serialize;
use serde_wasm_bindgen;
use std::collections::HashMap;
use wasm_bindgen::prelude::*;
// use std::fs;
extern crate console_error_panic_hook;

static SIXTEEN_BIT: usize = 2;
static THIRTY_TWO_BIT: usize = 4;
static SIXTY_FOUR_BIT: usize = 8;

#[wasm_bindgen]
extern "C" {
    // Use `js_namespace` here to bind `console.log(..)` instead of just
    // `log(..)`
    #[wasm_bindgen(js_namespace = console)]
    fn log(s: &str);

    // The `console.log` is quite polymorphic, so we can bind it with multiple
    // signatures. Note that we need to use `js_name` to ensure we always call
    // `log` in JS.
    #[wasm_bindgen(js_namespace = console, js_name = log)]
    fn log_u16(a: u16);

    #[wasm_bindgen(js_namespace = console, js_name = log)]
    fn log_usize(a: usize);

    // Multiple arguments too!
    #[wasm_bindgen(js_namespace = console, js_name = log)]
    fn log_many(a: &str, b: &str);
}

#[derive(Debug, Serialize)]
struct Point {
    x: i16,
    y: i16,
    #[serde(rename = "onCurve")]
    on_curve: bool,
}

#[derive(Debug, Serialize)]
struct Contour {
    points: Vec<Point>,
}

#[derive(Debug, Serialize)]
struct SimpleGlyph {
    #[serde(rename = "xMin")]
    x_min: i16,
    #[serde(rename = "yMin")]
    y_min: i16,
    #[serde(rename = "xMax")]
    x_max: i16,
    #[serde(rename = "yMax")]
    y_max: i16,
    contours: Vec<Contour>,
}

#[derive(Debug, Serialize)]
struct Font {
    glyphs: HashMap<char, SimpleGlyph>,
}

fn to_i16(bytes: &Box<[u8]>, index: usize) -> i16 {
    i16::from_be_bytes([bytes[index], bytes[index + 1]])
}

fn to_u16(bytes: &Box<[u8]>, index: usize) -> u16 {
    u16::from_be_bytes([bytes[index], bytes[index + 1]])
}

// fn to_i32(bytes: &Box<[u8]>, index: usize) -> i32 {
//     i32::from_be_bytes([
//         bytes[index],
//         bytes[index + 1],
//         bytes[index + 2],
//         bytes[index + 3],
//     ])
// }

fn to_u32(bytes: &Box<[u8]>, index: usize) -> u32 {
    u32::from_be_bytes([
        bytes[index],
        bytes[index + 1],
        bytes[index + 2],
        bytes[index + 3],
    ])
}

fn to_string(bytes: &Box<[u8]>, index: usize) -> String {
    String::from_utf8(vec![
        bytes[index],
        bytes[index + 1],
        bytes[index + 2],
        bytes[index + 3],
    ])
    .unwrap_or_default()
}

fn parse_table_offsets(bytes: &Box<[u8]>) -> HashMap<String, usize> {
    let num_tables = to_u16(bytes, 4);
    let mut table_offsets: HashMap<String, usize> = HashMap::new();

    for i in 0..num_tables {
        let table_index = (16 * (i as usize)) + 12;
        let tag = to_string(bytes, table_index);
        let offset = to_u32(bytes, table_index + (2 * THIRTY_TWO_BIT));
        table_offsets.insert(tag, offset as usize);
    }

    return table_offsets;
}

fn get_num_glyphs(bytes: &Box<[u8]>, table_offsets: &HashMap<String, usize>) -> u16 {
    let maxp_offset = table_offsets.get("maxp").copied().unwrap_or(0);
    return to_u16(bytes, maxp_offset + THIRTY_TWO_BIT);
}

fn get_glyph_offsets(
    bytes: &Box<[u8]>,
    table_offsets: &HashMap<String, usize>,
    num_glyphs: u16,
) -> Vec<usize> {
    let head_offset = table_offsets.get("head").copied().unwrap_or(0);
    let index_to_loc_format = to_i16(
        bytes,
        head_offset + (12 * SIXTEEN_BIT) + (3 * THIRTY_TWO_BIT) + (2 * SIXTY_FOUR_BIT),
    );

    let loca_offset = table_offsets.get("loca").copied().unwrap_or(0);
    let mut loca_offsets: Vec<usize> = vec![];

    for i in 0..num_glyphs {
        if index_to_loc_format == 0 {
            let o = to_u16(bytes, loca_offset + ((i as usize) * SIXTEEN_BIT));
            loca_offsets.push(o as usize * 2);
        } else {
            let o = to_u32(bytes, loca_offset + ((i as usize) * THIRTY_TWO_BIT));
            loca_offsets.push(o as usize);
        }
    }
    return loca_offsets;
}

fn get_glyph_id(bytes: &Box<[u8]>, table_offsets: &HashMap<String, usize>, character: char) -> u16 {
    let cmap_offset = table_offsets.get("cmap").copied().unwrap_or(0);
    // let version = to_u16(bytes, cmap_offset);
    // let num_encoding_records = to_u16(bytes, cmap_offset + SIXTEEN_BIT);

    let encoding_offset = cmap_offset + 2 * SIXTEEN_BIT;
    // let platform_id = to_u16(bytes, encoding_offset);
    // let encoding_id = to_u16(bytes, encoding_offset + SIXTEEN_BIT);
    let subtable_offset = to_u32(bytes, encoding_offset + (2 * SIXTEEN_BIT));

    let format_offset = cmap_offset + (subtable_offset as usize);
    // let format = to_u16(bytes, format_offset);
    // let length = to_u16(bytes, format_offset + SIXTEEN_BIT);
    // let language = to_u16(bytes, format_offset + (2 * SIXTEEN_BIT));
    let seg_count = to_u16(bytes, format_offset + (3 * SIXTEEN_BIT)) / 2;

    let end_code_offset = format_offset + (7 * SIXTEEN_BIT);
    let start_code_offset = end_code_offset + ((seg_count as usize) * SIXTEEN_BIT) + SIXTEEN_BIT;
    let id_delta_offset = start_code_offset + ((seg_count as usize) * SIXTEEN_BIT);
    let id_range_offset_offset = id_delta_offset + ((seg_count as usize) * SIXTEEN_BIT);

    for i in 0..seg_count {
        let end_code_index = end_code_offset + ((i as usize) * SIXTEEN_BIT);
        let end_code = to_u16(bytes, end_code_index);

        let start_code_index = start_code_offset + ((i as usize) * SIXTEEN_BIT);
        let start_code = to_u16(bytes, start_code_index);

        let id_delta_index = id_delta_offset + ((i as usize) * SIXTEEN_BIT);
        let id_delta = to_i16(bytes, id_delta_index);

        let id_range_offset_index = id_range_offset_offset + ((i as usize) * SIXTEEN_BIT);
        let id_range_offset = to_u16(bytes, id_range_offset_index);

        if end_code >= (character as u16) && start_code <= (character as u16) {
            if id_range_offset != 0 {
                let new_offset = ((character as u16) - start_code) + id_range_offset;
                let glyf_id_offset = (id_range_offset_index as u16) + new_offset;
                let glyf_id = to_u16(bytes, glyf_id_offset as usize);

                if glyf_id != 0 {
                    return (glyf_id as i32 + id_delta as i32) as u16;
                } else {
                    return 0;
                }
            } else {
                return (id_delta as i32 + character as i32) as u16;
            }
        }
    }
    return 0;
}

fn get_glyph(
    bytes: &Box<[u8]>,
    table_offsets: &HashMap<String, usize>,
    loca_offset: usize,
) -> SimpleGlyph {
    let glyf_offset = table_offsets.get("glyf").copied().unwrap_or(0);
    let glyph_offset_start = glyf_offset + loca_offset;

    let number_of_contours = to_i16(&bytes, glyph_offset_start);

    if number_of_contours >= 0 {

        let glyph_description_offset = glyph_offset_start + (5 * SIXTEEN_BIT);
        let mut end_pts_of_contours: Vec<u16> = vec![];

        for i in 0..number_of_contours {
            let contour_offset = glyph_description_offset + ((i as usize) * SIXTEEN_BIT);
            end_pts_of_contours.push(to_u16(&bytes, contour_offset));
        }

        let instruction_length_offset =
            glyph_description_offset + ((number_of_contours as usize) * SIXTEEN_BIT);
        let instruction_length = to_u16(&bytes, instruction_length_offset);

        let instructions_offset = instruction_length_offset + SIXTEEN_BIT;
        let flags_offset = instructions_offset + (instruction_length as usize);
        let mut flags: Vec<u8> = vec![];

        let num_points: usize = (end_pts_of_contours.last().copied().unwrap_or(0) + 1) as usize;
        let mut flags_length: usize = 0;

        while flags.len() < num_points {
            let flag = bytes[flags_offset + flags_length];
            flags.push(flag);

            if flag & (1 << 3) != 0 {
                flags_length += 1;

                let repeat = bytes[flags_offset + flags_length];

                for _ in 0..repeat {
                    flags.push(flag);
                }

                flags_length += 1;
            } else {
                flags_length += 1;
            }
        }

        let mut x_coordinates: Vec<i16> = vec![];
        let mut y_coordinates: Vec<i16> = vec![];

        let mut coordinates_offset = flags_offset + (flags_length as usize);
        // let mut prev_x = 0;
        for i in 0..num_points {
            let flag = flags[i as usize];
            // Handle x coordinates
            if flag & (1 << 1) != 0 {
                let value = bytes[coordinates_offset] as i16;
                if flag & (1 << 4) != 0 {
                    x_coordinates.push(value);
                } else {
                    x_coordinates.push(-value);
                }
                coordinates_offset += 1;
            } else if flag & (1 << 4) != 0 {
                x_coordinates.push(0);
            } else {
                let value = to_i16(&bytes, coordinates_offset);
                x_coordinates.push(value);
                coordinates_offset += 2;
            }
        }

        // let mut prev_y = 0;
        for i in 0..num_points {
            let flag = flags[i as usize];
            // Handle y coordinates
            if flag & (1 << 2) != 0 {
                let value = bytes[coordinates_offset] as i16;
                if flag & (1 << 5) != 0 {
                    y_coordinates.push(value);
                } else {
                    y_coordinates.push(-value);
                }
                coordinates_offset += 1;
            } else if flag & (1 << 5) != 0 {
                y_coordinates.push(0);
            } else {
                let value = to_i16(&bytes, coordinates_offset);
                y_coordinates.push(value);
                coordinates_offset += 2;
            }
        }

        let mut contours: Vec<Contour> = vec![];

        let mut start_pt = 0;
        for end_pt in end_pts_of_contours {

            let mut points = vec![];

            for i in start_pt..(end_pt + 1) {
                points.push(Point {
                    x: x_coordinates[i as usize],
                    y: y_coordinates[i as usize],
                    on_curve: flags[i as usize] & (1 << 0) != 0,
                });
            }

            let contour: Contour = Contour {
                points: points
            };

            start_pt = end_pt + 1;
            contours.push(contour);
        }

        let simple_glyph: SimpleGlyph = SimpleGlyph {
            x_min: to_i16(&bytes, glyph_offset_start + SIXTEEN_BIT),
            y_min: to_i16(&bytes, glyph_offset_start + (2 * SIXTEEN_BIT)),
            x_max: to_i16(&bytes, glyph_offset_start + (3 * SIXTEEN_BIT)),
            y_max: to_i16(&bytes, glyph_offset_start + (4 * SIXTEEN_BIT)),
            contours: contours
        };

        return simple_glyph;
    } else {
        return SimpleGlyph {
            x_min: 0,
            y_min: 0,
            x_max: 0,
            y_max: 0,
            contours: vec![]
        };
    }
}

fn unique_chars(text: &str) -> Vec<char> {
    let mut chars: Vec<char> = text.chars().filter(|c| !c.is_whitespace()).collect();
    chars.sort_unstable();
    chars.dedup();
    return chars;
}

#[wasm_bindgen]
pub fn render_font(text: String, bytes: Box<[u8]>) -> JsValue {
    console_error_panic_hook::set_once();

    // let bytes = read_font_file(&path);
    let table_offsets = parse_table_offsets(&bytes);
    let num_glyphs = get_num_glyphs(&bytes, &table_offsets);
    let loca_offsets = get_glyph_offsets(&bytes, &table_offsets, num_glyphs);

    let mut glyphs: HashMap<char, SimpleGlyph> = HashMap::new();

    let chars = unique_chars(&text);

    for c in chars {
        let id: u16 = get_glyph_id(&bytes, &table_offsets, c);

        println!("{id}");

        if loca_offsets[id as usize] < loca_offsets[(id as usize) + 1] {
            let glyph: SimpleGlyph = get_glyph(&bytes, &table_offsets, loca_offsets[id as usize]);
            glyphs.insert(c, glyph);
        }
    }

    let font = Font {
        glyphs: glyphs
    };

    serde_wasm_bindgen::to_value(&font).unwrap()
}
