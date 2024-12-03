use std::fs;
use std::io;

use crossterm::{
    event::{self, DisableMouseCapture, EnableMouseCapture, Event, KeyCode},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use ratatui::{
    backend::CrosstermBackend,
    layout::{Alignment, Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Span, Spans, Text},
    widgets::{Block, Borders, List, ListItem, Paragraph, Tabs, Wrap},
    Frame, Terminal,
};

#[derive(PartialEq)]
enum InputMode {
    Navigation,
    Editing,
}

struct Field {
    name: String,
    value: String,
    description: String,
}

struct App {
    input: String,
    input_mode: InputMode,
    fields: Vec<Field>,
    current_field: usize,
    license_options: Vec<String>,
    selected_license: usize,
}

impl Default for App {
    fn default() -> App {
        App {
            input: String::new(),
            input_mode: InputMode::Navigation,
            fields: vec![
                Field {
                    name: String::from("Repository Name"),
                    value: String::new(),
                    description: String::from("The name of your project/repository"),
                },
                Field {
                    name: String::from("Project Description"),
                    value: String::new(),
                    description: String::from("A brief description of what your project does"),
                },
                Field {
                    name: String::from("Installation"),
                    value: String::new(),
                    description: String::from("Steps required to install your project"),
                },
                Field {
                    name: String::from("Usage"),
                    value: String::new(),
                    description: String::from("How to use your project, with examples"),
                },
                Field {
                    name: String::from("Contributors"),
                    value: String::new(),
                    description: String::from("List of contributors and how to contribute"),
                },
            ],
            current_field: 0,
            license_options: vec![
                String::from("MIT License"),
                String::from("Apache License 2.0"),
                String::from("GNU General Public License v3.0"),
            ],
            selected_license: 0,
        }
    }
}

impl App {
    fn all_fields_filled(&self) -> bool {
        self.fields.iter().all(|field| !field.value.is_empty())
    }

    fn generate_preview(&self) -> String {
        let repository_name = &self.fields[0].value;
        let project_description = &self.fields[1].value;
        let installation = &self.fields[2].value;
        let usage = &self.fields[3].value;
        let contributors = &self.fields[4].value;
        let license = &self.license_options[self.selected_license];

        let stars_badge = if !repository_name.is_empty() {
            format!("[![GitHub stars](https://img.shields.io/github/stars/{repository_name})](https://github.com/{repository_name}/stargazers)")
        } else {
            String::new()
        };

        format!(
            "# {}\n{}\n\n{}\n\n## Installation\n```\n{}\n```\n\n## Usage\n```\n{}\n```\n\n## Contributors\n{}\n\n## License\nThis project is licensed under the {} - see the LICENSE file for details.",
            if repository_name.is_empty() { "<Repository Name>" } else { repository_name },
            if project_description.is_empty() { "<Project Description>" } else { project_description },
            if !repository_name.is_empty() { &stars_badge } else { "" },
            if installation.is_empty() { "<Installation Instructions>" } else { installation },
            if usage.is_empty() { "<Usage Instructions>" } else { usage },
            if contributors.is_empty() { "<Contributors>" } else { contributors },
            license
        )
    }
}

fn main() -> Result<(), io::Error> {
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen, EnableMouseCapture)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    let mut app = App::default();
    let res = run_app(&mut terminal, &mut app);

    disable_raw_mode()?;
    execute!(
        terminal.backend_mut(),
        LeaveAlternateScreen,
        DisableMouseCapture
    )?;
    terminal.show_cursor()?;

    if let Ok(true) = res {
        generate_readme(&app);
    }

    Ok(())
}

fn run_app<B: ratatui::backend::Backend>(
    terminal: &mut Terminal<B>,
    app: &mut App,
) -> io::Result<bool> {
    loop {
        terminal.draw(|f| ui(f, app))?;

        if let Event::Key(key) = event::read()? {
            match app.input_mode {
                InputMode::Navigation => match key.code {
                    KeyCode::Char('q') => {
                        return Ok(false);
                    }
                    KeyCode::Down => {
                        if app.current_field < app.fields.len() - 1 {
                            app.current_field += 1;
                        }
                    }
                    KeyCode::Up => {
                        if app.current_field > 0 {
                            app.current_field -= 1;
                        }
                    }
                    KeyCode::Enter => {
                        app.input_mode = InputMode::Editing;
                        app.input = app.fields[app.current_field].value.clone();
                    }
                    KeyCode::Tab => {
                        if app.all_fields_filled() {
                            return Ok(true);
                        }
                    }
                    _ => {}
                },
                InputMode::Editing => match key.code {
                    KeyCode::Enter => {
                        app.fields[app.current_field].value = app.input.drain(..).collect();
                        if app.current_field < app.fields.len() - 1 {
                            app.current_field += 1;
                            app.input = app.fields[app.current_field].value.clone();
                        } else {
                            app.input_mode = InputMode::Navigation;
                        }
                    }
                    KeyCode::Char(c) => {
                        app.input.push(c);
                    }
                    KeyCode::Backspace => {
                        app.input.pop();
                    }
                    KeyCode::Esc => {
                        app.input.clear();
                        app.input_mode = InputMode::Navigation;
                    }
                    _ => {}
                },
            }
        }
    }
}

