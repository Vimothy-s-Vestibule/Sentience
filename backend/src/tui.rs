use ratatui::{
    layout::{Constraint, Direction, Layout},
    style::{Color, Modifier, Style},
    text::Span,
    widgets::{Block, Borders, Gauge, List, ListItem, ListState, Paragraph, Wrap},
};
use tui_logger::{TuiLoggerLevelOutput, TuiLoggerWidget};
use crossterm::event::KeyCode;

use syl_scr::DiscordMessage;
use syl_scr::RecordStatus;
use syl_scr::VestibuleUserRecord;

pub enum AppEvent {
    Init(Vec<(VestibuleUserRecord, DiscordMessage)>),
    NewPending(Vec<(VestibuleUserRecord, DiscordMessage)>),
    Processing(String), // user_id
    Scored(VestibuleUserRecord, DiscordMessage),
    Input(KeyCode),
    Tick,
}

pub struct App {
    pub users: Vec<(VestibuleUserRecord, DiscordMessage)>,
    pub list_state: ListState,
    pub processing_id: Option<String>,
    pub should_quit: bool,
    pub search_query: String,
    pub search_mode: bool,
}

impl App {
    pub fn new() -> Self {
        Self {
            users: Vec::new(),
            list_state: ListState::default(),
            processing_id: None,
            should_quit: false,
            search_query: String::new(),
            search_mode: false,
        }
    }

    pub fn filtered_users(&self) -> Vec<&(VestibuleUserRecord, DiscordMessage)> {
        if self.search_query.is_empty() {
            self.users.iter().collect()
        } else {
            self.users
                .iter()
                .filter(|(u, _)| {
                    u.discord_username.contains(&self.search_query)
                        || u.discord_user_id.contains(&self.search_query)
                })
                .collect()
        }
    }

    pub fn next(&mut self) {
        let filtered = self.filtered_users();
        if filtered.is_empty() {
            return;
        }
        let i = match self.list_state.selected() {
            Some(i) => {
                if i >= filtered.len() - 1 {
                    0
                } else {
                    i + 1
                }
            }
            None => 0,
        };
        self.list_state.select(Some(i));
    }

    pub fn previous(&mut self) {
        let filtered = self.filtered_users();
        if filtered.is_empty() {
            return;
        }
        let i = match self.list_state.selected() {
            Some(i) => {
                if i == 0 {
                    filtered.len() - 1
                } else {
                    i - 1
                }
            }
            None => 0,
        };
        self.list_state.select(Some(i));
    }

    pub fn handle_event(&mut self, event: AppEvent) {
        match event {
            AppEvent::Init(mut users) => {
                // Sort pending at the top
                users.sort_by(|a, b| {
                    let a_score = if a.0.status == RecordStatus::Pending {
                        0
                    } else {
                        1
                    };
                    let b_score = if b.0.status == RecordStatus::Pending {
                        0
                    } else {
                        1
                    };
                    a_score.cmp(&b_score)
                });
                self.users = users;
                if !self.users.is_empty() {
                    self.list_state.select(Some(0));
                }
            }
            AppEvent::NewPending(new_users) => {
                for nu in new_users {
                    if !self
                        .users
                        .iter()
                        .any(|(u, _)| u.discord_user_id == nu.0.discord_user_id)
                    {
                        self.users.insert(0, nu);
                    }
                }
            }
            AppEvent::Processing(id) => {
                self.processing_id = Some(id);
            }
            AppEvent::Scored(record, msg) => {
                self.processing_id = None;
                if let Some(pos) = self
                    .users
                    .iter()
                    .position(|(u, _)| u.discord_user_id == record.discord_user_id)
                {
                    self.users[pos] = (record, msg);
                } else {
                    self.users.push((record, msg));
                }
            }
            AppEvent::Input(key) => {
                if self.search_mode {
                    match key {
                        KeyCode::Char(c) => {
                            self.search_query.push(c);
                            self.list_state.select(Some(0)); // Reset selection to prevent out of bounds
                        }
                        KeyCode::Backspace => {
                            self.search_query.pop();
                            self.list_state.select(Some(0));
                        }
                        KeyCode::Esc | KeyCode::Enter => {
                            self.search_mode = false;
                        }
                        _ => {}
                    }
                } else {
                    match key {
                        KeyCode::Char('q') => self.should_quit = true,
                        KeyCode::Char('/') => self.search_mode = true,
                        KeyCode::Esc => {
                            self.search_query.clear();
                            self.list_state.select(Some(0));
                        }
                        KeyCode::Down | KeyCode::Char('j') => self.next(),
                        KeyCode::Up | KeyCode::Char('k') => self.previous(),
                        _ => {}
                    }
                }
            }
            AppEvent::Tick => {}
        }
    }
}

