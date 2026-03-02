use anyhow::Result;
use crossterm::event::{self, Event, KeyCode, KeyEvent, KeyEventKind};
use devicons::FileIcon;
use ratatui::{
    layout::{Constraint, Layout, Position, Rect},
    style::{palette::tailwind::SLATE, Color, Modifier, Style, Stylize},
    symbols::border,
    text::{Line, Span, Text},
    widgets::{
        Block, Borders, HighlightSpacing, List, ListItem, ListState, Padding, Paragraph, Widget,
    },
    DefaultTerminal, Frame,
};
use std::{fs, str::FromStr};

#[derive(Debug)]
pub struct FdPath {
    path: String,
    name: String,
    spacer: String,
    is_dir: bool,
    is_expended: bool,
    total_paths: usize,
}

const SELECTED_STYLE: Style = Style::new().bg(SLATE.c800).add_modifier(Modifier::BOLD);
const NESTED_SPACER: &str = "|_";

pub fn init() -> Result<()> {
    let paths = retreive_paths(String::from("./"));
    let mut fd = Fd::new(paths);

    ratatui::run(|terminal| fd.run(terminal))
}

fn retreive_paths(path: String) -> Vec<FdPath> {
    let mut paths: Vec<FdPath> = Vec::new();

    for entry in fs::read_dir(path).expect("unable to read path") {
        let entry = entry.unwrap();
        let path = entry.path();
        let metadata = entry.metadata().unwrap();
        let p = format!("{}", path.display());
        let spacer = String::from("");

        if metadata.is_dir() {
            paths.push(FdPath {
                path: p,
                name: entry.file_name().into_string().unwrap(),
                spacer,
                is_expended: false,
                is_dir: true,
                total_paths: 0,
            });
        } else if metadata.is_file() {
            paths.push(FdPath {
                path: p,
                name: entry.file_name().into_string().unwrap(),
                spacer,
                is_expended: false,
                is_dir: false,
                total_paths: 0,
            });
        }
    }

    paths
}

#[derive(Debug)]
pub struct Fd {
    exit: bool,
    list_state: ListState,
    list: Vec<FdPath>,
    action: FdAction,
    input: String,
    input_index: usize,
}

#[derive(Debug)]
enum FdAction {
    Traverse,
    Select,
    Rename,
    Delete,
}

impl Fd {
    pub fn new(paths: Vec<FdPath>) -> Fd {
        Fd {
            exit: false,
            list_state: ListState::default(),
            list: paths,
            action: FdAction::Traverse,
            input: String::new(),
            input_index: 0,
        }
    }

    /// runs the application's main loop until the user quits
    pub fn run(&mut self, terminal: &mut DefaultTerminal) -> Result<()> {
        // defalt to select first item
        self.list_state.select_first();

        while !self.exit {
            terminal.draw(|frame| self.draw(frame))?;
            self.handle_events()?;
        }
        Ok(())
    }

    fn draw(&mut self, frame: &mut Frame) {
        let bottom_height = if let FdAction::Rename = self.action {
            3
        } else {
            1
        };

        // setup layouts for app
        let vertical = Layout::vertical([
            Constraint::Length(1),
            Constraint::Min(0),
            Constraint::Length(bottom_height),
        ]);
        let horizontal = Layout::horizontal([Constraint::Fill(1); 2]);

        let [title_area, main_area, bottom_area] = vertical.areas(frame.area());
        let [list_area, preview_area] = horizontal.areas(main_area);

        // title area
        frame.render_widget(Line::from(" File Explorer ".bold()).centered(), title_area);

        // list area
        self.render_list(frame, list_area);

        // preview area
        self.render_preview(frame, preview_area);

        // bottom area
        self.render_bottom(frame, bottom_area);
    }

    fn handle_events(&mut self) -> Result<()> {
        match event::read()? {
            // it's important to check that the event is a key press event as
            // crossterm also emits key release and repeat events on Windows.
            Event::Key(key_event) if key_event.kind == KeyEventKind::Press => {
                self.handle_key_event(key_event)
            }

            _ => {}
        };

        Ok(())
    }

    fn handle_key_event(&mut self, key_event: KeyEvent) {
        match self.action {
            FdAction::Traverse => match key_event.code {
                KeyCode::Char('q') => self.exit(),

                KeyCode::Char('j') => self.list_state.select_next(),

                KeyCode::Char('k') => self.list_state.select_previous(),

                KeyCode::Enter => self.toggle_expend(),

                KeyCode::Char('r') => {
                    self.action = FdAction::Rename;

                    if let Some(selected) = self.get_selected() {
                        self.input = selected.path.to_owned();
                    }
                }

                _ => {}
            },

            FdAction::Select => {}

            FdAction::Rename => match key_event.code {
                KeyCode::Esc => {
                    self.action = FdAction::Traverse;
                    self.reset_cursor();
                }

                KeyCode::Enter => {
                    let input_value = self.input.to_owned();

                    // rename file
                    if let Some(selected) = self.get_selected() {
                        let _ = fs::rename(&selected.path, input_value);
                    }

                    self.list = retreive_paths("./".to_string());
                    self.reset_cursor();
                    self.action = FdAction::Traverse;
                }

                KeyCode::Right => self.move_cursor_right(),

                KeyCode::Left => self.move_cursor_left(),

                KeyCode::Char(to_insert) => {
                    let index = self.byte_index();
                    self.input.insert(index, to_insert);
                    self.move_cursor_right();
                }

                KeyCode::Backspace => {
                    let is_not_cursor_leftmost = self.input_index != 0;
                    if is_not_cursor_leftmost {
                        // Method "remove" is not used on the saved text for deleting the selected char.
                        // Reason: Using remove on String works on bytes instead of the chars.
                        // Using remove would require special care because of char boundaries.

                        let current_index = self.input_index;
                        let from_left_to_current_index = current_index - 1;

                        // Getting all characters before the selected character.
                        let before_char_to_delete =
                            self.input.chars().take(from_left_to_current_index);
                        // Getting all characters after selected character.
                        let after_char_to_delete = self.input.chars().skip(current_index);

                        // Put all characters together except the selected one.
                        // By leaving the selected one out, it is forgotten and therefore deleted.
                        self.input = before_char_to_delete.chain(after_char_to_delete).collect();
                        self.move_cursor_left();
                    }
                }

                _ => {}
            },

            FdAction::Delete => {}
        }
    }

