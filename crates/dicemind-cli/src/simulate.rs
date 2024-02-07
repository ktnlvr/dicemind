use std::{collections::HashMap, error::Error};

use dicemind::{interpreter::StandardFastRoller, syntax::Expression};
use rayon::iter::{IntoParallelIterator, ParallelIterator};
use textplots::{Chart, ColorPlot, Shape};

use crate::DisplayOptions;

#[derive(Debug, Default, Hash, PartialEq, Clone, Copy)]
pub struct SimulationOptions {
    pub trials: u64,
}

pub fn simulate(
    expr: Expression,
    opts: SimulationOptions,
) -> Result<Vec<(i64, i64)>, Box<dyn Error + 'static>> {
    let SimulationOptions { trials } = opts;
    let values = (0..trials)
        .into_par_iter()
        .map(|_| StandardFastRoller::default().roll(expr.clone()).unwrap())
        .fold_with(HashMap::<i64, i64>::new(), |mut values, n| {
            *values.entry(n).or_insert_with(|| 0) += 1;
            values
        })
        .reduce_with(|mut a, b| {
            for (k, u) in b.into_iter() {
                let v = a.entry(k).or_insert_with(|| 0);
                *v += u;
            }
            a
        })
        .unwrap_or_default();

    let mut values: Vec<_> = values.into_iter().collect();
    values.sort_unstable_by_key(|(a, _)| *a);

    Ok(values)
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
