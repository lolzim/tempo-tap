/******
* tempo-tap by Lol Zimmerli
* 
* This Source Code Form is subject to the terms of the Mozilla Public
* License, v. 2.0. If a copy of the MPL was not distributed with this
* file, You can obtain one at https://mozilla.org/MPL/2.0/.
*/

use crossterm::{
    cursor,
    event::{self, Event, KeyCode, KeyEvent, KeyEventKind, KeyModifiers},
    execute,
    style::{Color, Print, ResetColor, SetForegroundColor},
    terminal::{self, ClearType},
};
use std::io::{stdout, Write};
use std::time::{Duration, Instant};

// internationalisation module
mod i18n;

// Number of steps to compute tempo
const MIN_STEPS: usize = 8;

// Delay before entering pause-mode
const PAUSE_THRESHOLD_MS: u128 = 3000;

// Line for tempo (header: lines 0-4)
const TEMPO_ROW: u16 = 5;

struct TapTempo {
    // Latests keystrockes
    taps: Vec<Instant>,
    // Last computed BPM (None if not yet two keystrockes)
    last_bpm: Option<f64>,
    // Latest keystrock instant
    last_tap: Option<Instant>,
}

impl TapTempo {
    fn new() -> Self {
        Self {
            taps: Vec::with_capacity(MIN_STEPS + 1),
            last_bpm: None,
            last_tap: None,
        }
    }
    
    fn tap(&mut self) {
        let now = Instant::now();
        
        // If pause is too long, clear to compute from zero again
        if let Some(last) = self.last_tap {
            if last.elapsed().as_millis() > PAUSE_THRESHOLD_MS {
                self.taps.clear();
            }
        }
        
        self.taps.push(now);
        self.last_tap = Some(now);
        
        // Keep only latest MIN_STEPS keystrockes
        if self.taps.len() > MIN_STEPS {
            self.taps.remove(0);
        }
        
        // At least two keystrocks to have an interval
        if self.taps.len() >= 2 {
            self.last_bpm = Some(self.compute_bpm());
        }
    }
    
    fn compute_bpm(&self) -> f64 {
        let n = self.taps.len();
        // Total duration between keystrocks
        let total_ms = self.taps[n - 1]
            .duration_since(self.taps[0])
            .as_secs_f64()
            * 1000.0;
        // ƒ of intervals is # of keystrocks -1
        let avg_interval_ms = total_ms / (n as f64 - 1.0);
        // BPM = 60 000 ms / average interval
        60_000.0 / avg_interval_ms
    }
    
    fn tap_count(&self) -> usize {
        self.taps.len()
    }
    
    // Show if pause-mode
    fn is_paused(&self) -> bool {
        self.last_tap
            .map(|t| t.elapsed().as_millis() > PAUSE_THRESHOLD_MS)
            .unwrap_or(false)
    }
}

