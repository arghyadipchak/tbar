mod line;
mod tab;

use std::cmp::{max, min};

use zellij_tile::prelude::*;

use crate::{line::line, tab::style};

#[derive(Clone, Debug, Default)]
pub struct Part {
  part: String,
  len: usize,
  tab_idx: Option<usize>,
}

#[derive(Default)]
struct State {
  tabs: Vec<TabInfo>,
  active_tab_idx: usize,
  mode_info: ModeInfo,
  mouse_click_pos: usize,
  change_tab: bool,
}

register_plugin!(State);

const ARROW_SEPARATOR: &str = "";

impl ZellijPlugin for State {
  fn load(&mut self) {
    set_selectable(false);
    subscribe(&[
      EventType::TabUpdate,
      EventType::ModeUpdate,
      EventType::Mouse,
    ]);
  }

  fn update(&mut self, event: Event) -> bool {
    match event {
      Event::ModeUpdate(mode_info) => {
        if self.mode_info == mode_info {
          false
        } else {
          self.mode_info = mode_info;
          true
        }
      }
      Event::TabUpdate(tabs) => {
        if let Some(active_tab_idx) = tabs.iter().position(|t| t.active) {
          if self.active_tab_idx == active_tab_idx && self.tabs == tabs {
            false
          } else {
            self.active_tab_idx = active_tab_idx;
            self.tabs = tabs;
            true
          }
        } else {
          eprintln!("Active tab not found!");
          false
        }
      }
      Event::Mouse(me) => match me {
        Mouse::LeftClick(_, col) => {
          if self.mouse_click_pos == col {
            false
          } else {
            self.mouse_click_pos = col;
            self.change_tab = true;
            true
          }
        }
        Mouse::ScrollUp(_) => {
          switch_tab_to(min(self.active_tab_idx + 2, self.tabs.len()) as u32);
          true
        }
        Mouse::ScrollDown(_) => {
          switch_tab_to(max(self.active_tab_idx, 1) as u32);
          true
        }
        _ => false,
      },
      _ => {
        eprintln!("Unrecognized Event: {event:?}");
        false
      }
    }
  }

  fn render(&mut self, _rows: usize, cols: usize) {
    if self.tabs.is_empty() {
      return;
    }

    let tab_parts = self
      .tabs
      .iter()
      .enumerate()
      .map(|(i, t)| {
        style(
          if t.active
            && self.mode_info.mode == InputMode::RenameTab
            && t.name.is_empty()
          {
            "?"
          } else {
            &t.name
          },
          t,
          i % 2 == 1,
          self.mode_info.style.colors,
          self.mode_info.capabilities,
        )
      })
      .collect::<Vec<_>>();

    let bar_parts = line(
      &tab_parts,
      self.active_tab_idx,
      cols.saturating_sub(1),
      self.mode_info.style.colors,
      self.mode_info.capabilities,
      self.mode_info.mode,
    );

    if self.change_tab {
      let mut len_cum = 0;
      for bar_part in &bar_parts {
        if self.mouse_click_pos >= len_cum
          && self.mouse_click_pos < len_cum + bar_part.len
        {
          if let Some(tab_idx) = bar_part.tab_idx {
            switch_tab_to(tab_idx as u32 + 1);
          }
          break;
        }
        len_cum += bar_part.len;
      }
      self.change_tab = false;
    }

    let bar: String = bar_parts.iter().map(|lp| lp.part.as_str()).collect();
    match match self.mode_info.style.colors.theme_hue {
      ThemeHue::Dark => self.mode_info.style.colors.black,
      ThemeHue::Light => self.mode_info.style.colors.white,
    } {
      PaletteColor::Rgb((r, g, b)) => {
        print!("{bar}\u{1b}[48;2;{r};{g};{b}m\u{1b}[0K");
      }
      PaletteColor::EightBit(color) => {
        print!("{bar}\u{1b}[48;5;{color}m\u{1b}[0K");
      }
    }
  }
}
