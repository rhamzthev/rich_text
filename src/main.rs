/**
 * RHAMSEZ, REMEMBER:
 * 8-BIT = 1 BYTE
 * 16-BIT = 2 BYTES
 * 32-BIT = 4 BYTES
 */

use std::collections::HashMap;
use std::fs;

static EIGHT_BIT: usize = 1;
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
    ((bytes[index] as i16) << 8) | (bytes[index + 1] as i16)
}

fn to_u16(bytes: &Vec<u8>, index: usize) -> u16 {
    ((bytes[index] as u16) << 8) | (bytes[index + 1] as u16)
}

fn to_i32(bytes: &Vec<u8>, index: usize) -> i32 {
    ((bytes[index] as i32) << 24)
        | ((bytes[index + 1] as i32) << 16)
        | ((bytes[index + 2] as i32) << 8)
        | (bytes[index + 3] as i32)
}

fn to_u32(bytes: &Vec<u8>, index: usize) -> u32 {
    ((bytes[index] as u32) << 24)
        | ((bytes[index + 1] as u32) << 16)
        | ((bytes[index + 2] as u32) << 8)
        | (bytes[index + 3] as u32)
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

fn foo() {
    let bytes = fs::read("Roboto.ttf").expect("Should have been able to read the file");

    let num_tables = to_u16(&bytes, 4);

    let mut table_offsets: HashMap<String, usize> = HashMap::new();

    for i in 0..num_tables {
        // TODO: LEAVE COMMENT HERE
        let table_index = (16 * (i as usize)) + 12;

        let tag = to_string(&bytes, table_index);
        let offset = to_u32(&bytes, table_index + (2 * THIRTY_TWO_BIT));

        table_offsets.insert(tag, offset as usize);
    }

    println!("{:#?}", table_offsets);

    let maxp_offset = table_offsets.get("maxp").copied().unwrap_or(0);
    let num_glyphs = to_u16(&bytes, maxp_offset + THIRTY_TWO_BIT);

    let head_offset = table_offsets.get("head").copied().unwrap_or(0);
    let index_to_loc_format = to_i16(&bytes, head_offset + (12 * SIXTEEN_BIT) + (3 * THIRTY_TWO_BIT) + (2 * SIXTY_FOUR_BIT));
    // println!("{index_to_loc_format}");

    let loca_offset = table_offsets.get("loca").copied().unwrap_or(0);
    let mut loca_offsets: Vec<usize> = vec![];

    for i in 0..num_glyphs {
        if index_to_loc_format == 0 {
            let o = to_u16(&bytes, loca_offset + ((i as usize) * SIXTEEN_BIT));
            let p = o as u32;
            loca_offsets.push((p * 2) as usize);
        } else {
            let o = to_u32(&bytes, loca_offset + ((i as usize) * THIRTY_TWO_BIT));
            loca_offsets.push(o as usize);
        }
    }

    let cmap_offset = table_offsets.get("cmap").copied().unwrap_or(0);
    let version = to_u16(&bytes, cmap_offset);
    let num_encoding_records = to_u16(&bytes, cmap_offset + SIXTEEN_BIT);

    let encoding_offset = cmap_offset + 2 * SIXTEEN_BIT;
    let platform_id = to_u16(&bytes, encoding_offset);
    let encoding_id = to_u16(&bytes, encoding_offset + SIXTEEN_BIT);
    let subtable_offset = to_u32(&bytes, encoding_offset + (2 * SIXTEEN_BIT));
    // println!("{version}, {num_encoding_records}: {platform_id}, {encoding_id}, {subtable_offset}");

    let format_offset = cmap_offset + (subtable_offset as usize);
    let format = to_u16(&bytes, format_offset);
    let length = to_u16(&bytes, format_offset + SIXTEEN_BIT);
    let language = to_u16(&bytes, format_offset + (2 * SIXTEEN_BIT));
    let seg_count_x2 = to_u16(&bytes, format_offset + (3 * SIXTEEN_BIT));
    println!("{format} {length} {language} {seg_count_x2}");

    // for i in 0..num_encoding_records {
    //     let 
    // }

    // let test = "A";
    // let glyf_offset = table_offsets.get("glyf").copied().unwrap_or(0);

    // for i in 0..100 {
    //     let char_offset = glyf_offset + loca_offsets[i];
    //     let number_of_contours = to_i16(&bytes, char_offset);
    //     let x_min = to_i16(&bytes, char_offset + 2); 
    //     let y_min = to_i16(&bytes, char_offset + 4);
    //     let x_max = to_i16(&bytes, char_offset + 6);
    //     let y_max = to_i16(&bytes, char_offset + 8);
        
    //     println!("{}: Number of Contours: {} \t| X Min: {} \t| Y Min: {} \t| X Max: {} \t| Y Max: {}", 
    //        i,
    //        number_of_contours,
    //        x_min,
    //        y_min, 
    //        x_max,
    //        y_max
    //     );
    // }
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
    foo();
}
