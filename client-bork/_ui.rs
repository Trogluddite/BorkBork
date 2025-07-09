use ratatui::symbols::border;
#[allow(unused_imports)]
use ratatui::{
    buffer::Buffer,
    layout::{Alignment, Constraint, Direction, Layout, Rect},
    style::{Color, Stylize},
    text::{Line, Span, Text},
    widgets::{Block, Borders, BorderType, Paragraph, Widget},
};

use crate::app::App;

impl Widget for &App {
    fn render(self, area: Rect, buf: &mut Buffer) {
        let outer_layout = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Length(0),
                Constraint::Length(3),
                Constraint::Percentage(100),
            ])
            .split(area);
        let inner_layout = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([
                Constraint::Percentage(80),
                Constraint::Percentage(20),
            ])
            .split(outer_layout[2]);
        let title = Line::from(" BorkBork ");
        let footer = Line::from( "Press (q) or (Ctrl+c) to quit ");
        let header_block = Block::bordered()
            .title(title.centered())
            .title_bottom(footer.centered())
            .border_set(border::ROUNDED);
        
        let status_line:Line = vec![
            " Sever: ".gray().bold(),
            format!("{}", self.server_address).cyan(),
            " | ".into(),
            "Username: ".gray().bold(),
            "Guest".cyan(),
            " | ".into(),
            "Status: ".gray().bold(),
            {if self.connected == true {"Online".green()} else {"Offline".red()}},
            " | ".into(),
            " Server Version: ".gray().bold(),
            format!("{}.{}.{}", self.server_major_ver, self.server_minor_ver, self.server_subminor_ver).into(),
        ].into();
        let status_paragraph = Paragraph::new(status_line)
            .block(header_block);
        status_paragraph.render(outer_layout[1], buf);
        
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
        let users_block = Block::bordered()
            .title(users_title.centered())
            .border_set(border::ROUNDED);
        recv_messages_block.render(chat_inner_layout[0], buf);
        send_message_block.render(chat_inner_layout[1], buf);
        chat_block.render(inner_layout[0], buf);
        users_block.render(inner_layout[1], buf);
    }
}
