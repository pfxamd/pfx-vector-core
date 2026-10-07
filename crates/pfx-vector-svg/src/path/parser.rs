use pfx_vector_core::Path;

use super::{SvgPathCommand, normalize_path_commands};

#[derive(Clone, Debug, PartialEq)]
pub struct SvgDiagnostic {
    pub message: String,
    pub start: usize,
    pub end: usize,
}

#[derive(Clone, Debug, PartialEq)]
pub struct SvgError {
    pub diagnostic: SvgDiagnostic,
}

impl core::fmt::Display for SvgError {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(
            f,
            "{} at {}..{}",
            self.diagnostic.message, self.diagnostic.start, self.diagnostic.end
        )
    }
}

impl std::error::Error for SvgError {}

struct Parser<'a> {
    input: &'a str,
    position: usize,
}

impl<'a> Parser<'a> {
    fn new(input: &'a str) -> Self {
        Self { input, position: 0 }
    }

    fn bytes(&self) -> &[u8] {
        self.input.as_bytes()
    }

    fn skip_separators(&mut self) {
        while let Some(byte) = self.bytes().get(self.position) {
            if byte.is_ascii_whitespace() || *byte == b',' {
                self.position += 1;
            } else {
                break;
            }
        }
    }

    fn command(&mut self) -> Option<u8> {
        self.skip_separators();
        let byte = *self.bytes().get(self.position)?;
        if byte.is_ascii_alphabetic() {
            self.position += 1;
            Some(byte)
        } else {
            None
        }
    }

    fn has_number(&mut self) -> bool {
        self.skip_separators();
        matches!(
            self.bytes().get(self.position),
            Some(b'+' | b'-' | b'.' | b'0'..=b'9')
        )
    }

    fn number(&mut self) -> Result<f64, SvgError> {
        self.skip_separators();
        let start = self.position;
        let bytes = self.bytes();

        if matches!(bytes.get(self.position), Some(b'+' | b'-')) {
            self.position += 1;
        }

        let mut has_digit = false;
        while matches!(bytes.get(self.position), Some(b'0'..=b'9')) {
            has_digit = true;
            self.position += 1;
        }

        if bytes.get(self.position) == Some(&b'.') {
            self.position += 1;
            while matches!(bytes.get(self.position), Some(b'0'..=b'9')) {
                has_digit = true;
                self.position += 1;
            }
        }

        if !has_digit {
            return Err(SvgError {
                diagnostic: SvgDiagnostic {
                    message: "expected number".into(),
                    start,
                    end: self.position,
                },
            });
        }

        if matches!(bytes.get(self.position), Some(b'e' | b'E')) {
            let exponent_start = self.position;
            self.position += 1;

            if matches!(bytes.get(self.position), Some(b'+' | b'-')) {
                self.position += 1;
            }

            let digits_start = self.position;
            while matches!(bytes.get(self.position), Some(b'0'..=b'9')) {
                self.position += 1;
            }

            if digits_start == self.position {
                self.position = exponent_start;
            }
        }

        let value: f64 = self.input[start..self.position]
            .parse()
            .map_err(|_| SvgError {
                diagnostic: SvgDiagnostic {
                    message: "invalid number".into(),
                    start,
                    end: self.position,
                },
            })?;

        if !value.is_finite() {
            return Err(SvgError {
                diagnostic: SvgDiagnostic {
                    message: "non-finite number".into(),
                    start,
                    end: self.position,
                },
            });
        }

        Ok(value)
    }

    fn flag(&mut self) -> Result<bool, SvgError> {
        self.skip_separators();
        match self.bytes().get(self.position) {
            Some(b'0') => {
                self.position += 1;
                Ok(false)
            }
            Some(b'1') => {
                self.position += 1;
                Ok(true)
            }
            _ => Err(SvgError {
                diagnostic: SvgDiagnostic {
                    message: "expected arc flag".into(),
                    start: self.position,
                    end: self.position + 1,
                },
            }),
        }
    }
}