pub fn draw_ui(f: &mut ratatui::Frame, app: &mut App) {
    let size = f.size();

    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Percentage(75), Constraint::Percentage(25)].as_ref())
        .split(size);

    let top_chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints(
            [
                Constraint::Percentage(25),
                Constraint::Percentage(45),
                Constraint::Percentage(30),
            ]
            .as_ref(),
        )
        .split(chunks[0]);

    let bottom_chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Percentage(80), Constraint::Percentage(20)].as_ref())
        .split(chunks[1]);

    let filtered_users: Vec<&(VestibuleUserRecord, DiscordMessage)> = if app.search_query.is_empty()
    {
        app.users.iter().collect()
    } else {
        app.users
            .iter()
            .filter(|(u, _)| {
                u.discord_username.contains(&app.search_query)
                    || u.discord_user_id.contains(&app.search_query)
            })
            .collect()
    };

    let items: Vec<ListItem> = filtered_users
        .iter()
        .map(|(u, _)| {
            let is_processing = app.processing_id.as_deref() == Some(&u.discord_user_id);
            let color = if is_processing {
                Color::Yellow
            } else if u.status == RecordStatus::Scored {
                Color::Green
            } else {
                Color::White
            };

            let symbol = if is_processing {
                "⟳ "
            } else if u.status == RecordStatus::Scored {
                "✓ "
            } else {
                "⏳ "
            };

            ListItem::new(format!("{}{}", symbol, u.discord_username))
                .style(Style::default().fg(color))
        })
        .collect();

    let list_title = if app.search_mode {
        format!(" Search: {}█ ", app.search_query)
    } else if !app.search_query.is_empty() {
        format!(" Users (Filtered: {}) [/] ", app.search_query)
    } else {
        " Users [/] ".to_string()
    };

    let users_list = List::new(items)
        .block(Block::default().title(list_title).borders(Borders::ALL))
        .highlight_style(
            Style::default()
                .bg(Color::DarkGray)
                .add_modifier(Modifier::BOLD),
        );

    f.render_stateful_widget(users_list, top_chunks[0], &mut app.list_state);

    // 2. Middle Pane (Intro Text)
    let intro_text = if let Some(i) = app.list_state.selected() {
        if let Some((_, msg)) = filtered_users.get(i) {
            msg.content.clone()
        } else {
            "Select a user...".to_string()
        }
    } else {
        "Select a user...".to_string()
    };

    let intro_para = Paragraph::new(intro_text)
        .wrap(Wrap { trim: true })
        .block(Block::default().title(" Intro Text ").borders(Borders::ALL));
    f.render_widget(intro_para, top_chunks[1]);

    // 3. Right Pane (Traits)
    let traits_text = if let Some(i) = app.list_state.selected() {
        if let Some((u, _)) = filtered_users.get(i) {
            if u.status == RecordStatus::Scored {
                format!(
                    "=== HEXACO TRAITS ===\n\
                     Honesty-Humility:   {:.2}\n\
                     Emotionality:       {:.2}\n\
                     Extraversion:       {:.2}\n\
                     Agreeableness:      {:.2}\n\
                     Conscientiousness:  {:.2}\n\
                     Openness:           {:.2}\n\n\
                     === COMMUNICATION ===\n\
                     Agency:             {:.2}\n\
                     Communion:          {:.2}\n\n\
                     === VALUES ===\n\
                     Self-Direction:     {:.2}\n\
                     Stimulation:        {:.2}\n\
                     Hedonism:           {:.2}\n\
                     Achievement:        {:.2}\n\
                     Power:              {:.2}\n\
                     Security:           {:.2}\n\
                     Conformity:         {:.2}\n\
                     Tradition:          {:.2}\n\
                     Benevolence:        {:.2}\n\
                     Universalism:       {:.2}\n\n\
                     === INTERESTS ===\n\
                     Domains: {}\n\
                     Activities: {}",
                    u.personality.honesty_humility,
                    u.personality.emotionality,
                    u.personality.extraversion,
                    u.personality.agreeableness,
                    u.personality.conscientiousness,
                    u.personality.openness_to_experience,
                    u.communication.agency,
                    u.communication.communion,
                    u.values.self_direction,
                    u.values.stimulation,
                    u.values.hedonism,
                    u.values.achievement,
                    u.values.power,
                    u.values.security,
                    u.values.conformity,
                    u.values.tradition,
                    u.values.benevolence,
                    u.values.universalism,
                    u.interests.domains.join(", "),
                    u.interests.activities.join(", ")
                )
            } else {
                "Awaiting Processing...".to_string()
            }
        } else {
            "".to_string()
        }
    } else {
        "".to_string()
    };

    let traits_para = Paragraph::new(traits_text).wrap(Wrap { trim: true }).block(
        Block::default()
            .title(" Traits & Grading ")
            .borders(Borders::ALL),
    );
    f.render_widget(traits_para, top_chunks[2]);

    // 4. Log Pane
    let tui_sm = TuiLoggerWidget::default()
        .block(Block::default().title(" Logs ").borders(Borders::ALL))
        .style_error(Style::default().fg(Color::Red))
        .style_debug(Style::default().fg(Color::Green))
        .style_warn(Style::default().fg(Color::Yellow))
        .style_trace(Style::default().fg(Color::Magenta))
        .style_info(Style::default().fg(Color::Cyan))
        .output_separator(':')
        .output_timestamp(Some("%H:%M:%S".to_string()))
        .output_level(Some(TuiLoggerLevelOutput::Abbreviated))
        .output_target(false)
        .output_file(false)
        .output_line(false);
    f.render_widget(tui_sm, bottom_chunks[0]);

    // 5. Progress Bar
    let total = app.users.len();
    let scored = app
        .users
        .iter()
        .filter(|(u, _)| u.status == RecordStatus::Scored)
        .count();
    let percent = if total > 0 {
        ((scored as f64 / total as f64) * 100.0).round() as u16
    } else {
        0
    };

    let label = format!("Processed: {} / Total: {} ({}%)", scored, total, percent);
    let gauge = Gauge::default()
        .block(Block::default().title(" Progress ").borders(Borders::ALL))
        .gauge_style(Style::default().fg(Color::Cyan).bg(Color::DarkGray))
        .percent(percent)
        .label(Span::styled(
            label,
            Style::default().add_modifier(Modifier::BOLD),
        ));

    f.render_widget(gauge, bottom_chunks[1]);
}
