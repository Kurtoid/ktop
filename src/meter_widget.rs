use core::num;
use std::{
    time::{SystemTime, UNIX_EPOCH},
    usize,
};

use bytefmt::format;
use tui::{
    buffer::Buffer,
    layout::Rect,
    style::{Color, Style},
    symbols::bar,
    text::{Span, Spans},
    widgets::Widget,
};

use crate::zswap::ZswapStats;

pub struct MeterWidget {
    pub cpu_percent: f32,
    pub cpu_system_percent: f32,
    pub memory_percent: f32,
    pub swap_percent: f32,
    pub total_swap: u64,
    pub zswap_stats: Option<ZswapStats>,
}

impl Default for MeterWidget {
    fn default() -> MeterWidget {
        MeterWidget {
            cpu_percent: 0.0,
            cpu_system_percent: 0.0,
            memory_percent: 0.0,
            swap_percent: 0.0,
            total_swap: 0,
            zswap_stats: None,
        }
    }
}

impl Widget for MeterWidget {
    fn render(self, area: Rect, buf: &mut Buffer) {
        // TODO: make the bars here
        buf.set_spans(
            area.left(),
            area.top(),
            &make_bar(self.cpu_percent, area.width as usize / 2, "CPU".to_string()),
            area.width / 2,
        );
        buf.set_spans(
            area.left() + area.width / 2,
            area.top(),
            &make_bar(
                self.cpu_system_percent,
                area.width as usize / 2,
                "SCPU".to_string(),
            ),
            area.width / 2,
        );
        buf.set_spans(
            area.left(),
            area.top() + 1,
            &make_bar(
                self.memory_percent,
                area.width as usize / 2,
                "MEM".to_string(),
            ),
            area.width / 2,
        );
        buf.set_spans(
            area.left() + area.width / 2,
            area.top() + 1,
            &make_bar_with_label(
                self.swap_percent,
                area.width as usize / 2,
                "SWAP".to_string(),
                // this unit isn't correct - 1024 mb is displayed as 1.07 gb. for RAM, we want to use base 2, not base 10
                // again, just make my own converter
                bytefmt::format((self.total_swap as f32 * 1000f32 * 0.976562)as u64)
                    .replace(" ", "")
                    .replace("B", ""),
            ),
            area.width / 2,
        );
        if self.zswap_stats.is_some() {
            let stats = self.zswap_stats.unwrap();
            let zswap_size = stats.pool_total_size;
            buf.set_string(
                area.left(),
                area.top() + 2,
                bytefmt::format(zswap_size).replace(' ', "").replace('B', ""),
                Style::default(),
            );
        }

        /*
        let start = SystemTime::now();
        let since_the_epoch = start
            .duration_since(UNIX_EPOCH)
            .expect("Time went backwards");
        buf.set_spans(
            area.left(),
            area.top() + 1,
            &make_bar_with_label(
                (since_the_epoch.as_secs() % 10) as f32 / 10f32,
                area.width as usize / 2,
                "SCPU".to_string(),
                "Some long string".to_string(),
            ),
            area.width / 2,
        );
        */
    }
}

fn make_bar<'a>(percentage: f32, width: usize, label: String) -> Spans<'a> {
    make_bar_with_label(percentage, width, label, String::from(""))
}
const LABEL_WIDTH: usize = 5;
fn make_bar_with_label<'a>(
    percentage: f32,
    width: usize,
    label: String,
    inner_label_prefix: String,
) -> Spans<'a> {
    // case 1: bar + space + label
    //        |---|
    // case 2: bar + label
    //        |--------|   some (or all) of label is colored
    // TODO: handle case 2. for now, the label hides the bar
    let bar_width = width - LABEL_WIDTH - 2 - 2;
    let percent_label = format!(
        "{:width$}{:3.1}%",
        inner_label_prefix,
        percentage * 100f32,
        width = inner_label_prefix.len() + 1
    );
    let num_filled_blocks = f32::round(bar_width as f32 * percentage) as usize;
    let mut bar_spans = vec![
        Span::styled(format!("{:1$}[", label, LABEL_WIDTH), Style::default()),
        Span::styled(
            format!(
                "{:1$}",
                tui::symbols::line::VERTICAL
                    .repeat(num_filled_blocks.min(bar_width - percent_label.len())),
                bar_width - percent_label.len()
            ),
            // TODO: the blue is just for debugging
            Style::default().fg(Color::Red),
        ),
    ];
    bar_spans.push(Span::styled(percent_label, Style::default()));
    bar_spans.push(Span::styled("] ", Style::default()));

    Spans::from(bar_spans)
}
