use anyhow::Result;
use crossterm::event::{self, Event, KeyCode, KeyEvent, KeyEventKind};
use devicons::FileIcon;
use ratatui::{
    layout::{Constraint, Layout, Position, Rect},
    style::{palette::tailwind::SLATE, Color, Modifier, Style, Stylize},
    symbols::border,
    text::{Line, Span, Text},
    widgets::{Block, HighlightSpacing, List, ListItem, ListState, Paragraph},
    DefaultTerminal, Frame,
};
use std::{fs, iter, str::FromStr};

const SELECTED_STYLE: Style = Style::new().bg(SLATE.c800).add_modifier(Modifier::BOLD);
const NESTED_SPACER: &str = "|-";

pub fn init() -> Result<()> {
    let mut fd = Fd::new(String::from("./"));

    ratatui::run(|terminal| fd.run(terminal))
}

#[derive(Debug)]
pub struct FdPath {
    /// full path of the file or dir
    path: String,

    /// name of the file or dir
    name: String,

    /// if path is dir or not
    is_dir: bool,

    /// for dir type if dir is expended
    is_expended: bool,

    /// string to render front of the name and
    /// used to create a seperation
    spacer: String,

    /// path of direct parent dir
    direct_parent: String,

    /// paths of all parent dir
    parents: Vec<String>,

    /// total number of children path
    total_paths: usize,
}

#[derive(Debug)]
pub struct Fd {
    base_path: String,
    exit: bool,
    list_state: ListState,
    list: Vec<FdPath>,
    action: FdAction,
    input: String,
    input_index: usize,
}

#[derive(Debug)]
enum FdAction {
    Normal,
    Create,
    Rename,
    Delete,
}

