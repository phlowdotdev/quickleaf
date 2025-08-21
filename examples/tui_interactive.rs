//! Interactive Terminal UI example for Quickleaf with SQLite persistence
//! 
//! Run with: cargo run --example tui_interactive --features tui-example

#[cfg(feature = "tui-example")]
use quickleaf::{Cache, ListProps, Order, Filter};
#[cfg(feature = "tui-example")]
use std::time::Duration;

#[cfg(feature = "tui-example")]
use ratatui::{
    backend::{Backend, CrosstermBackend},
    layout::{Alignment, Constraint, Direction, Layout},
    style::{Color, Modifier, Style},
    widgets::{Block, Borders, List, ListItem, Paragraph, Clear},
    Frame, Terminal,
};

#[cfg(feature = "tui-example")]
use crossterm::{
    event::{self, DisableMouseCapture, EnableMouseCapture, Event, KeyCode, KeyEventKind},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};

#[cfg(feature = "tui-example")]
use std::io;

#[cfg(feature = "tui-example")]
#[derive(Debug, Clone)]
enum MenuItem {
    Insert,
    InsertWithTTL,
    Get,
    Remove,
    List,
    Filter,
    Clear,
    CleanupExpired,
    Stats,
    Exit,
}

#[cfg(feature = "tui-example")]
impl MenuItem {
    fn all() -> Vec<MenuItem> {
        vec![
            MenuItem::Insert,
            MenuItem::InsertWithTTL,
            MenuItem::Get,
            MenuItem::Remove,
            MenuItem::List,
            MenuItem::Filter,
            MenuItem::Clear,
            MenuItem::CleanupExpired,
            MenuItem::Stats,
            MenuItem::Exit,
        ]
    }
    
    fn name(&self) -> &str {
        match self {
            MenuItem::Insert => "📝 Insert Key-Value",
            MenuItem::InsertWithTTL => "⏰ Insert with TTL",
            MenuItem::Get => "🔍 Get Value",
            MenuItem::Remove => "🗑️  Remove Key",
            MenuItem::List => "📋 List All Items",
            MenuItem::Filter => "🔎 Filter Items",
            MenuItem::Clear => "🧹 Clear Cache",
            MenuItem::CleanupExpired => "♻️  Cleanup Expired",
            MenuItem::Stats => "📊 Cache Statistics",
            MenuItem::Exit => "🚪 Exit",
        }
    }
    
    fn description(&self) -> &str {
        match self {
            MenuItem::Insert => "Insert a new key-value pair into the cache",
            MenuItem::InsertWithTTL => "Insert a key-value pair with Time To Live",
            MenuItem::Get => "Retrieve a value by its key",
            MenuItem::Remove => "Remove a key-value pair from the cache",
            MenuItem::List => "List all items in the cache",
            MenuItem::Filter => "Filter items by prefix, suffix, or pattern",
            MenuItem::Clear => "Clear all items from the cache",
            MenuItem::CleanupExpired => "Remove all expired items from the cache",
            MenuItem::Stats => "View cache statistics and information",
            MenuItem::Exit => "Exit the application",
        }
    }
}

#[cfg(feature = "tui-example")]
struct App {
    cache: Cache,
    selected_menu: usize,
    input_mode: bool,
    input_buffer: String,
    second_input_buffer: String,
    third_input_buffer: String,
    messages: Vec<String>,
    current_action: Option<MenuItem>,
    input_stage: usize, // For multi-input actions
}

#[cfg(feature = "tui-example")]
impl App {
    fn new() -> Result<Self, Box<dyn std::error::Error>> {
        let cache = Cache::with_persist("tui_cache.db", 1000)?;
        Ok(Self {
            cache,
            selected_menu: 0,
            input_mode: false,
            input_buffer: String::new(),
            second_input_buffer: String::new(),
            third_input_buffer: String::new(),
            messages: vec!["Welcome to Quickleaf Interactive TUI! 🍃".to_string()],
            current_action: None,
            input_stage: 0,
        })
    }
    
