#[allow(unused)]
use ratatui::{
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Style, Stylize},
    symbols::border,
    text::{Line, Span, Text},
    widgets::{Block, Borders, Clear, List, ListItem, Table, Row, Paragraph, Wrap},
    Frame,
};

#[allow(unused)]
use crate::app::{App, CurrentScreen};

#[allow(unused)]
pub fn ui(frame: &mut Frame, app: &App){
    let outer_layout = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3),
            Constraint::Percentage(100),
        ])
        .split(frame.area());
    let inner_layout = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Percentage(80),
            Constraint::Percentage(20),
        ])
        .split(outer_layout[1]);
    let title = Line::from(" BorkBork ");
    let footer = Line::from(" Press (q) to quit ");
    let header_block = Block::bordered()
        .title(title.centered())
        .title_bottom(footer.centered())
        .border_set(border::ROUNDED);
    let line:Line = vec![
        "Server: ".gray().bold(),
        "164.90.146.27".cyan(),
        " | ".into(),
        "Username: ".gray().bold(),
        "Guest".cyan(),
        " | ".into(),
        "Staus: ".gray().bold(),
        "Online".green(),
    ].into();
    frame.render_widget(Paragraph::new(line).block(header_block), outer_layout[0]);

    let chat_title = Line::from(" Chat ");
    let users_title = Line::from(" Users ");
    let chat_block = Block::bordered()
        .title(chat_title.centered())
        .border_set(border::ROUNDED);
    let chat_inner_layout = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Percentage(100),
            Constraint::Length(3),
        ])
        .split(inner_layout[0]);
    let send_message_block = Block::bordered()
        .border_set(border::DOUBLE);
    let recv_messages_block = Block::bordered()
        .border_set(border::EMPTY);
    let recv_messages_text = Paragraph::new("hi hello here is some text mkay")
        .block(recv_messages_block);
    let users_block = Block::bordered()
        .title(users_title.centered())
        .border_set(border::ROUNDED);

    frame.render_widget( recv_messages_text, chat_inner_layout[0]);
    frame.render_widget( send_message_block, chat_inner_layout[1]);
    frame.render_widget( chat_block,  inner_layout[0]);
    frame.render_widget( users_block, inner_layout[1]);
}

