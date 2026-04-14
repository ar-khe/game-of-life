use std::fmt::Display;
use std::time::Duration;
use std::vec;

use chrono::Local;
use color_eyre::eyre::WrapErr;
use crossterm::{self, event};
use ratatui::text::{Line as TextLine, Span};
use ratatui::widgets::BorderType;
use ratatui::widgets::canvas::{Canvas, Line, Map, MapResolution, Points, Rectangle};
use ratatui::{
    self, DefaultTerminal,
    prelude::*,
    symbols::Marker,
    widgets::{Block, Paragraph, Widget},
};

const TIME_LIMIT_IN_SECONDS: i64 = 10;

fn main() -> color_eyre::Result<()> {
    let mut app = App::new();

    ratatui::run(|terminal| app.run(terminal))

    // let mut grid = Grid::new(2, 2);
    // println!("{:#?}", grid);
    // grid.set(0, 0, true)?;
    // println!("{:#?}", grid);

    // let mut game = GameOfLife::new(10, 10);
    // let starting_cells = vec![
    //     (0, 0),
    //     (0, 1),
    //     (1, 0),
    //     (9,0)
    // ];
    // game.init(starting_cells.clone())?;

    // println!("{}", game.grid);

    // starting_cells.iter().for_each(|(x, y)| {
    //     println!("({},{}), {:?}", x, y, game.surrounding_alive(*x as i32, *y as i32));
    // });

    // game.next_grid()?;
    // println!("{}", game.grid);
    // Ok(())
}

#[derive(PartialEq, PartialOrd)]
enum AppState {
    Running,
    Paused,
    Quit
}

struct App {
    state: AppState,
    started: chrono::DateTime<Local>
}

impl App {
    pub fn new() -> App {
        App {
            state: AppState::Running,
            started: chrono::Local::now()
        }
    }

    // App Logic and loop
    pub fn run(&mut self, terminal: &mut DefaultTerminal) -> color_eyre::Result<()> {
        let starting_area = terminal.get_frame().area();

        let mut game = GameOfLife::new(starting_area.width.into(), starting_area.height.into());
        // let mut game = GameOfLife::new(10, 10);
        game.init(vec![
            // (5, 8),
            // (5, 9),
            // (5, 7),

            (5,5),
            (6,5),
            (7,5),
            (8,5),
            (5,4),
            (8,4),
            (6,3),
            (7,3)

        ].iter().map(|(x, y)| (x+50, y+3)).collect())?;

        // while self.state == AppState::Running {
        loop {
            if {chrono::Local::now() - game.last_updated}.num_milliseconds() > 100 {
                game.next_grid()?;
            }
            terminal.draw(|frame| self.draw(frame, &game))?;

            // let time_passed = {chrono::Local::now() - self.started}.num_milliseconds();

            // terminal.draw(|frame| frame.render_widget(format!("{} - {} = {}", self.started, chrono::Local::now(), time_passed), frame.area()))?;

            if event::poll(Duration::from_millis(1)).context("event poll failed")? {
                if crossterm::event::read()?.is_key_press() {
                    break;
                }
            }

        }
        Ok(())
    }

    // Layout
    fn draw(&self, frame: &mut Frame, game: &GameOfLife) {
        let seconds_passed = {chrono::Local::now() - self.started}.num_seconds();

        let vertical = Layout::vertical([Constraint::Length(1), Constraint::Fill(1)]).spacing(0);
        let [_, main] = frame.area().layout(&vertical);

        let border = Block::bordered().style(Style::new().bold().red())
            .border_type(BorderType::Rounded)
            .title(TextLine::from(format!("{}", seconds_passed )).left_aligned())
            .title(TextLine::from("Game of Life").centered())
            .title(TextLine::from("Press 'q' to quit").right_aligned());

        let canvas_area = border.inner(main);

        frame.render_widget(border, main);
        frame.render_widget(game, canvas_area);
    }

}

#[derive(Clone)]
struct GameOfLife {
    grid: Grid,
    last_updated: chrono::DateTime<Local>,
}

impl GameOfLife {
    pub fn new(width: i32, height: i32) -> Self {
        GameOfLife {
            grid: Grid::new(width, height),
            last_updated: chrono::Local::now(),
        }
    }

    pub fn init(&mut self, alive_cells: Vec<(u32, u32)>) -> color_eyre::Result<()> {
        for (x, y) in alive_cells {
            self.grid.set(x as i32, y as i32, true)?
        }

        Ok(())
    }

