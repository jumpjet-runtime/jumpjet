// use std::{
//     collections::HashMap, path::Path, time::{Duration, SystemTime, UNIX_EPOCH}
// };

// use color_eyre::eyre::Result;
// use crossterm::event::{KeyCode, KeyEvent};
// use ratatui::{prelude::*, widgets::*};
// use serde::{Deserialize, Serialize};
// use tokio::sync::mpsc::UnboundedSender;
// use wit_parser::UnresolvedPackageGroup;

// use super::{Component, Frame};
// use crate::{
//     action::Action,
//     config::{Config, KeyBindings}, tui::Event,
// };

// #[derive(Default)]
// pub struct Home {
//     command_tx: Option<UnboundedSender<Action>>,
//     config: Config,
//     docs_component: Docs
// }

// impl Home {
//     pub fn new() -> Self {
//         Self::default()
//     }

//     fn docs_page(&mut self, f: &mut Frame<'_>, area: Rect) {
//         self.docs_component.draw(f, area);
//     }
// }

// impl Component for Home {
//     fn register_action_handler(&mut self, tx: UnboundedSender<Action>) -> Result<()> {
//         self.docs_component.register_action_handler(tx.clone())?;
//         self.command_tx = Some(tx);
//         Ok(())
//     }

//     fn register_config_handler(&mut self, config: Config) -> Result<()> {
//         self.docs_component.register_config_handler(config.clone())?;
//         self.config = config;
//         Ok(())
//     }

//     fn handle_events(&mut self, event: Option<Event>) -> Result<Option<Action>> {
//         self.docs_component.handle_events(event)?;
//         Ok(None)
//     }

//     fn update(&mut self, action: Action) -> Result<Option<Action>> {
//         self.docs_component.update(action.clone())?;

//         match action {
//             Action::Tick => {}
//             _ => {}
//         }
//         Ok(None)
//     }

//     fn draw(&mut self, f: &mut Frame<'_>, area: Rect) -> Result<()> {
//         let layout = Layout::vertical([Constraint::Length(3), Constraint::Length(2), Constraint::Fill(1), Constraint::Length(3)]);

//         let [header_area, nav_area, content_area, footer_area] = layout.areas(area);

//         let logo = Paragraph::new(
//             Line::from(vec![
//                 Span::styled(" jumpjet ", Style::default().add_modifier(Modifier::REVERSED | Modifier::BOLD))
//             ])
//         );

//         let header_layout = Layout::horizontal([Constraint::Fill(1), Constraint::Fill(1)])
//             .vertical_margin(1)
//             .horizontal_margin(2);
//         let [logo_area, top_right_area] = header_layout.areas(header_area);

//         f.render_widget(logo, logo_area);

//         f.render_widget(
//             Paragraph::new(Span::styled("lochie@live.com", Style::default().bold())).right_aligned(),
//             top_right_area,
//         );

//         // f.render_widget(
//         //     Paragraph::new(
//         //         SystemTime::now()
//         //             .duration_since(UNIX_EPOCH)
//         //             .unwrap()
//         //             .as_micros()
//         //             .to_string(),
//         //     ).right_aligned(),
//         //     top_right_area,
//         // );

//         f.render_widget(
//             Tabs::new(vec!["Docs", "Games", "Settings"])
//                 .block(Block::default().padding(Padding::horizontal(1)))
//                 .style(Style::reset().add_modifier(Modifier::DIM | Modifier::BOLD))
//                 .highlight_style(Style::reset().bold())
//                 .select(0)
//                 .divider(""),
//             nav_area,
//         );

//         self.docs_page(f, content_area);

//         let footer_layout = Layout::horizontal([Constraint::Fill(1), Constraint::Fill(1)])
//             .vertical_margin(1)
//             .horizontal_margin(2);
//         let [nav_tips_area, bottom_right_area] = footer_layout.areas(footer_area);

//         f.render_widget(
//             Paragraph::new(
//                 Span::styled("↑/↓ nav tips ←/→", Style::reset().add_modifier(Modifier::DIM))
//             ),
//             nav_tips_area,
//         );

//         f.render_widget(
//             Paragraph::new(Span::styled("1.0.0", Style::reset().add_modifier(Modifier::DIM))).right_aligned(),
//             bottom_right_area,
//         );

//         Ok(())
//     }
// }