    fn add_message(&mut self, msg: String) {
        self.messages.push(msg);
        // Keep only last 10 messages
        if self.messages.len() > 10 {
            self.messages.remove(0);
        }
    }
    
    fn execute_action(&mut self) {
        match self.current_action.as_ref() {
            Some(MenuItem::Insert) => {
                if self.input_stage == 0 {
                    self.input_stage = 1;
                    self.add_message("Enter value:".to_string());
                } else {
                    let key = self.input_buffer.clone();
                    let value = self.second_input_buffer.clone();
                    self.cache.insert(&key, value.as_str());
                    self.add_message(format!("✅ Inserted: {} = {}", key, value));
                    self.reset_input();
                }
            }
            Some(MenuItem::InsertWithTTL) => {
                match self.input_stage {
                    0 => {
                        self.input_stage = 1;
                        self.add_message("Enter value:".to_string());
                    }
                    1 => {
                        self.input_stage = 2;
                        self.add_message("Enter TTL in seconds:".to_string());
                    }
                    2 => {
                        let key = self.input_buffer.clone();
                        let value = self.second_input_buffer.clone();
                        if let Ok(ttl_secs) = self.third_input_buffer.parse::<u64>() {
                            self.cache.insert_with_ttl(&key, value.as_str(), Duration::from_secs(ttl_secs));
                            self.add_message(format!("✅ Inserted with TTL: {} = {} ({}s)", key, value, ttl_secs));
                        } else {
                            self.add_message("❌ Invalid TTL value".to_string());
                        }
                        self.reset_input();
                    }
                    _ => {}
                }
            }
            Some(MenuItem::Get) => {
                let key = self.input_buffer.clone();
                let value_opt = self.cache.get(&key).cloned();
                match value_opt {
                    Some(value) => {
                        self.add_message(format!("✅ Found: {} = {:?}", key, value));
                    }
                    None => {
                        self.add_message(format!("❌ Key not found: {}", key));
                    }
                }
                self.reset_input();
            }
            Some(MenuItem::Remove) => {
                let key = self.input_buffer.clone();
                match self.cache.remove(&key) {
                    Ok(_) => {
                        self.add_message(format!("✅ Removed: {}", key));
                    }
                    Err(_) => {
                        self.add_message(format!("❌ Failed to remove: {}", key));
                    }
                }
                self.reset_input();
            }
            Some(MenuItem::List) => {
                let items = self.cache.list(ListProps::default().order(Order::Asc))
                    .unwrap_or_default()
                    .into_iter()
                    .map(|(k, v)| (k, v.clone()))
                    .collect::<Vec<_>>();
                    
                if items.is_empty() {
                    self.add_message("📋 Cache is empty".to_string());
                } else {
                    self.add_message(format!("📋 Cache contains {} items:", items.len()));
                    for (key, value) in items.iter().take(5) {
                        self.add_message(format!("  • {} = {:?}", key, value));
                    }
                    if items.len() > 5 {
                        self.add_message(format!("  ... and {} more items", items.len() - 5));
                    }
                }
                self.reset_input();
            }
            Some(MenuItem::Filter) => {
                let prefix = self.input_buffer.clone();
                let items = self.cache.list(
                    ListProps::default()
                        .filter(Filter::StartWith(prefix.clone()))
                        .order(Order::Asc)
                ).unwrap_or_default()
                    .into_iter()
                    .map(|(k, v)| (k, v.clone()))
                    .collect::<Vec<_>>();
                
                if items.is_empty() {
                    self.add_message(format!("🔍 No items found with prefix: {}", prefix));
                } else {
                    self.add_message(format!("🔍 Found {} items with prefix '{}':", items.len(), prefix));
                    for (key, value) in items.iter().take(5) {
                        self.add_message(format!("  • {} = {:?}", key, value));
                    }
                }
                self.reset_input();
            }
            Some(MenuItem::Clear) => {
                self.cache.clear();
                self.add_message("🧹 Cache cleared!".to_string());
                self.reset_input();
            }
            Some(MenuItem::CleanupExpired) => {
                let removed = self.cache.cleanup_expired();
                self.add_message(format!("♻️ Cleaned up {} expired items", removed));
                self.reset_input();
            }
            Some(MenuItem::Stats) => {
                let len = self.cache.len();
                let capacity = self.cache.capacity();
                
                self.add_message(format!("📊 Cache Statistics:"));
                self.add_message(format!("  • Items: {}/{}", len, capacity));
                self.add_message(format!("  • Capacity: {}", capacity));
                self.add_message(format!("  • Usage: {:.1}%", (len as f64 / capacity as f64) * 100.0));
                self.add_message(format!("  • Persistence: tui_cache.db (SQLite)"));
                
                self.reset_input();
            }
            _ => {}
        }
    }
    
