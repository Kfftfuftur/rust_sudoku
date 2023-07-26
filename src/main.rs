use iced::alignment::{Horizontal, Vertical};
use iced::widget::{button, column, row, text, Button, Column};
use iced::{Alignment, Element, Renderer, Sandbox, Settings};
use std::time::Instant;

const SMALLSIZE: usize = 3;
const SIZE: usize = SMALLSIZE * SMALLSIZE;
const BUTTONSIZE: f32 = 100.0;

fn main() -> iced::Result {
    Sudoku::run(Settings::default())
}

#[derive(Debug, Clone, Copy)]
struct FieldCoords {
    y: usize,
    x: usize,
}

type Options = [bool; SIZE];

#[derive(Debug, Clone, Copy)]
enum OptionCoords {
    None,
    One { coords: FieldCoords },
    TooMany,
}

#[derive(Debug, Clone, Copy)]
struct Update {
    coords: FieldCoords,
    number: u8,
}

impl Default for OptionCoords {
    fn default() -> Self {
        OptionCoords::None
    }
}

#[derive(Debug, Clone, Copy)]
enum Field {
    Number { number: u8, auto: bool },
    Empty { options: Options },
    Invalid,
}

impl Default for Field {
    fn default() -> Self {
        Field::Empty {
            options: [true; SIZE],
        }
    }
}

struct Sudoku {
    field: [[Field; SIZE]; SIZE],
}

enum ButtonStyle {
    Field { auto: bool },
    Option,
    Invalid,
}

impl button::StyleSheet for ButtonStyle {
    type Style = iced::Theme;

    fn active(&self, _style: &Self::Style) -> button::Appearance {
        match self {
            ButtonStyle::Field { auto } => {
                button::Appearance {
                    //background: Some(iced::Background::Color(iced::Color::from_rgb8(0, 147, 220))),
                    background: if *auto {
                        Some(iced::Background::Color(iced::Color::from_rgb8(
                            200, 200, 200,
                        )))
                    } else {
                        Some(iced::Background::Color(iced::Color::from_rgb8(0, 220, 220)))
                    },
                    border_color: iced::Color::WHITE,
                    border_width: 1.0,
                    border_radius: 0.0,
                    ..button::Appearance::default()
                }
            }
            ButtonStyle::Option => button::Appearance {
                background: Some(iced::Background::Color(iced::Color::from_rgb8(0, 220, 220))),
                border_color: iced::Color::WHITE,
                border_width: 1.0,
                border_radius: 0.0,
                ..button::Appearance::default()
            },
            ButtonStyle::Invalid => button::Appearance {
                background: Some(iced::Background::Color(iced::Color::from_rgb8(220, 50, 50))),
                border_color: iced::Color::WHITE,
                border_width: 1.0,
                border_radius: 0.0,
                ..button::Appearance::default()
            },
        }
    }
}

impl From<ButtonStyle> for iced::theme::Button {
    fn from(style: ButtonStyle) -> Self {
        iced::theme::Button::Custom(Box::new(style))
    }
}

#[derive(Debug, Clone, Copy)]
enum Message {
    FieldUpdated { coords: FieldCoords, number: Field },
    None,
}

impl Sandbox for Sudoku {
    type Message = Message;

    fn new() -> Self {
        let a = Sudoku {
            field: [[Field::default(); SIZE]; SIZE],
        };

        return a;
    }

    fn title(&self) -> String {
        let mut auto = 0;
        let mut entered = 0;
        for x in 0..SIZE {
            for y in 0..SIZE {
                match self.field[y][x] {
                    Field::Number { auto: false, ..} => {entered += 1},
                    Field::Number { auto: true, ..} => {auto += 1},
                    _ => (),
                }
            }
        };
        let solve = auto as f32 / ((SIZE * SIZE - entered) as f32);
        format!("Sudoku {:.1}% Solved", 100.0*solve)
    }

    fn update(&mut self, message: Message) {
        match message {
            Message::FieldUpdated { coords, number } => {
                self.field[coords.y][coords.x] = number;

                match number {
                    Field::Number { number, .. } => println!(
                        "Wrote Number {} to Field ({}, {})",
                        number, coords.x, coords.y
                    ),
                    Field::Empty { .. } => {
                        println!("Cleared Field ({}, {})", coords.x, coords.y)
                    }
                    _ => (),
                };
                self.update_options();
            }
            Message::None => (),
        }
    }

