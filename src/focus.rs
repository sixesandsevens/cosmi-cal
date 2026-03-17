// SPDX-License-Identifier: MPL-2.0

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub enum FocusTarget {
    #[default]
    None,
    TodayNote,
    Scratchpad,
}