    fn reset_input(&mut self) {
        self.input_mode = false;
        self.input_buffer.clear();
        self.second_input_buffer.clear();
        self.third_input_buffer.clear();
        self.current_action = None;
        self.input_stage = 0;
    }
    
    fn get_input_prompt(&self) -> String {
        match (&self.current_action, self.input_stage) {
            (Some(MenuItem::Insert), 0) => "Enter key: ".to_string(),
            (Some(MenuItem::Insert), 1) => "Enter value: ".to_string(),
            (Some(MenuItem::InsertWithTTL), 0) => "Enter key: ".to_string(),
            (Some(MenuItem::InsertWithTTL), 1) => "Enter value: ".to_string(),
            (Some(MenuItem::InsertWithTTL), 2) => "Enter TTL (seconds): ".to_string(),
            (Some(MenuItem::Get), _) => "Enter key to get: ".to_string(),
            (Some(MenuItem::Remove), _) => "Enter key to remove: ".to_string(),
            (Some(MenuItem::Filter), _) => "Enter prefix to filter: ".to_string(),
            _ => "Input: ".to_string(),
        }
    }
    
    fn get_current_input(&self) -> &str {
        match self.input_stage {
            0 => &self.input_buffer,
            1 => &self.second_input_buffer,
            2 => &self.third_input_buffer,
            _ => &self.input_buffer,
        }
    }
    
    fn append_to_current_input(&mut self, c: char) {
        match self.input_stage {
            0 => self.input_buffer.push(c),
            1 => self.second_input_buffer.push(c),
            2 => self.third_input_buffer.push(c),
            _ => {}
        }
    }
    
    fn pop_from_current_input(&mut self) {
        match self.input_stage {
            0 => { self.input_buffer.pop(); }
            1 => { self.second_input_buffer.pop(); }
            2 => { self.third_input_buffer.pop(); }
            _ => {}
        }
    }
}

#[cfg(feature = "tui-example")]
fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Setup terminal
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen, EnableMouseCapture)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;
    
    // Create app
    let mut app = App::new()?;
    
    // Main loop
    let res = run_app(&mut terminal, &mut app);
    
    // Restore terminal
    disable_raw_mode()?;
    execute!(
        terminal.backend_mut(),
        LeaveAlternateScreen,
        DisableMouseCapture
    )?;
    terminal.show_cursor()?;
    
    if let Err(err) = res {
        println!("Error: {:?}", err);
    }
    
    Ok(())
}

