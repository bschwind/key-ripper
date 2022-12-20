use core::{convert::Infallible, ops::Deref};

use cortex_m::delay::Delay;
use embedded_hal::digital::v2::InputPin;
use usbd_hid::descriptor::KeyboardReport;

use crate::{debounce::Debounce, key_codes::KeyCode, key_mapping};

#[derive(Clone, Copy)]
pub struct KeyScan<const NUM_ROWS: usize, const NUM_COLS: usize> {
    matrix: [[bool; NUM_ROWS]; NUM_COLS],
}

impl<const NUM_ROWS: usize, const NUM_COLS: usize> Deref for KeyScan<NUM_ROWS, NUM_COLS> {
    type Target = [[bool; NUM_ROWS]; NUM_COLS];

    fn deref(&self) -> &Self::Target {
        &self.matrix
    }
}

impl<const NUM_ROWS: usize, const NUM_COLS: usize> KeyScan<NUM_ROWS, NUM_COLS> {
    pub fn scan(
        rows: &[&dyn InputPin<Error = Infallible>],
        columns: &mut [&mut dyn embedded_hal::digital::v2::OutputPin<Error = Infallible>],
        delay: &mut Delay,
        debounce: &mut Debounce<NUM_ROWS, NUM_COLS>,
    ) -> Self {
        let mut raw_matrix = [[false; NUM_ROWS]; NUM_COLS];

        for (gpio_col, matrix_col) in columns.iter_mut().zip(raw_matrix.iter_mut()) {
            gpio_col.set_high().unwrap();
            delay.delay_us(10);

            for (gpio_row, matrix_row) in rows.iter().zip(matrix_col.iter_mut()) {
                *matrix_row = gpio_row.is_high().unwrap();
            }

            gpio_col.set_low().unwrap();
            delay.delay_us(10);
        }

        let matrix = debounce.report_and_tick(&raw_matrix);
        Self { matrix }
    }
}

impl<const NUM_ROWS: usize, const NUM_COLS: usize> From<KeyScan<NUM_ROWS, NUM_COLS>>
    for KeyboardReport
{
    fn from(scan: KeyScan<NUM_ROWS, NUM_COLS>) -> Self {
        let mut keycodes = [0u8; 6];
        let mut keycode_index = 0;
        let mut modifier = 0;

        let mut push_keycode = |key| {
            if keycode_index < keycodes.len() {
                keycodes[keycode_index] = key;
                keycode_index += 1;
            }
        };

        // First scan for any function keys being pressed
        let mut layer_mapping = key_mapping::NORMAL_LAYER_MAPPING;
        for (matrix_column, mapping_column) in scan.matrix.iter().zip(layer_mapping) {
            for (key_pressed, mapping_row) in matrix_column.iter().zip(mapping_column) {
                if mapping_row == KeyCode::Fn && *key_pressed {
                    layer_mapping = key_mapping::FN_LAYER_MAPPING;
                }
            }
        }

        // Second scan to generate the correct keycodes given the activated key map
        for (matrix_column, mapping_column) in scan.matrix.iter().zip(layer_mapping) {
            for (key_pressed, mapping_row) in matrix_column.iter().zip(mapping_column) {
                if *key_pressed {
                    if let Some(bitmask) = mapping_row.modifier_bitmask() {
                        modifier |= bitmask;
                    } else {
                        push_keycode(mapping_row as u8);
                    }
                }
            }
        }

        KeyboardReport { modifier, reserved: 0, leds: 0, keycodes }
    }
}