fn ui<B: ratatui::backend::Backend>(f: &mut Frame<B>, app: &App) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .margin(2)
        .constraints(
            [
                Constraint::Length(3),  // Help text
                Constraint::Length(3),  // Progress bar
                Constraint::Min(10),    // Main content
                Constraint::Length(3),  // Input field
            ]
            .as_ref(),
        )
        .split(f.size());

    // Help message
    let (msg, style) = match app.input_mode {
        InputMode::Navigation => (
            vec![
                Span::raw("Press "),
                Span::styled("↑↓", Style::default().add_modifier(Modifier::BOLD)),
                Span::raw(" to move, "),
                Span::styled("Enter", Style::default().add_modifier(Modifier::BOLD)),
                Span::raw(" to edit, "),
                Span::styled("Tab", Style::default().add_modifier(Modifier::BOLD)),
                Span::raw(" for license, "),
                Span::styled("q", Style::default().add_modifier(Modifier::BOLD)),
                Span::raw(" to quit"),
            ],
            Style::default(),
        ),
        InputMode::Editing => (
            vec![
                Span::raw("Press "),
                Span::styled("Enter", Style::default().add_modifier(Modifier::BOLD)),
                Span::raw(" to save and continue, "),
                Span::styled("Esc", Style::default().add_modifier(Modifier::BOLD)),
                Span::raw(" to cancel"),
            ],
            Style::default(),
        ),
    };
    let mut text = Text::from(Spans::from(msg));
    text.patch_style(style);
    let help_message = Paragraph::new(text)
        .block(Block::default().borders(Borders::ALL).title("Help"));
    f.render_widget(help_message, chunks[0]);

    // Progress indicator
    let progress: Vec<Span> = app
        .fields
        .iter()
        .enumerate()
        .map(|(i, field)| {
            if i == app.current_field {
                Span::styled(
                    "●",
                    Style::default().fg(Color::Green).add_modifier(Modifier::BOLD),
                )
            } else if !field.value.is_empty() {
                Span::styled("●", Style::default().fg(Color::White))
            } else {
                Span::styled("○", Style::default().fg(Color::DarkGray))
            }
        })
        .collect();
    let progress_text = Spans::from(progress);
    let progress_widget = Paragraph::new(progress_text)
        .block(Block::default().borders(Borders::ALL).title("Progress"))
        .alignment(Alignment::Center);
    f.render_widget(progress_widget, chunks[1]);

    // Main content area
    let main_chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Percentage(50), Constraint::Percentage(50)].as_ref())
        .split(chunks[2]);

    // Fields list
    let fields: Vec<ListItem> = app
        .fields
        .iter()
        .enumerate()
        .map(|(i, field)| {
            let style = if i == app.current_field {
                Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD)
            } else {
                Style::default()
            };
            let header = Spans::from(vec![
                Span::styled(&field.name, style),
                Span::raw(": "),
                Span::styled(
                    if field.value.is_empty() {
                        "<empty>"
                    } else {
                        &field.value
                    },
                    if field.value.is_empty() {
                        Style::default().fg(Color::DarkGray)
                    } else {
                        Style::default()
                    },
                ),
            ]);
            ListItem::new(vec![header])
        })
        .collect();

    let fields_list = List::new(fields).block(
        Block::default()
            .borders(Borders::ALL)
            .title("README Sections"),
    );
    f.render_widget(fields_list, main_chunks[0]);

    // Right panel: Description or Preview
    let right_panel = if app.all_fields_filled() {
        // Show preview when all fields are filled
        Paragraph::new(app.generate_preview())
            .block(Block::default().borders(Borders::ALL).title("README Preview"))
            .wrap(Wrap { trim: true })
    } else {
        // Show field description when fields are being filled
        let current_field = &app.fields[app.current_field];
        Paragraph::new(current_field.description.as_ref())
            .block(Block::default().borders(Borders::ALL).title("Description"))
            .wrap(Wrap { trim: true })
    };
    f.render_widget(right_panel, main_chunks[1]);

    // Input field
    let input = Paragraph::new(app.input.as_ref())
        .style(match app.input_mode {
            InputMode::Navigation => Style::default(),
            InputMode::Editing => Style::default().fg(Color::Yellow),
        })
        .block(
            Block::default()
                .borders(Borders::ALL)
                .title(format!("Editing: {}", app.fields[app.current_field].name)),
        );
    f.render_widget(input, chunks[3]);

    if app.input_mode == InputMode::Editing {
        f.set_cursor(
            chunks[3].x + app.input.len() as u16 + 1,
            chunks[3].y + 1,
        );
    }
}

fn generate_readme(app: &App) {
    let repository_name = &app.fields[0].value;
    let project_description = &app.fields[1].value;
    let installation = &app.fields[2].value;
    let usage = &app.fields[3].value;
    let contributors = &app.fields[4].value;
    let license = &app.license_options[app.selected_license];

    let stars_badge = format!("[![GitHub stars](https://img.shields.io/github/stars/{repository_name})](https://github.com/{repository_name}/stargazers)");
    let forks_badge = format!("[![GitHub forks](https://img.shields.io/github/forks/{repository_name})](https://github.com/{repository_name}/network/members)");
    let issues_badge = format!("[![GitHub issues](https://img.shields.io/github/issues/{repository_name})](https://github.com/{repository_name}/issues)");
    let license_badge = format!("[![GitHub license](https://img.shields.io/github/license/{repository_name})](https://github.com/{repository_name}/blob/master/LICENSE)");

    let markdown_content = format!(
        r#"# {repository_name}
{project_description}

## Table of Contents
- [Installation](#installation)
- [Usage](#usage)
- [Contributors](#contributors)
- [License](#license)
- [Badges](#badges)
- [GitHub Repository](#github-repository)

## Installation
```
{installation}
```

## Usage
```
{usage}
```

## Contributors
{contributors}

## License
This project is licensed under the {license} - see the [LICENSE](LICENSE) file for details.

## Badges
{stars_badge} {forks_badge} {issues_badge} {license_badge}

## GitHub Repository
[Link to GitHub repository](https://github.com/{repository_name})
"#
    );

    fs::write("README.md", markdown_content).expect("Unable to write file");
    println!("README.md generated successfully!");
}
