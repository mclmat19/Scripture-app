use ratatui::{
    Frame,
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span, Text},
    widgets::{Block, Borders, BorderType, List, ListItem, ListState, Paragraph, Wrap, Clear},
};

use crate::app::{App, Mode, Pane, TreeItem};

const BASE: Color      = Color::Rgb(245, 245, 245);
const MANTLE: Color    = Color::Rgb(230, 230, 230);
const SURFACE0: Color  = Color::Rgb(200, 200, 200);
const SURFACE1: Color  = Color::Rgb(184, 184, 184);
const SURFACE2: Color  = Color::Rgb(170, 170, 170);
const TEXT: Color      = Color::Rgb(26, 26, 26);
const SUBTEXT1: Color  = Color::Rgb(51, 51, 51);
const SUBTEXT0: Color  = Color::Rgb(85, 85, 85);
const OVERLAY2: Color  = Color::Rgb(119, 119, 119);
const BLUE: Color      = Color::Rgb(58, 58, 58);
const MAUVE: Color     = Color::Rgb(68, 68, 68);
const TEAL: Color      = Color::Rgb(43, 43, 43);
const SKY: Color       = Color::Rgb(102, 102, 102);
const PEACH: Color     = Color::Rgb(136, 136, 136);
const RED: Color       = Color::Rgb(85, 85, 85);
const GREEN: Color     = Color::Rgb(51, 51, 51);
const YELLOW: Color    = Color::Rgb(153, 153, 153);

pub fn render(f: &mut Frame, app: &App) {
    let area = f.area();

    if app.mode == Mode::Menu {
        render_menu(f, app, area);
        return;
    }

    f.render_widget(
        Block::default().style(Style::default().bg(BASE)),
        area,
    );

    let root = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Min(0), Constraint::Length(1)])
        .split(area);

    let main = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Length(28), Constraint::Min(0)])
        .split(root[0]);

    render_tree(f, app, main[0]);
    render_reader(f, app, main[1]);
    render_statusbar(f, app, root[1]);

    if app.mode == Mode::Search {
        render_search(f, app, area);
    }
}

fn render_tree(f: &mut Frame, app: &App, area: Rect) {
    let is_active = app.active_pane == Pane::Tree;
    let border_color = if is_active { BLUE } else { SURFACE1 };

    let block = Block::default()
        .title(Line::from(vec![
            Span::styled(" ", Style::default()),
            Span::styled("Bible ", Style::default().fg(TEXT).add_modifier(Modifier::BOLD)),
        ]))
        .borders(Borders::ALL)
        .border_type(BorderType::Rounded)
        .border_style(Style::default().fg(border_color))
        .style(Style::default().bg(MANTLE));

    let inner = block.inner(area);
    f.render_widget(block, area);

    let items = app.flat_tree();
    let mut list_items = vec![];
    let mut list_state = ListState::default();
    list_state.select(Some(app.tree_selected));

    for (idx, item) in items.iter().enumerate() {
        let row = match item {
            TreeItem::Testament(name) => {
                ListItem::new(Line::from(vec![
                    Span::styled(
                        format!("  {}", name.to_uppercase()),
                        Style::default()
                            .fg(OVERLAY2)
                            .add_modifier(Modifier::BOLD),
                    ),
                ]))
            }
            TreeItem::Book { book_idx, expanded } => {
                let book = &app.books[*book_idx];
                let arrow = if *expanded { "▼" } else { "▶" };
                let icon_color = if book.name.len() > 5 { PEACH } else { MAUVE };
                let selected = idx == app.tree_selected;
                let style = if selected {
                    Style::default().fg(BLUE).bg(SURFACE0).add_modifier(Modifier::BOLD)
                } else {
                    Style::default().fg(TEXT)
                };
                ListItem::new(Line::from(vec![
                    Span::styled(format!(" {} ", arrow), Style::default().fg(OVERLAY2)),
                    Span::styled("📖 ", Style::default().fg(icon_color)),
                    Span::styled(book.name.clone(), style),
                    Span::styled(
                        format!(" {}", book.chapter_count()),
                        Style::default().fg(SURFACE2),
                    ),
                ]))
            }
            TreeItem::Chapter { book_idx, chapter_idx } => {
                let book = &app.books[*book_idx];
                let chapter = &book.chapters[*chapter_idx];
                let selected = idx == app.tree_selected;
                let is_open = *book_idx == app.selected_book
                    && *chapter_idx == app.selected_chapter;

                let connector = if *chapter_idx + 1 < book.chapters.len() { "│" } else { "└" };
                let name_color = if is_open { TEAL } else { SUBTEXT0 };
                let style = if selected {
                    Style::default().fg(MAUVE).bg(SURFACE0)
                } else {
                    Style::default().fg(name_color)
                };

                ListItem::new(Line::from(vec![
                    Span::styled(format!("   {} ", connector), Style::default().fg(SURFACE2)),
                    Span::styled(format!("Ch {}", chapter.number), style),
                ]))
            }
        };
        list_items.push(row);
    }

    let list = List::new(list_items)
        .highlight_style(Style::default())
        .style(Style::default().bg(MANTLE));

    f.render_stateful_widget(list, inner, &mut list_state);
}