pub fn parse_path_commands(data: &str) -> Result<Vec<SvgPathCommand>, SvgError> {
    let mut parser = Parser::new(data);
    let mut output = Vec::new();
    let mut active_command = None;
    let mut first_move_pair = false;

    loop {
        parser.skip_separators();

        if parser.position >= data.len() {
            break;
        }

        if let Some(command) = parser.command() {
            active_command = Some(command);
            first_move_pair = matches!(command, b'M' | b'm');

            if matches!(command, b'Z' | b'z') {
                output.push(SvgPathCommand::Close);
                active_command = None;
                continue;
            }
        }

        let command = active_command.ok_or_else(|| SvgError {
            diagnostic: SvgDiagnostic {
                message: "expected SVG path command".into(),
                start: parser.position,
                end: (parser.position + 1).min(data.len()),
            },
        })?;

        let relative = command.is_ascii_lowercase();
        let normalized = command.to_ascii_uppercase();

        if !parser.has_number() {
            return Err(SvgError {
                diagnostic: SvgDiagnostic {
                    message: "expected command parameters".into(),
                    start: parser.position,
                    end: parser.position,
                },
            });
        }

        loop {
            let parsed = match normalized {
                b'M' => {
                    let x = parser.number()?;
                    let y = parser.number()?;

                    if first_move_pair {
                        first_move_pair = false;
                        SvgPathCommand::Move { relative, x, y }
                    } else {
                        SvgPathCommand::Line { relative, x, y }
                    }
                }
                b'L' => SvgPathCommand::Line {
                    relative,
                    x: parser.number()?,
                    y: parser.number()?,
                },
                b'H' => SvgPathCommand::Horizontal {
                    relative,
                    x: parser.number()?,
                },
                b'V' => SvgPathCommand::Vertical {
                    relative,
                    y: parser.number()?,
                },
                b'C' => SvgPathCommand::Cubic {
                    relative,
                    x1: parser.number()?,
                    y1: parser.number()?,
                    x2: parser.number()?,
                    y2: parser.number()?,
                    x: parser.number()?,
                    y: parser.number()?,
                },
                b'S' => SvgPathCommand::SmoothCubic {
                    relative,
                    x2: parser.number()?,
                    y2: parser.number()?,
                    x: parser.number()?,
                    y: parser.number()?,
                },
                b'Q' => SvgPathCommand::Quadratic {
                    relative,
                    x1: parser.number()?,
                    y1: parser.number()?,
                    x: parser.number()?,
                    y: parser.number()?,
                },
                b'T' => SvgPathCommand::SmoothQuadratic {
                    relative,
                    x: parser.number()?,
                    y: parser.number()?,
                },
                b'A' => SvgPathCommand::Arc {
                    relative,
                    rx: parser.number()?,
                    ry: parser.number()?,
                    rotation: parser.number()?,
                    large_arc: parser.flag()?,
                    sweep: parser.flag()?,
                    x: parser.number()?,
                    y: parser.number()?,
                },
                _ => {
                    return Err(SvgError {
                        diagnostic: SvgDiagnostic {
                            message: "unsupported command".into(),
                            start: parser.position.saturating_sub(1),
                            end: parser.position,
                        },
                    });
                }
            };

            output.push(parsed);
            parser.skip_separators();

            if parser.position >= data.len()
                || parser.bytes()[parser.position].is_ascii_alphabetic()
                || !parser.has_number()
            {
                break;
            }
        }
    }

    Ok(output)
}

pub fn parse_path(data: &str) -> Result<Path, SvgError> {
    let commands = parse_path_commands(data)?;
    normalize_path_commands(&commands).map_err(|error| SvgError {
        diagnostic: SvgDiagnostic {
            message: format!("geometry normalization failed: {error}"),
            start: 0,
            end: data.len().min(1),
        },
    })
}
