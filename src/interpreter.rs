use std::collections::HashMap;
use std::{fs, vec};

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