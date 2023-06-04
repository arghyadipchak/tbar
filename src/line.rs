use unicode_width::UnicodeWidthStr;
use zellij_tile::prelude::*;
use zellij_tile_utils::style;

use crate::{
  tab::{collapsed, separator},
  Part,
};

fn prefix(mode: InputMode, palette: Palette) -> Vec<Part> {
  let mode_part = match mode {
    InputMode::EnterSearch => "ES",
    InputMode::Locked => "L ",
    InputMode::Move => "M ",
    InputMode::Normal => "N ",
    InputMode::Pane => "P ",
    InputMode::Prompt => "PR",
    InputMode::RenamePane => "RP",
    InputMode::RenameTab => "RT",
    InputMode::Resize => "R ",
    InputMode::Search => "SR",
    InputMode::Scroll => "SC",
    InputMode::Session => "SS",
    InputMode::Tab => "T ",
    InputMode::Tmux => "TM",
  };

  let mode_part_styled = style!(
    if mode == InputMode::Locked {
      palette.magenta
    } else if mode == InputMode::Normal {
      palette.green
    } else {
      palette.orange
    },
    match palette.theme_hue {
      ThemeHue::Dark => palette.black,
      ThemeHue::Light => palette.white,
    }
  )
  .bold()
  .paint(mode_part)
  .to_string();

  vec![Part {
    part: mode_part_styled,
    len: mode_part.width(),
    tab_idx: None,
  }]
}

pub fn line(
  all_tabs: &[Part],
  active_tab_idx: usize,
  cols: usize,
  palette: Palette,
  capabilities: PluginCapabilities,
  mode: InputMode,
) -> Vec<Part> {
  let prefix_parts = prefix(mode, palette);
  if prefix_parts.len() > cols {
    return vec![];
  }

  let pre_tab_len = prefix_parts.len() + all_tabs[active_tab_idx].len;
  if pre_tab_len > cols {
    return prefix_parts;
  }

  let tab_cnt = all_tabs.len();
  let tab_separator = separator(capabilities);

  let mut i = active_tab_idx;
  let mut j = active_tab_idx + 1;

  let mut len_right = (cols - pre_tab_len + 1) / 2;
  while j < tab_cnt
    && all_tabs[j].len
      + collapsed(tab_cnt - j - 1, j + 1, palette, tab_separator, false).len
      <= len_right
  {
    len_right -= all_tabs[j].len;
    j += 1;
  }

  let mut len_left = (cols - pre_tab_len) / 2 + len_right
    - collapsed(tab_cnt - j, j, palette, tab_separator, false).len;
  while i != 0
    && all_tabs[i - 1].len
      + collapsed(i - 1, i.saturating_sub(2), palette, tab_separator, true).len
      <= len_left
  {
    i -= 1;
    len_left -= all_tabs[i].len;
  }

  len_right = len_left
    + collapsed(tab_cnt - j, j, palette, tab_separator, false).len
    - collapsed(i, i.saturating_sub(1), palette, tab_separator, true).len;
  while j < tab_cnt
    && all_tabs[j].len
      + collapsed(tab_cnt - j - 1, j + 1, palette, tab_separator, false).len
      <= len_right
  {
    len_right -= all_tabs[j].len;
    j += 1;
  }

  let mut parts = prefix_parts;
  if i != 0 {
    parts.push(collapsed(i, i - 1, palette, tab_separator, true));
  }
  parts.extend(all_tabs[i..j].iter().cloned());
  if j < tab_cnt {
    parts.push(collapsed(tab_cnt - j, j, palette, tab_separator, false));
  }

  parts
}