impl Fd {
    pub fn new(path: String) -> Fd {
        let paths = Fd::retreive_paths(path.to_owned(), String::from(""), Vec::new());

        Fd {
            base_path: path.to_owned(),
            exit: false,
            list_state: ListState::default(),
            list: paths,
            action: FdAction::Normal,
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
        let bottom_height = if let FdAction::Normal = self.action {
            1
        } else {
            3
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

    fn retreive_paths(path: String, spacer: String, parents: Vec<String>) -> Vec<FdPath> {
        let mut paths: Vec<FdPath> = Vec::new();
        let dir = fs::read_dir(&path).expect("unable to read dir");

        for item in dir.into_iter().flatten() {
            if let Ok(metadata) = item.metadata() {
                let mut fd_path = FdPath {
                    path: item.path().to_string_lossy().to_string(),
                    name: item
                        .file_name()
                        .into_string()
                        .unwrap_or("unknown".to_string()),
                    spacer: spacer.to_owned(),
                    is_expended: false,
                    is_dir: metadata.is_dir(),
                    total_paths: 0,
                    direct_parent: path.to_owned(),
                    parents: Vec::new(),
                };

                if !parents.is_empty() {
                    fd_path.parents.append(&mut parents.to_vec());
                }

                fd_path.parents.push(path.to_owned());
                paths.push(fd_path);
            }
        }

        paths
    }

    fn handle_key_event(&mut self, key_event: KeyEvent) {
        match self.action {
            FdAction::Normal => match key_event.code {
                KeyCode::Char('q') => self.exit(),

                KeyCode::Char('j') => self.list_state.select_next(),

                KeyCode::Char('k') => self.list_state.select_previous(),

                KeyCode::Char('d') => {
                    self.action = FdAction::Delete;

                    if let Some(selected) = self.get_selected() {
                        self.input = selected.path.to_owned();
                    }
                }

                KeyCode::Char('a') => {
                    self.action = FdAction::Create;
                }

                KeyCode::Char('r') => {
                    self.action = FdAction::Rename;

                    if let Some(selected) = self.get_selected() {
                        self.input = selected.path.to_owned();
                    }
                }

                KeyCode::Enter => self.toggle_expend(),

                _ => {}
            },

            FdAction::Create => match key_event.code {
                KeyCode::Esc => {
                    self.action = FdAction::Normal;
                    self.reset_cursor();
                }

                KeyCode::Enter => {}

                _ => {}
            },

            FdAction::Rename => match key_event.code {
                KeyCode::Esc => {
                    self.action = FdAction::Normal;
                    self.reset_cursor();
                }

                KeyCode::Enter => {
                    let input_value = self.input.to_owned();

                    // rename file
                    if let Some(selected) = self.get_selected() {
                        fs::rename(&selected.path, input_value)
                            .expect("unable to rename something went wrong");
                    }

                    self.list =
                        Fd::retreive_paths(self.base_path.to_owned(), String::from(""), Vec::new());
                    self.reset_cursor();
                    self.action = FdAction::Normal;
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

            FdAction::Delete => match key_event.code {
                KeyCode::Esc => {
                    self.action = FdAction::Normal;
                    self.reset_cursor();
                }

                KeyCode::Enter => {
                    if let Some(selected) = self.get_selected() {
                        if selected.is_dir {
                            fs::remove_dir_all(&selected.path)
                                .expect("unable to delete dir something went wrong");
                        } else {
                            fs::remove_file(&selected.path)
                                .expect("unable to delete something went wrong");
                        }
                    }

                    self.list =
                        Fd::retreive_paths(self.base_path.to_owned(), String::from(""), Vec::new());
                    self.reset_cursor();
                    self.action = FdAction::Normal;
                }

                _ => {}
            },
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
        if let Some(selected_idx) = self.list_state.selected() {
            if let Some(item) = self.list.get(selected_idx) {
                if item.is_dir {
                    if item.is_expended {
                        self.collaspe(selected_idx);
                    } else {
                        self.expand(selected_idx);
                    }
                }
            }
        }
    }

    fn collaspe(&mut self, selected_idx: usize) {
        if let Some(item) = self.list.get_mut(selected_idx) {
            item.total_paths = 0;
            item.is_expended = false;

            let path = item.path.to_owned();

            let mut remove_idxs: Vec<usize> = Vec::new();
            for (i, list_item) in self.list.iter().enumerate() {
                for p in list_item.parents.iter() {
                    if *p == path {
                        remove_idxs.push(i);
                    }
                }
            }

            if !remove_idxs.is_empty() {
                let remove_idx_from = remove_idxs[0];
                let remove_idx_to = remove_idxs[remove_idxs.len() - 1] + 1;

                self.list.drain(remove_idx_from..remove_idx_to);
            }
        }
    }

    fn expand(&mut self, selected_idx: usize) {
        if let Some(item) = self.list.get_mut(selected_idx) {
            let insert_idx = selected_idx + 1;
            let spacer = item.spacer.to_owned() + NESTED_SPACER;
            let paths = Fd::retreive_paths(item.path.to_owned(), spacer, item.parents.to_vec());

            item.total_paths += paths.len();
            item.is_expended = true;

            self.list.splice(insert_idx..insert_idx, paths);
        }
    }

    fn render_preview(&self, frame: &mut Frame, area: Rect) {
        let selected_idx = self.list_state.selected();
        let item = self.list.get(selected_idx.unwrap()).unwrap();

        let block = Block::bordered()
            .title(Line::from(" Preview ".bold()).centered())
            .border_set(border::THICK);

        let text = vec![
            Line::from(vec![
                Span::raw("name: "),
                Span::styled(item.name.to_owned(), Style::new().green().italic()),
            ]),
            Line::from(vec![
                Span::raw("path: "),
                Span::styled(item.path.to_owned(), Style::new().green().italic()),
            ]),
            Line::from(vec![
                Span::raw("parents: "),
                Span::styled(
                    format!("{:?}", item.parents.to_owned()),
                    Style::new().green().italic(),
                ),
            ]),
            Line::from(vec![
                Span::raw("total_paths: "),
                Span::styled(
                    format!("{:?}", item.total_paths),
                    Style::new().green().italic(),
                ),
            ]),
        ];
        let preview = Paragraph::new(text).block(block);

        frame.render_widget(preview, area);
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
        match self.action {
            FdAction::Delete => {
                let delete_block = Block::bordered()
                    .title(Line::from(" Delete ".bold()).centered())
                    .title_bottom(
                        Line::from(vec![
                            " submit ".into(),
                            "<enter>".blue().bold(),
                            " exit ".into(),
                            "<esc>".blue().bold(),
                        ])
                        .centered(),
                    )
                    .border_set(border::THICK);

                // render input
                let delete = Paragraph::new(self.input.to_owned()).block(delete_block);

                frame.render_widget(delete, area);
            }

            FdAction::Rename => {
                // set cursor position inside box
                frame.set_cursor_position(Position::new(
                    area.x + 1 + self.input_index as u16,
                    area.y + 1,
                ));

                // input box
                let input_block = Block::bordered()
                    .title(Line::from(" Rename ".bold()).centered())
                    .title_bottom(
                        Line::from(vec![
                            " submit ".into(),
                            "<enter>".blue().bold(),
                            " move left ".into(),
                            "<left>".blue().bold(),
                            " move right ".into(),
                            "<right>".blue().bold(),
                            " exit ".into(),
                            "<esc>".blue().bold(),
                        ])
                        .centered(),
                    )
                    .border_set(border::THICK);

                // render input
                let input = Paragraph::new(self.input.to_owned()).block(input_block);

                frame.render_widget(input, area);
            }

            _ => {
                let instructions = Line::from(vec![
                    " delete ".into(),
                    "<d>".blue().bold(),
                    " rename ".into(),
                    "<r>".blue().bold(),
                    " Down ".into(),
                    "<j>".blue().bold(),
                    " Up ".into(),
                    "<k>".blue().bold(),
                    " Quit ".into(),
                    "<q> ".blue().bold(),
                ]);
                frame.render_widget(instructions, area);
            }
        }
    }
}
