use crossterm::{
    event::{self, DisableMouseCapture, EnableMouseCapture, Event, KeyCode},
    execute,
    terminal::{EnterAlternateScreen, LeaveAlternateScreen, disable_raw_mode, enable_raw_mode},
};
use std::{io::stdout, time::Duration};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    enable_raw_mode()?;
    let mut stdout = stdout();
    execute!(stdout, EnterAlternateScreen, EnableMouseCapture)?;
    let mut terminal = ratatui::Terminal::new(ratatui::backend::CrosstermBackend::new(stdout))?;
    
    loop {
        terminal.draw(|f| {
            let size = f.size();
            let greeting = ratatui::widgets::Paragraph::new("Hello World! (press q to quit)")
                .block(ratatui::widgets::Block::default().title("Test").borders(ratatui::widgets::Borders::ALL))
                .alignment(ratatui::layout::Alignment::Center);
            f.render_widget(greeting, size);
        })?;
        
        if event::poll(Duration::from_millis(100))? {
            if let Event::Key(key) = event::read()? {
                if key.code == KeyCode::Char('q') {
                    break;
                }
            }
        }
    }
    
    disable_raw_mode()?;
    execute!(
        stdout,
        LeaveAlternateScreen,
        DisableMouseCapture
    )?;
    Ok(())
}
