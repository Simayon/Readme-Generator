use std::fs;
use std::io;

use crossterm::{
    event::{self, DisableMouseCapture, EnableMouseCapture, Event, KeyCode},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use ratatui::{
    backend::CrosstermBackend,
    layout::{Alignment, Constraint, Direction, Layout},
    style::{Color, Modifier, Style},
    text::{Span, Spans, Text},
    widgets::{Block, Borders, List, ListItem, Paragraph, Wrap},
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
                    description: String::from("The name of your project/repository (e.g., username/repo)"),
                },
                Field {
                    name: String::from("Project Title"),
                    value: String::new(),
                    description: String::from("A catchy title for your project"),
                },
                Field {
                    name: String::from("Short Description"),
                    value: String::new(),
                    description: String::from("A brief one-line description of your project"),
                },
                Field {
                    name: String::from("Detailed Description"),
                    value: String::new(),
                    description: String::from("A detailed explanation of what your project does and why it's useful"),
                },
                Field {
                    name: String::from("Features"),
                    value: String::new(),
                    description: String::from("Key features of your project (separate with semicolons)"),
                },
                Field {
                    name: String::from("Technologies"),
                    value: String::new(),
                    description: String::from("Technologies used (separate with semicolons) e.g., React;TypeScript;Node.js"),
                },
                Field {
                    name: String::from("Prerequisites"),
                    value: String::new(),
                    description: String::from("Required software/tools to run your project (separate with semicolons)"),
                },
                Field {
                    name: String::from("Installation"),
                    value: String::new(),
                    description: String::from("Step-by-step installation instructions (separate steps with semicolons)"),
                },
                Field {
                    name: String::from("Usage Example"),
                    value: String::new(),
                    description: String::from("Example code or commands to use your project"),
                },
                Field {
                    name: String::from("API Documentation"),
                    value: String::new(),
                    description: String::from("Brief API documentation or endpoints (optional)"),
                },
                Field {
                    name: String::from("Contributing Guidelines"),
                    value: String::new(),
                    description: String::from("How others can contribute to your project"),
                },
                Field {
                    name: String::from("Tests"),
                    value: String::new(),
                    description: String::from("How to run tests (separate steps with semicolons)"),
                },
                Field {
                    name: String::from("Authors"),
                    value: String::new(),
                    description: String::from("Project authors/maintainers (separate with semicolons)"),
                },
            ],
            current_field: 0,
            license_options: vec![
                String::from("MIT License"),
                String::from("Apache License 2.0"),
                String::from("GNU GPL v3"),
                String::from("BSD 3-Clause"),
                String::from("ISC License"),
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
        let repo_name = &self.fields[0].value;
        let project_title = &self.fields[1].value;
        let short_desc = &self.fields[2].value;
        let detailed_desc = &self.fields[3].value;
        let features = self.fields[4].value.split(';').collect::<Vec<_>>();
        let technologies = self.fields[5].value.split(';').collect::<Vec<_>>();
        let prerequisites = self.fields[6].value.split(';').collect::<Vec<_>>();
        let installation = self.fields[7].value.split(';').collect::<Vec<_>>();
        let usage = &self.fields[8].value;
        let api_docs = &self.fields[9].value;
        let contributing = &self.fields[10].value;
        let tests = self.fields[11].value.split(';').collect::<Vec<_>>();
        let authors = self.fields[12].value.split(';').collect::<Vec<_>>();
        let license = &self.license_options[self.selected_license];

        let mut badges = Vec::new();
        if !repo_name.is_empty() {
            badges.push(format!("[![Stars](https://img.shields.io/github/stars/{repo_name}?style=flat-square)](https://github.com/{repo_name}/stargazers)"));
            badges.push(format!("[![Forks](https://img.shields.io/github/forks/{repo_name}?style=flat-square)](https://github.com/{repo_name}/network/members)"));
            badges.push(format!("[![Issues](https://img.shields.io/github/issues/{repo_name}?style=flat-square)](https://github.com/{repo_name}/issues)"));
            badges.push(format!("[![License](https://img.shields.io/github/license/{repo_name}?style=flat-square)](https://github.com/{repo_name}/blob/main/LICENSE)"));
        }

        let tech_badges: Vec<String> = technologies.iter()
            .filter(|&t| !t.is_empty())
            .map(|tech| {
                let tech = tech.trim().to_lowercase();
                format!("![{}](https://img.shields.io/badge/-{}-informational?style=flat-square&logo={}&logoColor=white)",
                    tech, tech, tech)
            })
            .collect();

        let features_list = if features.is_empty() || features[0].is_empty() {
            String::from("- <Features of your project>")
        } else {
            features.iter()
                .map(|f| format!("- {}", f.trim()))
                .collect::<Vec<_>>()
                .join("\n")
        };

        let prereq_list = if prerequisites.is_empty() || prerequisites[0].is_empty() {
            String::from("- <Prerequisites>")
        } else {
            prerequisites.iter()
                .map(|p| format!("- {}", p.trim()))
                .collect::<Vec<_>>()
                .join("\n")
        };

        let install_steps = if installation.is_empty() || installation[0].is_empty() {
            String::from("1. <Installation steps>")
        } else {
            installation.iter()
                .enumerate()
                .map(|(i, step)| format!("{}. {}", i + 1, step.trim()))
                .collect::<Vec<_>>()
                .join("\n")
        };

        let test_steps = if tests.is_empty() || tests[0].is_empty() {
            String::from("1. <Test instructions>")
        } else {
            tests.iter()
                .enumerate()
                .map(|(i, step)| format!("{}. {}", i + 1, step.trim()))
                .collect::<Vec<_>>()
                .join("\n")
        };

        format!(
r#"<div align="center">

# {}

{}

{}

[Documentation](#{}) · [Report Bug](https://github.com/{}/issues) · [Request Feature](https://github.com/{}/issues)

{}</div>

## 📋 Table of Contents
- [About](#about)
- [Features](#features)
- [Built With](#built-with)
- [Getting Started](#getting-started)
  - [Prerequisites](#prerequisites)
  - [Installation](#installation)
- [Usage](#usage)
- [API Documentation](#api-documentation)
- [Testing](#testing)
- [Contributing](#contributing)
- [License](#license)
- [Contact](#contact)

## 🔍 About
{}

## ✨ Features
{}

## 🛠️ Built With
{}

## 🚀 Getting Started

### Prerequisites
{}

### Installation
{}

## 💡 Usage
```bash
{}
```

## 📚 API Documentation
```
{}
```

## 🧪 Testing
{}

## 🤝 Contributing
{}

## 📝 License
This project is licensed under the {} - see the [LICENSE](LICENSE) file for details.

## 👥 Authors
{}

---
<div align="center">
Made with ❤️ by contributors
</div>"#,
            // Title and badges section
            project_title,
            short_desc,
            badges.join("\n"),
            repo_name,
            repo_name,
            repo_name,
            if !tech_badges.is_empty() { tech_badges.join(" ") } else { String::from("<Technology badges>") },
            // Main content
            detailed_desc,
            features_list,
            if technologies.is_empty() || technologies[0].is_empty() {
                String::from("- <Technologies used>")
            } else {
                technologies.iter()
                    .map(|t| format!("- {}", t.trim()))
                    .collect::<Vec<_>>()
                    .join("\n")
            },
            prereq_list,
            install_steps,
            usage,
            api_docs,
            test_steps,
            contributing,
            license,
            if authors.is_empty() || authors[0].is_empty() {
                String::from("- <Project authors>")
            } else {
                authors.iter()
                    .map(|a| format!("- {}", a.trim()))
                    .collect::<Vec<_>>()
                    .join("\n")
            }
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
    let markdown_content = format!(
        r#"<div align="center">

# {}

{}

[![Stars](https://img.shields.io/github/stars/{}?style=flat-square)](https://github.com/{}/stargazers)
[![Forks](https://img.shields.io/github/forks/{}?style=flat-square)](https://github.com/{}/network/members)
[![Issues](https://img.shields.io/github/issues/{}?style=flat-square)](https://github.com/{}/issues)
[![License](https://img.shields.io/github/license/{}?style=flat-square)](https://github.com/{}/blob/main/LICENSE)

[Documentation](#{}) · [Report Bug](https://github.com/{}/issues) · [Request Feature](https://github.com/{}/issues)

</div>

## 📋 Table of Contents
- [About](#about)
- [Features](#features)
- [Built With](#built-with)
- [Getting Started](#getting-started)
  - [Prerequisites](#prerequisites)
  - [Installation](#installation)
- [Usage](#usage)
- [API Documentation](#api-documentation)
- [Testing](#testing)
- [Contributing](#contributing)
- [License](#license)
- [Contact](#contact)

## 🔍 About
{}

## ✨ Features
{}

## 🛠️ Built With
{}

## 🚀 Getting Started

### Prerequisites
{}

### Installation
{}

## 💡 Usage
```bash
{}
```

## 📚 API Documentation
```
{}
```

## 🧪 Testing
{}

## 🤝 Contributing
{}

## 📝 License
This project is licensed under the {} - see the [LICENSE](LICENSE) file for details.

## 👥 Authors
{}

---
<div align="center">
Made with ❤️ by contributors
</div>"#,
        app.fields[1].value,  // project_title (1)
        app.fields[2].value,  // short_description (2)
        app.fields[0].value,  // repository_name (3)
        app.fields[0].value,  // repository_name (4)
        app.fields[0].value,  // repository_name (5)
        app.fields[0].value,  // repository_name (6)
        app.fields[0].value,  // repository_name (7)
        app.fields[0].value,  // repository_name (8)
        app.fields[0].value,  // repository_name (9)
        app.fields[0].value,  // repository_name (10)
        app.fields[0].value,  // repository_name (11)
        app.fields[0].value,  // repository_name (12)
        app.fields[0].value,  // repository_name (13)
        app.fields[3].value,  // detailed_description (14)
        app.fields[4].value,  // features (15)
        app.fields[5].value,  // technologies (16)
        app.fields[6].value,  // prerequisites (17)
        app.fields[7].value,  // installation (18)
        app.fields[8].value,  // usage (19)
        app.fields[9].value,  // api_docs (20)
        app.fields[11].value, // tests (21)
        app.fields[10].value, // contributing (22)
        app.license_options[app.selected_license], // license (23)
        app.fields[12].value  // authors (24)
    );

    fs::write("README.md", markdown_content).expect("Unable to write file");
}
