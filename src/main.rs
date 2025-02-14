/**
 * RHAMSEZ, REMEMBER:
 * 8-BIT = 1 BYTE
 * 16-BIT = 2 BYTES
 * 32-BIT = 4 BYTES
 */
use std::collections::HashMap;
use std::fs;

static SIXTEEN_BIT: usize = 2;
static THIRTY_TWO_BIT: usize = 4;
static SIXTY_FOUR_BIT: usize = 8;

#[derive(Debug, PartialEq, Clone)]
enum TokenType {
    Text,
    ItalicOpen,
    ItalicClose,
    BoldOpen,
    BoldClose,
    ColorOpen,
    ColorClose,
}

#[derive(Clone)]
struct Token<'a> {
    text: &'a str,
    token_type: TokenType,
}

fn lexer(text: &str) -> Vec<Token> {
    let mut tokens: Vec<Token> = vec![];

    let mut start = 0;
    let mut in_tag = false;
    for (i, char) in text.chars().enumerate() {
        if in_tag {
            if char == ']' {
                let tag: &str = &text[start..i + 1];

                let token_type: TokenType;

                match tag {
                    "[b]" => token_type = TokenType::BoldOpen,
                    "[/b]" => token_type = TokenType::BoldClose,
                    "[i]" => token_type = TokenType::ItalicOpen,
                    "[/i]" => token_type = TokenType::ItalicClose,
                    "[/color]" => token_type = TokenType::ColorClose,
                    _ => {
                        if tag.len() == 15
                            && tag.starts_with("[color=#")
                            && tag.ends_with(']')
                            && tag[8..14].chars().all(|c| c.is_ascii_hexdigit())
                        {
                            token_type = TokenType::ColorOpen;
                        } else {
                            token_type = TokenType::Text;
                        }
                    }
                }

                tokens.push(Token {
                    text: tag,
                    token_type: token_type,
                });

                // now outside of tag
                in_tag = false;
                start = i + 1;
            }
        }

        if char == '[' {
            // capture previous text
            if start < i {
                tokens.push(Token {
                    text: &text[start..i],
                    token_type: TokenType::Text,
                });
            }

            // now inside of tag
            in_tag = true;
            start = i;
        }
    }
    if start <= text.len() - 1 {
        tokens.push(Token {
            text: &text[start..text.len()],
            token_type: TokenType::Text,
        });
    }

    return tokens;
}

#[derive(Debug, PartialEq, Copy, Clone)]
enum FontStyle {
    Normal,
    Italic,
}

#[derive(Debug, PartialEq, Copy, Clone)]
enum FontWeight {
    Normal,
    Bold,
}

#[derive(Copy, Clone)]
struct Style<'a> {
    color: &'a str,
    font_style: FontStyle,
    font_weight: FontWeight,
}

struct Node<'a> {
    style: Style<'a>,
    text: &'a str,
}

fn parser<'a>(tokens: Vec<Token<'a>>, style: Style<'a>) -> Vec<Node<'a>> {
    let mut output: Vec<Node> = vec![];

    let mut i: usize = 0;
    while i < tokens.len() {
        let token = &tokens[i];

        if token.token_type == TokenType::Text {
            output.push(Node {
                style: Style {
                    color: style.color,
                    font_style: style.font_style,
                    font_weight: style.font_weight,
                },
                text: token.text,
            });

            i += 1;
        } else {
            let close_type: TokenType;

            let mut new_style = style;

            // create variable that keeps track of parent color + styling for child color + styling
            match token.token_type {
                TokenType::BoldOpen => {
                    close_type = TokenType::BoldClose;
                    new_style.font_weight = FontWeight::Bold;
                }
                TokenType::ItalicOpen => {
                    close_type = TokenType::ItalicClose;
                    new_style.font_style = FontStyle::Italic;
                }
                TokenType::ColorOpen => {
                    close_type = TokenType::ColorClose;
                    new_style.color = &token.text[7..14];
                }
                _ => close_type = TokenType::Text,
            }

            // Find the index of the closing tag
            let close_index = tokens[i..]
                .iter()
                .position(|t| t.token_type == close_type)
                .map(|pos| i + pos)
                .unwrap_or(tokens.len());

            if close_index <= i {
                // Invalid syntax; skip to avoid infinite loop
                i += 1;
                continue;
            }

            // Process tokens between i+1 and close_index (exclusive)
            output.extend(parser(tokens[i + 1..close_index].to_vec(), new_style));
            i = close_index + 1; // Move past the closing tag
        }
    }
    return output;
}

fn to_i16(bytes: &Vec<u8>, index: usize) -> i16 {
    i16::from_be_bytes([bytes[index], bytes[index + 1]])
}

fn to_u16(bytes: &Vec<u8>, index: usize) -> u16 {
    u16::from_be_bytes([bytes[index], bytes[index + 1]])}

fn to_i32(bytes: &Vec<u8>, index: usize) -> i32 {
    i32::from_be_bytes([
        bytes[index],
        bytes[index + 1],
        bytes[index + 2],
        bytes[index + 3],
    ])
}

fn to_u32(bytes: &Vec<u8>, index: usize) -> u32 {
    u32::from_be_bytes([
        bytes[index],
        bytes[index + 1],
        bytes[index + 2],
        bytes[index + 3],
    ])
}

fn to_string(bytes: &Vec<u8>, index: usize) -> String {
    String::from_utf8(vec![
        bytes[index],
        bytes[index + 1],
        bytes[index + 2],
        bytes[index + 3],
    ])
    .unwrap_or_default()
}

