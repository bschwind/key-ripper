//! A simple-as-possible key debouncer module to reduce undesired duplicate keypress
//! reports.

/// `Debounce` is a tick-based allocation-free "eager" (reports keypresses immediately)
/// debouncer.
///
/// # Algorithm
/// Its main purpose is to prevent rapid double-keypress events (i.e. when a key is
/// reported as not pressed, then immediately re-pressed). It does this by maintaining
/// an internal matrix of countdown ticks, where if a key is un-pressed and re-pressed
/// within `expiration` ticks, `Debounce` will report it as one continuous keypress.
///
/// # Ticks
/// Ticks are of arbitrary unit, with a configurable tick-count in which a repeat
/// keypress is suppressed. It's up to the user to handle the tick-to-millisecond conversion.
pub struct Debounce<const NUM_ROWS: usize, const NUM_COLS: usize> {
    /// The state matrix of debounce countdowns per-key.
    countdown_matrix: [[u8; NUM_ROWS]; NUM_COLS],

    /// The number of ticks to begin the debounce countdown from on a reported keypress.
    expiration_ticks: u8,
}

impl<const NUM_ROWS: usize, const NUM_COLS: usize> Debounce<NUM_ROWS, NUM_COLS> {
    /// Create a `Debounce` with a specified expiration tick amount.
    /// See struct documentation for what a "tick" means in this Debouncer.
    pub fn with_expiration(expiration_ticks: u8) -> Self {
        Self { countdown_matrix: [[0; NUM_ROWS]; NUM_COLS], expiration_ticks }
    }

    /// Report a new raw key scan matrix, expected to be called at a periodic "tick rate"
    /// corresponding to the same debouncing expiration tick amount specified in the
    /// constructor.
    pub fn report_and_tick(
        &mut self,
        report_matrix: &[[bool; NUM_ROWS]; NUM_COLS],
    ) -> [[bool; NUM_ROWS]; NUM_COLS] {
        let mut debounced_matrix = [[false; NUM_ROWS]; NUM_COLS];
        // Things got a bit hairy with iterators, writing this way for legibility.
        for col in 0..NUM_COLS {
            for row in 0..NUM_ROWS {
                let countdown_entry = &mut self.countdown_matrix[col][row];
                *countdown_entry = if report_matrix[col][row] {
                    self.expiration_ticks
                } else {
                    countdown_entry.saturating_sub(1)
                };
                debounced_matrix[col][row] = *countdown_entry != 0;
            }
        }

        debounced_matrix
    }
}