fn render_reader(f: &mut Frame, app: &App, area: Rect) {
    let is_active = app.active_pane == Pane::Reader;
    let border_color = if is_active { MAUVE } else { SURFACE1 };

    let book = match app.books.get(app.selected_book) {
        Some(b) => b,
        None => return,
    };
    let chapter = match book.chapters.get(app.selected_chapter) {
        Some(c) => c,
        None => return,
    };

    let title_str = format!(" {} — Chapter {} ", book.name, chapter.number);
    let block = Block::default()
        .title(Line::from(vec![
            Span::styled(title_str, Style::default().fg(MAUVE).add_modifier(Modifier::BOLD)),
        ]))
        .borders(Borders::ALL)
        .border_type(BorderType::Rounded)
        .border_style(Style::default().fg(border_color))
        .style(Style::default().bg(BASE));

    let inner = block.inner(area);
    f.render_widget(block, area);

    let layout = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Length(2), Constraint::Min(0)])
        .split(inner);

    let heading = Paragraph::new(Line::from(vec![
        Span::styled(
            format!("{}, Chapter {}", book.name, chapter.number),
            Style::default().fg(BLUE).add_modifier(Modifier::BOLD | Modifier::ITALIC),
        ),
    ]))
    .style(Style::default().bg(BASE));
    f.render_widget(heading, layout[0]);

    let visible_area = layout[1];
    let height = visible_area.height as usize;
    let mut lines: Vec<Line> = vec![];

    for (vi, verse) in chapter.verses.iter().enumerate() {
        let is_selected = vi == app.selected_verse;

        let verse_num = Span::styled(
            format!("{:>3} ", verse.number),
            Style::default().fg(RED).add_modifier(Modifier::BOLD),
        );

        let text_style = if is_selected {
            Style::default().fg(TEXT).bg(SURFACE0)
        } else {
            Style::default().fg(SUBTEXT1)
        };

        let verse_text = Span::styled(verse.text.clone(), text_style);
        lines.push(Line::from(vec![verse_num, verse_text]));
        lines.push(Line::from(""));
    }

    let visible_lines: Vec<Line> = lines
        .into_iter()
        .skip(app.scroll_offset * 2)
        .take(height)
        .collect();

    let para = Paragraph::new(Text::from(visible_lines))
        .wrap(Wrap { trim: false })
        .style(Style::default().bg(BASE));

    f.render_widget(para, visible_area);
}

