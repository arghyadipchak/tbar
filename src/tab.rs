use ansi_term::{ANSIString, ANSIStrings};
use unicode_width::UnicodeWidthStr;
use zellij_tile::prelude::*;
use zellij_tile_utils::style;

use crate::{Part, ARROW_SEPARATOR};

const MAX_TAB_COUNT: usize = 10000;

pub fn separator(capabilities: PluginCapabilities) -> &'static str {
  if capabilities.arrow_fonts {
    ""
  } else {
    ARROW_SEPARATOR
  }
}

pub fn collapsed(
  tab_count: usize,
  tab_index: usize,
  palette: Palette,
  separator: &str,
  left: bool,
) -> Part {
  if tab_count == 0 {
    return Part::default();
  }

  let (more_left, more_right) = if left { (" ←", "") } else { ("", "→ ") };
  let more_text = if tab_count < MAX_TAB_COUNT {
    format!("{more_left} +{tab_count} {more_right}")
  } else {
    format!("{more_left} +many {more_right}")
  };

  let (text_color, sep_color) = match palette.theme_hue {
    ThemeHue::Dark => (palette.white, palette.black),
    ThemeHue::Light => (palette.black, palette.white),
  };

  let separator_left = style!(sep_color, palette.orange).paint(separator);
  let separator_right = style!(palette.orange, sep_color).paint(separator);

  let more_text_styled =
    style!(text_color, palette.orange).bold().paint(&more_text);

  Part {
    part: ANSIStrings(&[separator_left, more_text_styled, separator_right])
      .to_string(),
    len: more_text.width() + 2 * separator.width(),
    tab_idx: Some(tab_index),
  }
}

fn cursors(focused_clients: &[ClientId], palette: Palette) -> Vec<ANSIString> {
  focused_clients
    .iter()
    .filter_map(|client_id| client_id_to_colors(*client_id, palette))
    .map(|color| style!(color.1, color.0).paint(" "))
    .collect()
}

pub fn render(
  text: &str,
  tab: &TabInfo,
  is_alternate_tab: bool,
  palette: Palette,
  separator: &str,
) -> Part {
  let bg_color = if tab.active {
    palette.green
  } else if is_alternate_tab {
    match palette.theme_hue {
      ThemeHue::Dark => palette.white,
      ThemeHue::Light => palette.black,
    }
  } else {
    palette.fg
  };
  let fg_color = match palette.theme_hue {
    ThemeHue::Dark => palette.black,
    ThemeHue::Light => palette.white,
  };

  let left_separator = style!(fg_color, bg_color).paint(separator);
  let right_separator = style!(bg_color, fg_color).paint(separator);

  let mut tab_text_len = text.width() + (separator.width() * 2) + 2; // + 2 for padding

  let tab_styled_text =
    style!(fg_color, bg_color).bold().paint(format!(" {text} "));

  let focused_clients = tab.other_focused_clients.as_slice();

  let tab_styled_text = if focused_clients.is_empty() {
    ANSIStrings(&[left_separator, tab_styled_text, right_separator]).to_string()
  } else {
    let cursor_section = cursors(focused_clients, palette);
    tab_text_len += cursor_section.len();

    let cursor_beginning = style!(fg_color, bg_color).bold().paint("[");
    let cursor_end = style!(fg_color, bg_color).bold().paint("]");
    let cursor_section = ANSIStrings(&cursor_section);

    format!("{left_separator}{tab_styled_text}{cursor_beginning}{cursor_section}{cursor_end}{right_separator}")
  };

  Part {
    part: tab_styled_text,
    len: tab_text_len,
    tab_idx: Some(tab.position),
  }
}

pub fn style(
  tabname: &str,
  tab: &TabInfo,
  is_alternate_tab: bool,
  palette: Palette,
  capabilities: PluginCapabilities,
) -> Part {
  render(
    &(tabname.to_string()
      + if tab.is_sync_panes_active {
        " (Sync)"
      } else {
        ""
      }),
    tab,
    capabilities.arrow_fonts && is_alternate_tab,
    palette,
    separator(capabilities),
  )
}