fn read_font_file(path: &str) -> Vec<u8> {
    fs::read(path).expect("Should have been able to read the file")
}

fn parse_table_offsets(bytes: &Vec<u8>) -> HashMap<String, usize> {
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

fn get_num_glyphs(bytes: &Vec<u8>, table_offsets: &HashMap<String, usize>) -> u16 {
    let maxp_offset = table_offsets.get("maxp").copied().unwrap_or(0);
    return to_u16(bytes, maxp_offset + THIRTY_TWO_BIT);
}

fn get_glyph_offsets(
    bytes: &Vec<u8>,
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

fn get_glyph_id(bytes: &Vec<u8>, table_offsets: &HashMap<String, usize>, character: char) -> u16 {
    let cmap_offset = table_offsets.get("cmap").copied().unwrap_or(0);
    // let version = to_u16(bytes, cmap_offset);
    // let num_encoding_records = to_u16(bytes, cmap_offset + SIXTEEN_BIT);

    let encoding_offset = cmap_offset + 2 * SIXTEEN_BIT;
    let platform_id = to_u16(bytes, encoding_offset);
    let encoding_id = to_u16(bytes, encoding_offset + SIXTEEN_BIT);
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

#[derive(Debug)]
struct Point {
    x: i16,
    y: i16,
    on_curve: bool,
}

fn get_glyph_coordinates(bytes: &Vec<u8>, table_offsets: &HashMap<String, usize>, loca_offset: usize) -> Vec<Point> {
    let glyf_offset = table_offsets.get("glyf").copied().unwrap_or(0);
    let glyph_offset_start = glyf_offset + loca_offset;

    let number_of_contours = to_i16(&bytes, glyph_offset_start);

    let glyph_description_offset = glyph_offset_start + (5 * SIXTEEN_BIT);
    let mut end_pts_of_contours: Vec<u16> = vec![];

    for i in 0..number_of_contours {
        let contour_offset = glyph_description_offset + ((i as usize) * SIXTEEN_BIT);
        end_pts_of_contours.push(to_u16(&bytes, contour_offset));
    }

    let instruction_length_offset = glyph_description_offset + ((number_of_contours as usize) * SIXTEEN_BIT);
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
    let mut prev_x = 0;
    for i in 0..num_points {
        let flag = flags[i as usize];
        // Handle x coordinates
        if flag & (1 << 1) != 0 {
            let value = bytes[coordinates_offset] as i16;
            if flag & (1 << 4) != 0 {
                x_coordinates.push(value + prev_x);
                prev_x += value;
            } else {
                x_coordinates.push(-value + prev_x);
                prev_x += -value;
            }
            coordinates_offset += 1;
        } else if flag & (1 << 4) != 0 {
            x_coordinates.push(prev_x);
        } else {
            let value = to_i16(&bytes, coordinates_offset);
            x_coordinates.push(prev_x + value);
            prev_x += value;
            coordinates_offset += 2;
        }
    }

    let mut prev_y = 0;
    for i in 0..num_points {
        let flag = flags[i as usize];
        // Handle y coordinates
        if flag & (1 << 2) != 0 {
            let value = bytes[coordinates_offset] as i16;
            if flag & (1 << 5) != 0 {
                y_coordinates.push(value + prev_y);
                prev_y += value;
            } else {
                y_coordinates.push(-value + prev_y);
                prev_y += -value;
            }
            coordinates_offset += 1;
        } else if flag & (1 << 5) != 0 {
            y_coordinates.push(prev_y);
        } else {
            let value = to_i16(&bytes, coordinates_offset);
            y_coordinates.push(prev_y + value);
            prev_y += value;
            coordinates_offset += 2;
        }
    }

    let mut points: Vec<Point> = vec![];
    for i in 0..num_points {
        points.push(Point {
            x: x_coordinates[i as usize],
            y: y_coordinates[i as usize],
            on_curve: flags[i as usize] & (1 << 0) != 0,
        });
    }

    return points;

}
fn render_font(path: &str) {
    let bytes = read_font_file(path);
    let table_offsets = parse_table_offsets(&bytes);
    let num_glyphs = get_num_glyphs(&bytes, &table_offsets);
    let loca_offsets = get_glyph_offsets(&bytes, &table_offsets, num_glyphs);

    let example = "Hello World";
    let mut output = vec![];

    for e in example.chars() {
        let id = get_glyph_id(&bytes, &table_offsets, e);
        println!("{id}");

        if loca_offsets[id as usize] < loca_offsets[(id as usize) + 1] {
            let coords = get_glyph_coordinates(&bytes, &table_offsets, loca_offsets[id as usize]);
            output.push(coords);
        }
    }

    for point in output {
        println!("{:?}", point);
    }
}
fn main() {
    let text: &str = "resubbed for [b][color=#ff0000]10[/color] months[/b] at [b]Tier[color=#ff0000]3[/color]![/b]";
    let tokens: Vec<Token> = lexer(text);
    let output = parser(
        tokens,
        Style {
            color: "#FFFFFF",
            font_style: FontStyle::Normal,
            font_weight: FontWeight::Normal,
        },
    );

    for node in output {
        println!("text: \"{}\",\nstyle: {{\n  color: {},\n  font_style: {:?},\n  font_weight: {:?},\n}}\n", node.text, node.style.color, node.style.font_style, node.style.font_weight);
    }
    render_font("./fonts/Roboto.ttf");
}