#[cfg(feature = "tui-example")]
fn run_app<B: Backend>(terminal: &mut Terminal<B>, app: &mut App) -> io::Result<()> {
    loop {
        terminal.draw(|f| ui(f, app))?;
        
        if let Event::Key(key) = event::read()? {
            if key.kind == KeyEventKind::Press {
                if app.input_mode {
                    match key.code {
                        KeyCode::Enter => {
                            app.execute_action();
                        }
                        KeyCode::Char(c) => {
                            app.append_to_current_input(c);
                        }
                        KeyCode::Backspace => {
                            app.pop_from_current_input();
                        }
                        KeyCode::Esc => {
                            app.reset_input();
                        }
                        _ => {}
                    }
                } else {
                    match key.code {
                        KeyCode::Char('q') => {
                            return Ok(());
                        }
                        KeyCode::Up => {
                            if app.selected_menu > 0 {
                                app.selected_menu -= 1;
                            }
                        }
                        KeyCode::Down => {
                            let menu_items = MenuItem::all();
                            if app.selected_menu < menu_items.len() - 1 {
                                app.selected_menu += 1;
                            }
                        }
                        KeyCode::Enter => {
                            let menu_items = MenuItem::all();
                            let selected = &menu_items[app.selected_menu];
                            
                            match selected {
                                MenuItem::Exit => {
                                    return Ok(());
                                }
                                MenuItem::List | MenuItem::Clear | 
                                MenuItem::CleanupExpired | MenuItem::Stats => {
                                    app.current_action = Some(selected.clone());
                                    app.execute_action();
                                }
                                _ => {
                                    app.input_mode = true;
                                    app.current_action = Some(selected.clone());
                                    app.input_stage = 0;
                                }
                            }
                        }
                        _ => {}
                    }
                }
            }
        }
    }
}

#[cfg(feature = "tui-example")]
fn ui(f: &mut Frame, app: &App) {
    let chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Percentage(30),
            Constraint::Percentage(70),
        ])
        .split(f.area());
    
    // Left panel - Menu
    let menu_items = MenuItem::all();
    let items: Vec<ListItem> = menu_items
        .iter()
        .enumerate()
        .map(|(i, item)| {
            let style = if i == app.selected_menu {
                Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD)
            } else {
                Style::default()
            };
            ListItem::new(item.name()).style(style)
        })
        .collect();
    
    let menu = List::new(items)
        .block(Block::default()
            .title(" 🍃 Quickleaf Menu ")
            .borders(Borders::ALL)
            .border_style(Style::default().fg(Color::Green)));
    
    f.render_widget(menu, chunks[0]);
    
    // Right panel - split into description, messages, and input
    let right_chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(4),  // Description
            Constraint::Min(10),     // Messages
            Constraint::Length(3),   // Input
        ])
        .split(chunks[1]);
    
    // Description area
    let selected_item = &menu_items[app.selected_menu];
    let description = Paragraph::new(selected_item.description())
        .block(Block::default()
            .title(" Description ")
            .borders(Borders::ALL))
        .style(Style::default().fg(Color::White))
        .alignment(Alignment::Left);
    
    f.render_widget(description, right_chunks[0]);
    
    // Messages area
    let messages: Vec<ListItem> = app.messages
        .iter()
        .map(|msg| ListItem::new(msg.as_str()))
        .collect();
    
    let messages_list = List::new(messages)
        .block(Block::default()
            .title(" Output ")
            .borders(Borders::ALL))
        .style(Style::default().fg(Color::Yellow));
    
    f.render_widget(messages_list, right_chunks[1]);
    
    // Input area (shown when in input mode)
    if app.input_mode {
        let input_text = format!("{}{}", app.get_input_prompt(), app.get_current_input());
        let input = Paragraph::new(input_text)
            .block(Block::default()
                .title(" Input (ESC to cancel, ENTER to submit) ")
                .borders(Borders::ALL)
                .border_style(Style::default().fg(Color::Cyan)))
            .style(Style::default().fg(Color::White));
        
        f.render_widget(Clear, right_chunks[2]);
        f.render_widget(input, right_chunks[2]);
    } else {
        let help = Paragraph::new("↑/↓: Navigate | Enter: Select | q: Quit")
            .block(Block::default()
                .title(" Help ")
                .borders(Borders::ALL))
            .style(Style::default().fg(Color::Gray))
            .alignment(Alignment::Center);
        
        f.render_widget(help, right_chunks[2]);
    }
}

#[cfg(not(feature = "tui-example"))]
fn main() {
    println!("❌ This example requires the 'tui-example' feature to be enabled.");
    println!("   Run with: cargo run --example tui_interactive --features tui-example");
}