    fn reset_cursor(&mut self) {
        self.input.clear();
        self.input_index = 0;
    }

    fn clamp_cursor(&self, new_cursor_pos: usize) -> usize {
        new_cursor_pos.clamp(0, self.input.chars().count())
    }

    fn move_cursor_left(&mut self) {
        let cursor_moved_left = self.input_index.saturating_sub(1);
        self.input_index = self.clamp_cursor(cursor_moved_left);
    }

    fn move_cursor_right(&mut self) {
        let cursor_moved_right = self.input_index.saturating_add(1);
        self.input_index = self.clamp_cursor(cursor_moved_right);
    }

    fn byte_index(&self) -> usize {
        self.input
            .char_indices()
            .map(|(i, _)| i)
            .nth(self.input_index)
            .unwrap_or(self.input.len())
    }

    fn exit(&mut self) {
        self.exit = true;
    }

    fn get_selected(&mut self) -> Option<&FdPath> {
        if let Some(selected_idx) = self.list_state.selected() {
            if let Some(item) = self.list.get(selected_idx) {
                Some(item)
            } else {
                None
            }
        } else {
            None
        }
    }

    fn toggle_expend(&mut self) {
        if let Some(idx) = self.list_state.selected() {
            if let Some(item) = self.list.get_mut(idx) {
                if item.is_dir {
                    if item.is_expended {
                        item.is_expended = false;

                        let remove_idx_from = idx + 1;
                        let remove_idx_to = item.total_paths + remove_idx_from;

                        item.total_paths = 0;
                        self.list.drain(remove_idx_from..remove_idx_to);
                    } else {
                        item.is_expended = true;

                        let insert_idx = idx + 1;
                        let mut paths = retreive_paths(item.path.to_owned());

                        item.total_paths += paths.len();

                        paths.iter_mut().for_each(|p| {
                            p.spacer = item.spacer.to_owned() + NESTED_SPACER;
                        });

                        self.list.splice(insert_idx..insert_idx, paths);
                    }
                }
            }
        }
    }

    fn render_preview(&self, frame: &mut Frame, area: Rect) {
        frame.render_widget(
            Block::bordered()
                .title(Line::from(" Preview ".bold()).centered())
                .border_set(border::THICK),
            area,
        );
    }

    fn render_list(&mut self, frame: &mut Frame, area: Rect) {
        let items: Vec<ListItem> = self
            .list
            .iter()
            .map(|p| {
                let icon = FileIcon::from(&p.path);
                let spacer = &p.spacer;

                ListItem::from(
                    Text::from(Line::from(vec![
                        // spacer to show
                        Span::from(spacer),
                        // icon with color
                        Span::from(icon.to_string())
                            .style(Style::new().fg(Color::from_str(icon.color).unwrap())),
                        Span::from(" "),
                        // filename
                        Span::from(p.name.to_owned()),
                    ]))
                    .bold(),
                )
            })
            .collect();

        let list = List::new(items)
            .block(
                Block::bordered()
                    .title(Line::from(" All File/Dir ".bold()).centered())
                    .border_set(border::THICK),
            )
            .highlight_symbol(">> ".bold())
            .highlight_spacing(HighlightSpacing::Always)
            .highlight_style(SELECTED_STYLE);

        frame.render_stateful_widget(list, area, &mut self.list_state);
    }

    fn render_bottom(&mut self, frame: &mut Frame, area: Rect) {
        // render input if in Rename Action
        if let FdAction::Rename = self.action {
            // set cursor position inside box
            frame.set_cursor_position(Position::new(
                area.x + 1 + self.input_index as u16,
                area.y + 1,
            ));

            // render input
            frame.render_widget(
                Paragraph::new(self.input.to_owned()).block(
                    Block::bordered()
                        .title(Line::from(" Rename ".bold()).centered())
                        .title_bottom(Line::from(" submit <enter> "))
                        .border_set(border::THICK),
                ),
                area,
            );
        } else {
            frame.render_widget(
                Line::from(vec![
                    " Down ".into(),
                    "<j>".blue().bold(),
                    " Up ".into(),
                    "<k>".blue().bold(),
                    " Quit ".into(),
                    "<q> ".blue().bold(),
                ]),
                area,
            );
        }
    }
}
