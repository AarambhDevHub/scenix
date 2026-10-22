use std::string::String;
use std::vec::Vec;

use scenix_loader::AssetLoadStatus;

/// Upper bound of rows a browser model retains.
pub const BROWSER_ROW_LIMIT: usize = 256;

/// Truncation length for browser labels (paths can be long).
pub const BROWSER_LABEL_CAP: usize = 64;

/// Coarse status class a host paints a badge from.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum AssetStatusClass {
    /// Queued, not started.
    Pending,
    /// Loading with a `0..=1` progress ratio.
    Loading,
    /// Fully resident.
    Loaded,
    /// Explicitly cancelled by the host.
    Cancelled,
    /// failed with a diagnostic.
    Failed,
}

impl From<&AssetLoadStatus> for AssetStatusClass {
    fn from(status: &AssetLoadStatus) -> Self {
        match status {
            AssetLoadStatus::Pending => Self::Pending,
            AssetLoadStatus::Loading { .. } => Self::Loading,
            AssetLoadStatus::Loaded => Self::Loaded,
            AssetLoadStatus::Cancelled => Self::Cancelled,
            AssetLoadStatus::Failed(_) => Self::Failed,
        }
    }
}

/// One flattened browser row over the loader's statuses.
#[derive(Clone, Debug, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct AssetBrowserRow {
    /// Display label: end of the request path (truncated to cap).
    pub label: String,
    /// Coarse status class.
    pub status: AssetStatusClass,
    /// Progress ratio for loading rows (`0..=1`, `1.0` otherwise).
    pub progress: f32,
}

impl AssetBrowserRow {
    /// Build a row from a request label plus the loader's current status.
    ///
    /// Labels truncate at [`BROWSER_LABEL_CAP`] and progress clamps into
    /// `[0, 1]`, so a hostile loader report cannot break the panel.
    pub fn new(label: impl Into<String>, status: &AssetLoadStatus) -> Self {
        let label: String = label.into();
        let label = if label.chars().count() > BROWSER_LABEL_CAP {
            label.chars().take(BROWSER_LABEL_CAP).collect()
        } else {
            label
        };
        let progress = match status {
            AssetLoadStatus::Loading { progress } => progress.clamp(0.0, 1.0),
            // Failed rows report progress zero so retry bars start clean.
            AssetLoadStatus::Failed(_) => 0.0,
            _ => 1.0,
        };
        Self {
            label,
            status: AssetStatusClass::from(status),
            progress,
        }
    }
}

/// The asset browser model: rows plus active filter state.
///
/// Rows are host-fed from the loader watch set (the loader intentionally has
/// no global registry — hosts already track their own requests). Filtering
/// is a plain, allocation-light substring + status predicate.
#[derive(Clone, Debug, Default)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct AssetBrowserModel {
    rows: Vec<AssetBrowserRow>,
    filter: String,
    status_filter: Option<AssetStatusClass>,
}

impl AssetBrowserModel {
    /// Empty browser.
    pub fn new() -> Self {
        Self::default()
    }

    /// Push one row; drops the row silently past the cap.
    pub fn push_row(&mut self, label: impl Into<String>, status: &AssetLoadStatus) -> bool {
        if self.rows.len() >= BROWSER_ROW_LIMIT {
            return false;
        }
        self.rows.push(AssetBrowserRow::new(label, status));
        true
    }

    /// Replace the whole row set (per-frame sync), respecting the cap.
    pub fn sync(&mut self, rows: Vec<(String, AssetLoadStatus)>) {
        self.rows.clear();
        for (label, status) in rows {
            if !self.push_row(label, &status) {
                break;
            }
        }
    }

    /// All retained rows, unfiltered.
    pub fn rows(&self) -> &[AssetBrowserRow] {
        &self.rows
    }

    /// Active substring filter (empty means show-all).
    pub fn filter(&self) -> &str {
        &self.filter
    }

    /// Set the case-insensitive substring filter.
    pub fn set_filter(&mut self, filter: impl Into<String>) {
        self.filter = filter.into();
    }

    /// Active status filter, if any.
    pub fn status_filter(&self) -> Option<AssetStatusClass> {
        self.status_filter
    }

    /// Filter by status class.
    pub fn set_status_filter(&mut self, status: Option<AssetStatusClass>) {
        self.status_filter = status;
    }

    /// Rows matching the current filters, in retention order.
    pub fn visible(&self) -> impl Iterator<Item = &AssetBrowserRow> {
        let filter = self.filter.trim().to_lowercase();
        let status = self.status_filter;
        self.rows.iter().filter(move |row| {
            let text_ok = filter.is_empty() || row.label.to_lowercase().contains(&filter);
            let status_ok = status.is_none_or(|class| row.status == class);
            text_ok && status_ok
        })
    }

    /// Count rows in one status class.
    pub fn count_by(&self, class: AssetStatusClass) -> usize {
        self.rows.iter().filter(|row| row.status == class).count()
    }
}
