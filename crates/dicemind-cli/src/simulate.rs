use std::{collections::HashMap, error::Error};

use dicemind::{interpreter::StandardNaiveRoller, syntax::Expression};
use rayon::iter::{IntoParallelIterator, ParallelIterator};
use textplots::{Chart, ColorPlot, Shape};

use crate::DisplayOptions;

#[derive(Debug, Default, Hash, PartialEq, Clone, Copy)]
pub struct SimulationOptions {
    pub trials: u64,
}

pub fn print_chart<'a>(
    display_options: DisplayOptions,
    frequency_tables: impl Iterator<Item = ((u8, u8, u8), &'a Vec<(i64, i64)>)>,
) {
    let DisplayOptions { height, width, .. } = display_options;

    for (color, table) in frequency_tables {
        let (min_x, max_x, max_y) = table
            .iter()
            .cloned()
            .fold((i64::MAX, i64::MIN, 0), |(xmi, xma, yma), (x, y)| {
                (xmi.min(x), xma.max(x), yma.max(y))
            });

        let values: Vec<_> = table
            .into_iter()
            .map(|&(a, b)| (a as f32, (b as f64 / max_y as f64) as f32))
            .collect();

        Chart::new_with_y_range(width, height, min_x as f32, max_x as f32, 0., 1.)
            .linecolorplot(&Shape::Steps(&values[..]), color.into())
            .nice();
    }
}
