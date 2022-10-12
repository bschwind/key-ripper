//! A tick-based allocation-free "eager" (reports keypresses immediately) debouncer.
//! Ticks are of arbitrary unit, with a configurable tick-count in which a repeat
//! keypress is suppressed.
//!
//! It's up to the user to handle the tick-to-millisecond conversion.

pub struct Debounce<const NUM_ROWS: usize, const NUM_COLS: usize> {
    matrix: [[u8; NUM_ROWS]; NUM_COLS],
    expiration: u8,
}

const DEFAULT_EXPIRATION: u8 = 6;

impl<const NUM_ROWS: usize, const NUM_COLS: usize> Debounce<NUM_ROWS, NUM_COLS> {
    fn with_expiration(expiration: u8) -> Self {
        Self { matrix: [[0; NUM_ROWS]; NUM_COLS], expiration }
    }

    pub fn report_and_tick(
        &mut self,
        matrix: &[[bool; NUM_ROWS]; NUM_COLS],
    ) -> [[bool; NUM_ROWS]; NUM_COLS] {
        let mut debounced_matrix = [[false; NUM_ROWS]; NUM_COLS];
        // Things got a bit hairy with iterators, writing this way for legibility.
        for col in 0..NUM_COLS {
            for row in 0..NUM_ROWS {
                let expiration_key = &mut self.matrix[col][row];
                let report_key = matrix[col][row];
                *expiration_key = match (report_key, *expiration_key) {
                    // A new "true" keypress is recorded
                    (true, _) => self.expiration,
                    // No keypress detected
                    (false, 0) => *expiration_key,
                    // Continue expiring all previous keypresses
                    _ => *expiration_key - 1,
                };
                debounced_matrix[col][row] = *expiration_key != 0;
            }
        }

        // matrix.clone()
        debounced_matrix
    }
}

impl<const N: usize, const M: usize> Default for Debounce<N, M> {
    fn default() -> Self {
        Self::with_expiration(DEFAULT_EXPIRATION)
    }
}