fn render_statusbar(f: &mut Frame, app: &App, area: Rect) {
    if app.mode == Mode::Command {
        f.render_widget(
            Paragraph::new(Line::from(vec![
                Span::styled(":", Style::default().fg(TEXT).add_modifier(Modifier::BOLD)),
                Span::styled(app.command_input.clone(), Style::default().fg(TEXT)),
                Span::styled("█", Style::default().fg(BLUE)),
            ])).style(Style::default().bg(MANTLE)),
            area,
        );
        return;
    }

    let book_name = app.current_book_name();
    let chapter_num = app.current_chapter_num();

    let mode_label = match app.mode {
        Mode::Menu    => "MENU",
        Mode::Normal  => "NORMAL",
        Mode::Search  => "SEARCH",
        Mode::Command => "COMMAND",
    };
    let mode_color = match app.mode {
        Mode::Menu    => OVERLAY2,
        Mode::Normal  => BLUE,
        Mode::Search  => YELLOW,
        Mode::Command => BLUE,
    };

    let left = vec![
        Span::styled(
            format!(" {} ", mode_label),
            Style::default().fg(MANTLE).bg(mode_color).add_modifier(Modifier::BOLD),
        ),
        Span::styled(" ", Style::default().bg(SURFACE0).fg(mode_color)),
        Span::styled(
            format!("  {} Ch.{}", book_name, chapter_num),
            Style::default().fg(TEXT).bg(SURFACE0),
        ),
        Span::styled(" ", Style::default().fg(SURFACE0).bg(BASE)),
    ];

    let right = vec![
        Span::styled(" j/k ", Style::default().fg(MANTLE).bg(GREEN).add_modifier(Modifier::BOLD)),
        Span::styled(" move  ", Style::default().fg(SUBTEXT0).bg(BASE)),
        Span::styled(" Tab ", Style::default().fg(MANTLE).bg(SKY).add_modifier(Modifier::BOLD)),
        Span::styled(" switch  ", Style::default().fg(SUBTEXT0).bg(BASE)),
        Span::styled(" / ", Style::default().fg(MANTLE).bg(PEACH).add_modifier(Modifier::BOLD)),
        Span::styled(" search  ", Style::default().fg(SUBTEXT0).bg(BASE)),
        Span::styled(" q ", Style::default().fg(MANTLE).bg(RED).add_modifier(Modifier::BOLD)),
        Span::styled(" quit ", Style::default().fg(SUBTEXT0).bg(BASE)),
    ];

    let bar_bg = Paragraph::new("").style(Style::default().bg(BASE));
    f.render_widget(bar_bg, area);

    let layout = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Min(0), Constraint::Length(50)])
        .split(area);

    f.render_widget(
        Paragraph::new(Line::from(left)).style(Style::default().bg(BASE)),
        layout[0],
    );
    f.render_widget(
        Paragraph::new(Line::from(right)).style(Style::default().bg(BASE)),
        layout[1],
    );
}

fn render_search(f: &mut Frame, app: &App, area: Rect) {
    let popup = centered_rect(60, 40, area);
    f.render_widget(Clear, popup);

    let block = Block::default()
        .title(Line::from(vec![
            Span::styled(" Search ", Style::default().fg(YELLOW).add_modifier(Modifier::BOLD)),
        ]))
        .borders(Borders::ALL)
        .border_type(BorderType::Rounded)
        .border_style(Style::default().fg(YELLOW))
        .style(Style::default().bg(MANTLE));

    let inner = block.inner(popup);
    f.render_widget(block, popup);

    let layout = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Length(1), Constraint::Length(1), Constraint::Min(0)])
        .margin(1)
        .split(inner);

    let mode_indicator = if app.search_navigating {
        Span::styled("  ↑↓ navigate ", Style::default().fg(TEAL))
    } else {
        Span::styled("  typing... ", Style::default().fg(YELLOW))
    };

    let input = Paragraph::new(Line::from(vec![
        Span::styled("/ ", Style::default().fg(YELLOW).add_modifier(Modifier::BOLD)),
        Span::styled(app.search_query.clone(), Style::default().fg(TEXT)),
        Span::styled("█", Style::default().fg(BLUE)),
        mode_indicator,
    ]));
    f.render_widget(input, layout[0]);

    let divider = Paragraph::new(Line::from(vec![
        Span::styled(
            "─".repeat(inner.width as usize - 2),
            Style::default().fg(SURFACE1),
        ),
    ]));
    f.render_widget(divider, layout[1]);

    let result_area = layout[2];
    if app.search_results.is_empty() {
        let msg = if app.search_query.is_empty() {
            "Type to search all verses..."
        } else {
            "No results found"
        };
        f.render_widget(
            Paragraph::new(Span::styled(msg, Style::default().fg(OVERLAY2))),
            result_area,
        );
        return;
    }

    let mut items: Vec<ListItem> = vec![];
    items.push(ListItem::new(Line::from(vec![
        Span::styled(
            format!(" {} result(s) ", app.search_results.len()),
            Style::default().fg(GREEN),
        ),
    ])));

    for (i, &(bi, ci, vi)) in app.search_results.iter().enumerate() {
        if let Some(book) = app.books.get(bi) {
            if let Some(chapter) = book.chapters.get(ci) {
                if let Some(verse) = chapter.verses.get(vi) {
                    let is_selected = i == app.search_cursor;
                    let ref_str = format!(" {}  {}:{} ", book.name, chapter.number, verse.number);

                    let text_preview = if verse.text.len() > 40 {
                        let cut = verse.text.char_indices()
                            .map(|(i, _)| i)
                            .nth(40)
                            .unwrap_or(verse.text.len());
                        format!("{}...", &verse.text[..cut])
                    } else {
                        verse.text.clone()
                    };

                    let style = if is_selected {
                        Style::default().fg(MAUVE).bg(SURFACE0).add_modifier(Modifier::BOLD)
                    } else {
                        Style::default().fg(SUBTEXT0)
                    };
                    items.push(ListItem::new(Line::from(vec![
                        Span::styled(ref_str, Style::default().fg(BLUE).add_modifier(Modifier::BOLD)),
                        Span::styled(text_preview, style),
                    ])));
                }
            }
        }
    }

    let mut list_state = ListState::default();
    list_state.select(Some(app.search_cursor + 1));

    let list = List::new(items)
        .highlight_style(Style::default())
        .style(Style::default().bg(MANTLE));

    f.render_stateful_widget(list, result_area, &mut list_state);
}

