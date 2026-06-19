use crate::config::Config;
use crate::{Result, action::Action, components::Component, mode::Mode, tui};
use docs::Docs;
use ratatui::layout::Rect;
use tokio::sync::mpsc;

mod docs;

pub async fn docs(config: &Config, mode: &Mode) -> Result<()> {
    let (action_tx, mut action_rx) = mpsc::unbounded_channel();

    let mut last_tick_key_events = Vec::new();
    let tick_rate = 4.0;
    let frame_rate = 60.0;
    let mut should_quit = false;
    let mut should_suspend = false;

    let mut tui = tui::Tui::new()?.tick_rate(tick_rate).frame_rate(frame_rate);
    // tui.mouse(true);
    tui.enter()?;

    let mut docs = Docs::default();

    docs.register_action_handler(action_tx.clone())?;

    docs.register_config_handler(config.clone())?;

    docs.init(Rect::new(0, 0, tui.size()?.width, tui.size()?.height))?;

    loop {
        if let Some(e) = tui.next().await {
            match e {
                tui::Event::Quit => action_tx.send(Action::Quit)?,
                tui::Event::Tick => action_tx.send(Action::Tick)?,
                tui::Event::Render => action_tx.send(Action::Render)?,
                tui::Event::Resize(x, y) => action_tx.send(Action::Resize(x, y))?,
                tui::Event::Key(key) => {
                    if let Some(keymap) = config.keybindings.get(&mode) {
                        if let Some(action) = keymap.get(&vec![key]) {
                            log::info!("Got action: {action:?}");
                            action_tx.send(action.clone())?;
                        } else {
                            // If the key was not handled as a single key action,
                            // then consider it for multi-key combinations.
                            last_tick_key_events.push(key);

                            // Check for multi-key combinations
                            if let Some(action) = keymap.get(&last_tick_key_events) {
                                log::info!("Got action: {action:?}");
                                action_tx.send(action.clone())?;
                            }
                        }
                    };
                }
                _ => {}
            }
            if let Some(action) = docs.handle_events(Some(e.clone()))? {
                action_tx.send(action)?;
            }
        }

        while let Ok(action) = action_rx.try_recv() {
            if action != Action::Tick && action != Action::Render {
                log::debug!("{action:?}");
            }
            match action {
                Action::Tick => {
                    last_tick_key_events.drain(..);
                }
                Action::Quit => should_quit = true,
                Action::Suspend => should_suspend = true,
                Action::Resume => should_suspend = false,
                Action::Resize(w, h) => {
                    tui.resize(Rect::new(0, 0, w, h))?;
                    tui.draw(|f| {
                        let r = docs.draw(f, f.area());
                        if let Err(e) = r {
                            action_tx
                                .send(Action::Error(format!("Failed to draw: {:?}", e)))
                                .unwrap();
                        }
                    })?;
                }
                Action::Render => {
                    tui.draw(|f| {
                        let r = docs.draw(f, f.area());
                        if let Err(e) = r {
                            action_tx
                                .send(Action::Error(format!("Failed to draw: {:?}", e)))
                                .unwrap();
                        }
                    })?;
                }
                _ => {}
            }
            if let Some(action) = docs.update(action.clone())? {
                action_tx.send(action)?
            };
        }
        if should_suspend {
            tui.suspend()?;
            action_tx.send(Action::Resume)?;
            tui = tui::Tui::new()?.tick_rate(tick_rate).frame_rate(frame_rate);
            // tui.mouse(true);
            tui.enter()?;
        } else if should_quit {
            tui.stop()?;
            break;
        }
    }
    tui.exit()?;

    Ok(())
}
