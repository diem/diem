// Copyright (c) The Diem Core Contributors
// SPDX-License-Identifier: Apache-2.0

use plotters::{
    evcxr::SVGWrapper,
    prelude::{
        evcxr_figure, ChartBuilder, Color, IntoFont, LineSeries, Palette9999, PaletteColor,
        PathElement, SeriesLabelPosition, BLACK, WHITE,
    },
};
use std::collections::{BTreeMap, BinaryHeap};
use z3tracer::{
    syntax::{Ident, Term},
    Model,
};

/// Read z3tracer model from  file.
pub fn process_file(path: &str) -> std::io::Result<Model> {
    let file = std::io::BufReader::new(std::fs::File::open(path)?);
    // Inject non-default configurations here with Model::new(config).
    let mut model = Model::default();
    if let Err(le) = model.process(Some(path.to_string()), file) {
        println!("Error at {:?}: {:?}", le.position, le.error);
    }
    Ok(model)
}

/// Helper trait to print es by their id.
pub trait ModelExt {
    fn id2s(&self, id: &Ident) -> String;
}

impl ModelExt for Model {
    fn id2s(&self, id: &Ident) -> String {
        self.id_to_sexp(&BTreeMap::new(), id).unwrap()
    }
}

/// Compute top instantiated terms and retrieve the "timestamps" at which instantiations occur for each of the top terms.
pub fn compute_instantiation_times(model: &Model) -> Vec<(String, Vec<usize>)> {
    IntoIterSorted::from(model.most_instantiated_terms())
        .map(|(_count, id)| {
            let mut timestamps = model
                .term_data(&id)
                .unwrap()
                .instantiation_timestamps
                .clone();
            timestamps.sort_unstable();
            let name = match model.term(&id).unwrap() {
                Term::Quant { name, .. } | Term::Builtin { name: Some(name) } => name,
                _ => "??",
            };
            (name.to_string(), timestamps)
        })
        .collect()
}

/// Plot the instantiations.
pub fn plot_instantiations(
    times: &[(String, Vec<usize>)],
    title: &str,
    top_n: usize,
) -> SVGWrapper {
    let max_ts = times
        .iter()
        .map(|(_, v)| v.last().cloned().unwrap_or(0))
        .max()
        .unwrap_or(1);
    let max_count = times[0].1.len();

    evcxr_figure((1000, 800), |root| {
        root.fill(&WHITE)?;
        let mut chart = ChartBuilder::on(&root)
            .caption(title, ("Arial", 30).into_font())
            .margin(10)
            .x_label_area_size(40)
            .y_label_area_size(50)
            .build_cartesian_2d(0..max_ts, 0..max_count)?;

        chart
            .configure_mesh()
            .y_desc("Cumulative number of instantiations")
            .x_desc("Time (line number)")
            .draw()?;

        for (j, (name, values)) in times.iter().take(top_n).enumerate() {
            let color: PaletteColor<Palette9999> = PaletteColor::pick(j);
            chart
                .draw_series(LineSeries::new(
                    values.iter().enumerate().map(|(i, c)| (*c, i)),
                    &color,
                ))
                .unwrap()
                .label(name)
                .legend(move |(x, y)| PathElement::new(vec![(x, y), (x + 20, y)], color.filled()));
        }

        chart
            .configure_series_labels()
            .background_style(&WHITE.mix(0.8))
            .border_style(&BLACK)
            .position(SeriesLabelPosition::UpperLeft)
            .draw()?;
        Ok(())
    })
}

// TODO: remove after Rust issue 59278 is closed.
pub struct IntoIterSorted<T> {
    inner: BinaryHeap<T>,
}

impl<T> From<BinaryHeap<T>> for IntoIterSorted<T> {
    fn from(inner: BinaryHeap<T>) -> Self {
        Self { inner }
    }
}

impl<T: Ord> Iterator for IntoIterSorted<T> {
    type Item = T;

    fn next(&mut self) -> Option<T> {
        self.inner.pop()
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        let exact = self.inner.len();
        (exact, Some(exact))
    }
}