    fn view(&self) -> Element<Message> {
        column![self.view_field(),]
            .align_items(Alignment::Center)
            .into()
    }
}

impl Sudoku {
    fn update_options(&mut self) {
        let before = Instant::now();

        for x in 0..SIZE {
            for y in 0..SIZE {
                match self.field[y][x] {
                    Field::Number { auto: false, .. } => (),
                    _ => self.field[y][x] = Field::default(),
                }
            }
        }

        let mut updated = true;
        while updated {
            updated = self.reduce_fieldwise();
            if !updated {
                updated = self.reduce_numberwise();
            }
        }

        println!("Update took:  {:.2?}", before.elapsed());
    }

    fn reduce_fieldwise(&mut self) -> bool {
        let mut updated: bool = false;
        for x in 0..SIZE {
            for y in 0..SIZE {
                match self.field[y][x] {
                    Field::Empty { .. } => {
                        let mut options = [true; SIZE];
                        for i in 0..SIZE {
                            match self.field[y][i] {
                                Field::Number { number, .. } => {
                                    options[(number - 1) as usize] = false;
                                }
                                _ => (),
                            }
                            match self.field[i][x] {
                                Field::Number { number, .. } => {
                                    options[(number - 1) as usize] = false;
                                }
                                _ => (),
                            }
                        }
                        let x_base = x - x % SMALLSIZE;
                        let y_base = y - y % SMALLSIZE;
                        for i in 0..SMALLSIZE {
                            for j in 0..SMALLSIZE {
                                match self.field[y_base + i][x_base + j] {
                                    Field::Number { number, .. } => {
                                        options[(number - 1) as usize] = false;
                                    }
                                    _ => (),
                                }
                            }
                        }

                        let mut number = None;
                        let mut solved = true;
                        for i in 0..SIZE {
                            if options[i] {
                                match number {
                                    Some(_) => {
                                        solved = false;
                                    }
                                    None => number = Some((i + 1) as u8),
                                }
                            }
                        }
                        if solved {
                            match number {
                                Some(number) => {
                                    self.field[y][x] = Field::Number { number, auto: true };
                                    updated = true;
                                }
                                None => self.field[y][x] = Field::Invalid,
                            }
                        } else {
                            self.field[y][x] = Field::Empty { options }
                        }
                    }
                    _ => (),
                }
            }
        }
        return updated;
    }

    fn reduce_numberwise(&mut self) -> bool {
        let mut updates: Vec<Update> = vec![];
        for x in 0..SIZE {
            let mut optioncoords: [OptionCoords; SIZE] = [OptionCoords::default(); SIZE];
            for y in 0..SIZE {
                match self.field[y][x] {
                    Field::Empty { options } => {
                        for i in 0..SIZE {
                            if options[i] {
                                match optioncoords[i] {
                                    OptionCoords::None => {
                                        optioncoords[i] = OptionCoords::One {
                                            coords: FieldCoords { y, x },
                                        }
                                    }
                                    OptionCoords::One { .. } => {
                                        optioncoords[i] = OptionCoords::TooMany
                                    }
                                    _ => {}
                                }
                            }
                        }
                    }
                    _ => {}
                }
            }
            for i in 0..SIZE {
                match optioncoords[i] {
                    OptionCoords::One { coords } => {
                        updates.push(Update { coords, number: (i+1) as u8 })
                    }
                    _ => {}
                }
            }
        }

        for y in 0..SIZE {
            let mut optioncoords: [OptionCoords; SIZE] = [OptionCoords::default(); SIZE];
            for x in 0..SIZE {
                match self.field[y][x] {
                    Field::Empty { options } => {
                        for i in 0..SIZE {
                            if options[i] {
                                match optioncoords[i] {
                                    OptionCoords::None => {
                                        optioncoords[i] = OptionCoords::One {
                                            coords: FieldCoords { y, x },
                                        }
                                    }
                                    OptionCoords::One { .. } => {
                                        optioncoords[i] = OptionCoords::TooMany
                                    }
                                    _ => {}
                                }
                            }
                        }
                    }
                    _ => {}
                }
            }
            for i in 0..SIZE {
                match optioncoords[i] {
                    OptionCoords::One { coords } => {
                        updates.push(Update { coords, number: (i+1) as u8 })
                    }
                    _ => {}
                }
            }
        }

        for x in 0..SMALLSIZE {
            for y in 0..SMALLSIZE {
                let mut optioncoords: [OptionCoords; SIZE] = [OptionCoords::default(); SIZE];
                for j in 0..SMALLSIZE {
                    for k in 0..SMALLSIZE {
                        let xcoord = SMALLSIZE * x + k;
                        let ycoord = SMALLSIZE * y + j;
                        match self.field[ycoord][xcoord] {
                            Field::Empty { options } => {
                                for i in 0..SIZE {
                                    if options[i] {
                                        match optioncoords[i] {
                                            OptionCoords::None => {
                                                optioncoords[i] = OptionCoords::One {
                                                    coords: FieldCoords {
                                                        y: ycoord,
                                                        x: xcoord,
                                                    },
                                                }
                                            }
                                            OptionCoords::One { .. } => {
                                                optioncoords[i] = OptionCoords::TooMany
                                            }
                                            _ => {}
                                        }
                                    }
                                }
                            }
                            _ => {}
                        }
                    }
                }
                for i in 0..SIZE {
                    match optioncoords[i] {
                        OptionCoords::One { coords} => {
                            updates.push(Update { coords, number: (i+1) as u8 })
                        }
                        _ => {}
                    }
                }
            }
        }
        for update in &updates {
            let x = update.coords.x;
            let y = update.coords.y;
            self.field[y][x] = Field::Number { number: update.number, auto: true };
        }
        return updates.len() != 0;
    }