    pub fn surrounding_alive(&self, x: i32, y: i32) -> color_eyre::Result<usize>{ 
        let current_value = self.grid.get(x, y)?;
        let surrounding_alive_amount = [
            self.grid.get(x - 1, y), // Left
            self.grid.get(x + 1, y), // Right
            self.grid.get(x, y - 1), // Top
            self.grid.get(x, y + 1), // Bottom
            self.grid.get(x - 1, y - 1), // Top-left
            self.grid.get(x + 1, y - 1), // Top-Right
            self.grid.get(x + 1, y + 1), // Bottom-right
            self.grid.get(x - 1, y + 1), // Bottom-left
        ]
        .iter()
        .filter_map(|v| v.as_ref().ok().filter(|v| *v == &true))
        .count();

        Ok(surrounding_alive_amount)
    }

    fn next_value(&self, x: i32, y: i32) -> color_eyre::Result<bool> {
        let current_value = self.grid.get(x, y)?;
        let surrounding_alive_amount = self.surrounding_alive(x, y)?;

        Ok(match current_value {
            // Any live cell with fewer than two live neighbours dies, as if by underpopulation.
            true if surrounding_alive_amount < 2 => false,
            // Any live cell with two or three live neighbours lives on to the next generation.
            true if [2, 3].contains(&surrounding_alive_amount) => true,
            // Any live cell with more than three live neighbours dies, as if by overpopulation.
            true if surrounding_alive_amount > 3 => false,
            // Any dead cell with exactly three live neighbours becomes a live cell, as if by reproduction.
            false if surrounding_alive_amount == 3 => true,
            _ => current_value,
        })
    }

    fn next_grid(&mut self) -> color_eyre::Result<()> {
        let mut new_grid = Grid::new(self.grid.width, self.grid.height);

        for x in 0..self.grid.width{
            for y in 0..self.grid.height{
                let new_cell = self.next_value(x, y)?;
                new_grid.set(x, y, new_cell)?;
            }
        }

        self.grid = new_grid;
        self.last_updated = chrono::Local::now();

        Ok(())
    }
}

#[derive(Clone, Debug)]
struct Grid {
    grid: Vec<Vec<bool>>,
    width: i32,
    height: i32
}

impl Grid {
    fn new(width: i32, height: i32) -> Self {
        Grid {
            width, height,
            grid: vec![vec![false; height as usize]; width as usize]
        }
    }

    fn get(&self, x: i32, y: i32) -> color_eyre::Result<bool> {
        if let Some(col) = self.grid.iter().nth(x as usize) {
            if let Some(cell) = col.iter().nth(y as usize) {
                Ok(cell.clone())
            } else {
                Err(color_eyre::Report::msg(format!(
                    "Y = {} out of Range for height {} ",
                    y, self.height
                )))
            }
        } else {
            Err(color_eyre::Report::msg(format!(
                "X = {} out of Range for width {}",
                x, self.width
            )))
        }
    }

    fn set(&mut self, x: i32, y: i32, value: bool) -> color_eyre::Result<()> {

        if let Some(col) = self.grid.get_mut(x as usize) {
            if let Some(cell) = col.get_mut(y as usize) {
                *cell = value;
                Ok(())
            } else {
                Err(color_eyre::Report::msg(format!(
                    "Y = {} out of Range for height {} ",
                    y, self.height
                )))
            }
        } else {
            Err(color_eyre::Report::msg(format!(
                "X = {} out of Range for width {}",
                x, self.width
            )))
        }
    }

    fn get_alive_cells(&self) -> Vec<(f64, f64)> {
        let mut alive_cells = vec![];

        for x in 0..self.width{
            for y in 0..self.height{
                if let Ok(value) = self.get(x, y) && value == true {
                    alive_cells.push((x as f64, y as f64));
                }
            }
        }

        alive_cells
    }

}

impl Widget for &GameOfLife {
    fn render(self, area: Rect, buf: &mut Buffer)
    where
        Self: Sized {

        let canvas = Canvas::default()
        .x_bounds([0f64, area.width as f64])
        .y_bounds([0f64, area.height as f64])
        .marker(Marker::Dot)
        .paint( move |ctx: &mut ratatui::widgets::canvas::Context<'_>| {
            ctx.draw(&Points {
                coords: &self.grid.get_alive_cells().as_slice(),
                // coords: &[
                //     (0.0, 0.0),
                //     (area.width as f64, area.height as f64),
                // ],
                color: Color::Cyan,
            });
        });


        canvas.render(area, buf);
    }
}

impl Display for Grid {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let mut out: String = String::new();
        for x in 0..self.width {
            out += "[";
            for y in 0..self.height {
                out += match self.grid[x as usize][y as usize] {
                    true => "1",
                    false => "0",
                };
                if y != {self.width - 1} {
                    out += ", "
                }
            }
            out += "]\n"
        }

        write!(f, "{}", out)
    }
}