fn centered_rect(percent_x: u16, percent_y: u16, area: Rect) -> Rect {
    let popup_layout = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Percentage((100 - percent_y) / 2),
            Constraint::Percentage(percent_y),
            Constraint::Percentage((100 - percent_y) / 2),
        ])
        .split(area);

    Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Percentage((100 - percent_x) / 2),
            Constraint::Percentage(percent_x),
            Constraint::Percentage((100 - percent_x) / 2),
        ])
        .split(popup_layout[1])[1]
}

fn render_menu(f: &mut Frame, _app: &App, area: Rect) {
    f.render_widget(
        Block::default().style(Style::default().bg(BASE)),
        area,
    );

    let logo = vec![
        " ███████╗ ██████╗██████╗ ██╗██████╗ ████████╗██╗   ██╗██████╗ ███████╗███████╗",
        " ██╔════╝██╔════╝██╔══██╗██║██╔══██╗╚══██╔══╝██║   ██║██╔══██╗██╔════╝██╔════╝",
        " ███████╗██║     ██████╔╝██║██████╔╝   ██║   ██║   ██║██████╔╝█████╗  ███████╗",
        " ╚════██║██║     ██╔══██╗██║██╔═══╝    ██║   ██║   ██║██╔══██╗██╔══╝  ╚════██║",
        " ███████║╚██████╗██║  ██║██║██║        ██║   ╚██████╔╝██║  ██║███████╗███████║",
        " ╚══════╝ ╚═════╝╚═╝  ╚═╝╚═╝╚═╝        ╚═╝    ╚═════╝ ╚═╝  ╚═╝╚══════╝╚══════╝",
    ];

    let menu_items = vec![
        ("›", "Continue Reading", "c"),
        ("›", "Find Verse",       "f"),
        ("›", "Notes",            "n"),
        ("›", "Devotionals",      "d"),
        ("›", "Recent Searches",  "r"),
        ("›", "Project",          "p"),
        ("›", "Config",           "g"),
        ("›", "Quit",             "q"),
    ];

    let total_height = logo.len() as u16 + 2 + menu_items.len() as u16 + 3;
    let v_pad = area.height.saturating_sub(total_height) / 2;
    let h_pad = area.width.saturating_sub(82) / 2;

    let layout = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(v_pad),
            Constraint::Length(6),
            Constraint::Length(2),
            Constraint::Length(8),
            Constraint::Length(2),
            Constraint::Min(0),
        ])
        .split(area);

    for (i, line) in logo.iter().enumerate() {
        let y = layout[1].y + i as u16;
        let logo_area = Rect::new(h_pad, y, area.width.saturating_sub(h_pad), 1);
        f.render_widget(
            Paragraph::new(Span::styled(*line, Style::default().fg(BLUE))),
            logo_area,
        );
    }

    let menu_x = area.width / 2 - 18;
    for (i, (icon, label, key)) in menu_items.iter().enumerate() {
        let y = layout[3].y + i as u16;
        let row_area = Rect::new(menu_x, y, 40, 1);
        f.render_widget(
            Paragraph::new(Line::from(vec![
                Span::styled(format!("{} ", icon), Style::default().fg(BLUE)),
                Span::styled(format!("{:<22}", label), Style::default().fg(TEXT)),
                Span::styled(key.to_string(), Style::default().fg(RED)),
            ])),
            row_area,
        );
    }

    let footer_area = Rect::new(menu_x, layout[4].y, 40, 1);
    f.render_widget(
        Paragraph::new(Line::from(vec![
            Span::styled("⚡ Scriptures loaded in ", Style::default().fg(OVERLAY2)),
            Span::styled("0.31ms", Style::default().fg(GREEN)),
        ])),
        footer_area,
    );
}