    fn view_field(&self) -> Column<'_, Message, Renderer> {
        let mut columns = column!();

        for y in 0..SMALLSIZE {
            let mut rows = row!();
            for x in 0..SMALLSIZE {
                rows = rows.push(self.view_small_square(x, y));
            }
            columns = columns.push(rows);
        }
        columns.padding(20).align_items(Alignment::Center)
    }

    fn view_small_square(&self, group_x: usize, group_y: usize) -> Column<'_, Message, Renderer> {
        let mut columns = column!();
        for y in 0..SMALLSIZE {
            let mut rows = row!();
            for x in 0..SMALLSIZE {
                rows =
                    rows.push(self.view_number(SMALLSIZE * group_x + x, SMALLSIZE * group_y + y));
            }
            columns = columns.push(rows);
        }
        columns.padding(2)
    }

    fn view_number(&self, x: usize, y: usize) -> Button<'_, Message, Renderer> {
        let b = match self.field[y][x] {
            Field::Number { number, auto } => {
                let t = text(number)
                    .size(BUTTONSIZE * 2.0 / 3.0)
                    .horizontal_alignment(Horizontal::Center)
                    .vertical_alignment(Vertical::Center);

                button(t).style(ButtonStyle::Field { auto }.into())
            }

            Field::Empty { options } => self
                .view_options(FieldCoords { y, x }, options)
                .style(ButtonStyle::Field { auto: false }.into()),

            Field::Invalid => button(text(" ")).style(ButtonStyle::Invalid.into()),
        };

        let msg = match self.field[y][x] {
            Field::Number { .. } => Message::FieldUpdated {
                coords: FieldCoords { y, x },
                number: Field::default(),
            },
            _ => Message::None,
        };

        b.on_press(msg).height(BUTTONSIZE).width(BUTTONSIZE)
    }

    fn view_options(
        &self,
        coords: FieldCoords,
        options: [bool; SIZE],
    ) -> Button<'_, Message, Renderer> {
        let mut columns = column!().align_items(Alignment::Center);
        let mut rows = row!().align_items(Alignment::Center);

        for n in 0..SIZE {
            let t = if options[n] { text(n + 1) } else { text(" ") }
                .size(BUTTONSIZE / (SMALLSIZE as f32) * 2.0 / 3.0)
                .vertical_alignment(Vertical::Center)
                .horizontal_alignment(Horizontal::Center);

            let b = button(t)
                .on_press(if options[n] {
                    Message::FieldUpdated {
                        coords,
                        number: Field::Number {
                            number: (n + 1) as u8,
                            auto: false,
                        },
                    }
                } else {
                    Message::None
                })
                .width((BUTTONSIZE - 8.0) / (SMALLSIZE as f32))
                .height((BUTTONSIZE - 8.0) / (SMALLSIZE as f32))
                .padding(0.0)
                .style(ButtonStyle::Option.into());
            rows = rows.push(b);

            if (n + 1) % SMALLSIZE == 0 {
                columns = columns.push(rows);
                rows = row!().align_items(Alignment::Center);
            };
        }
        button(columns)
    }
}