fn render(tap: &TapTempo, lang: &'static str, stdout: &mut impl Write) -> std::io::Result<()> {
    // Go to line an clear it
    execute!(
        stdout,
        cursor::MoveTo(0, TEMPO_ROW),
        terminal::Clear(ClearType::CurrentLine),
    )?;
    
    match tap.last_bpm {
        None => {
            let hint = format!("  {} ({}/{})", i18n::t(lang, "tap_hint"), tap.tap_count(), 2);
            execute!(
                stdout,
                SetForegroundColor(Color::DarkGrey),
                Print(hint),
                ResetColor,
            )?;
        }
        Some(bpm) => {
            let bpm_rounded = bpm.round() as u32;
            
            // Color is life, life is colors
            let color = match bpm_rounded {
                0..=59 => Color::Blue,
                60..=89 => Color::Green,
                90..=129 => Color::Yellow,
                130..=179 => Color::Magenta,
                _ => Color::Red,
            };
            
            let pause_indicator = if tap.is_paused() { " ⏸" } else { " ▶" };
            let appuis = i18n::t(lang, "appuis");
            execute!(
                stdout,
                SetForegroundColor(color),
                Print(format!("  ♩ {:>3} BPM", bpm_rounded)),
                ResetColor,
                SetForegroundColor(Color::DarkGrey),
                Print(format!(
                    "  │ {}  │  {} {}",
                    pause_indicator,
                    tap.tap_count(),
                    appuis
                )),
                ResetColor,
            )?;
        }
    }
    
    // Show delays for quarter, eighth & sixteenth notes in ms
    execute!(
        stdout,
        cursor::MoveTo(0, TEMPO_ROW + 1),
        terminal::Clear(ClearType::CurrentLine),
    )?;
    
    if let Some(bpm) = tap.last_bpm {
        let quarter_ms = 60_000.0 / bpm;
        let eigther_ms: f64 = quarter_ms / 2.00;
        let sx_ms: f64 = eigther_ms / 2.00;
        let note_4 = i18n::t(lang, "note_noire");
        let note_8 = i18n::t(lang, "note_croche");
        let note_16 = i18n::t(lang, "note_double");
        
        execute!(
            stdout,
            SetForegroundColor(Color::DarkGrey),
            Print(format!("  {} = {:.1} ms, {} = {:.1} ms, {} = {:.1} ms",
                note_4, quarter_ms, note_8, eigther_ms, note_16, sx_ms)),
            ResetColor,
        )?;
    }
    
    stdout.flush()?;
    Ok(())
}

fn print_header(stdout: &mut impl Write, lang: &'static str) -> std::io::Result<()> {
    let t_tap1 = i18n::t(lang, "tap_hint_too");
    let t_tap2 = i18n::t(lang, "quit_hint");
    execute!(
        stdout,
        terminal::Clear(ClearType::All),
        cursor::MoveTo(0, 0),
        SetForegroundColor(Color::Cyan),
        Print("  +----------------------------------+\r\n"),
        Print("  |         TEMPO TAP                |\r\n"),
        Print("  +----------------------------------+\r\n"),
        ResetColor,
        SetForegroundColor(Color::DarkGrey),
        Print("  ".to_owned() + t_tap1 + "\r\n"),
        Print("  ".to_owned() + t_tap2 + "\r\n"),
        ResetColor,
    )?;
    stdout.flush()?;
    Ok(())
}

fn main() -> std::io::Result<()> {
    let mut stdout = stdout();
    
    // Use raw mode : keys are read without wainting for [RETURN]
    terminal::enable_raw_mode()?;
    
    // Hide cursos
    execute!(stdout, cursor::Hide)?;
    
    let lang = i18n::detect_lang();
    let mut tap = TapTempo::new();
    
    print_header(&mut stdout, lang)?;
    
    loop {
        // Non-blocking reading with timeout (to detect pauses)
        if event::poll(Duration::from_millis(200))? {
            match event::read()? {
                // Quit with Ctrl+C or q
                Event::Key(KeyEvent {
                    code: KeyCode::Char('c'),
                    modifiers: KeyModifiers::CONTROL,
                    ..
                }) => { break; }
                
                // Quit with 'q'
                Event::Key(KeyEvent {
                    code: KeyCode::Char('q'),
                    modifiers: KeyModifiers::NONE,
                    kind: KeyEventKind::Press,
                    ..
                }) => { break; }
                
                // Any Press = tap (sauf modifiers seuls)
                Event::Key(KeyEvent { kind: KeyEventKind::Press, code, .. }) => {
                    match code {
                        KeyCode::Modifier(_) => {}
                        _ => {
                            tap.tap();
                            render(&tap, lang, &mut stdout)?;
                        }
                    }
                }
                
                // Ignore Release, Repeat, etc.
                _ => {}
            }
        } else {
            // Timeout : refresh display
            if tap.last_bpm.is_some() {
                render(&tap, lang, &mut stdout)?;
            }
        }
    }
    
    // Cleaning : restore terminal
    terminal::disable_raw_mode()?;
    execute!(
        stdout,
        cursor::Show,
        cursor::MoveToNextLine(1),
        SetForegroundColor(Color::DarkGrey),
        Print("  Bye !\r\n"),
        ResetColor,
    )?;
    
    Ok(())
}
