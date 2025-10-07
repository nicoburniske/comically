use ratatui::{
    buffer::Buffer,
    crossterm::event::{self, KeyEvent, MouseEvent, MouseEventKind},
    layout::{Alignment, Constraint, Layout, Rect},
    style::{Color, Style, Stylize},
    text::Span,
    widgets::{Block, Gauge, Padding, Paragraph, StatefulWidget, Widget},
};

use std::time::{Duration, Instant};

use comically::OutputFormat;

use crate::tui::{
    render_title,
    utils::{themed_block, themed_block_title},
    Theme,
};

#[derive(Debug, Clone, Copy)]
pub enum ComicStage {
    Process,
    Package, // Building the output format (EPUB/CBZ)
    Convert, // Converting EPUB to MOBI (only for MOBI output)
}

impl std::fmt::Display for ComicStage {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ComicStage::Process => write!(f, "process"),
            ComicStage::Package => write!(f, "package"),
            ComicStage::Convert => write!(f, "convert"),
        }
    }
}

#[derive(Debug)]
pub enum ComicStatus {
    Waiting,
    Progress {
        stage: ComicStage,
        progress: f64,
        start: Instant,
    },
    ImageProcessingStart {
        start: Instant,
    },
    ImageProcessed,
    StageCompleted {
        stage: ComicStage,
        duration: Duration,
    },
    Success,
    Failed {
        error: anyhow::Error,
    },
}

pub enum ProgressEvent {
    RegisterComic { id: usize, file_name: String },
    ComicStats { id: usize, total_images: usize },
    ComicUpdate { id: usize, status: ComicStatus },
    ProcessingComplete,
}

pub struct ProgressState {
    start: Instant,
    comics: Vec<ComicState>,
    complete: Option<Duration>,
    scroll_offset: usize,
    pub theme: Theme,
    pub output_format: OutputFormat,
}

#[derive(Debug)]
struct ComicState {
    title: String,
    status: ComicStatus,
    timings: StageTimings,
    image_processing_start: Option<Instant>,
    images_processed: usize,
    total_images: usize,
}

#[derive(Debug, Clone)]
pub struct StageTimings {
    stages: Vec<StageMetrics>,
}

impl StageTimings {
    pub fn add_stage(&mut self, stage: ComicStage, duration: Duration) {
        self.stages.push(StageMetrics { stage, duration });
    }

    pub fn new() -> Self {
        Self { stages: Vec::new() }
    }

    pub fn total(&self) -> Duration {
        self.stages.iter().map(|s| s.duration).sum()
    }
}

#[derive(Debug, Clone)]
struct StageMetrics {
    stage: ComicStage,
    duration: Duration,
}

impl ComicState {
    fn current_status(&self) -> &ComicStatus {
        &self.status
    }
}

impl ProgressState {
    pub fn new(theme: Theme, output_format: OutputFormat) -> Self {
        Self {
            start: Instant::now(),
            comics: Vec::new(),
            complete: None,
            scroll_offset: 0,
            theme,
            output_format,
        }
    }

    pub fn handle_event(&mut self, event: ProgressEvent) {
        match event {
            ProgressEvent::RegisterComic { id, file_name } => {
                debug_assert!(self.comics.get(id).is_none(), "comic already registered");
                debug_assert!(id <= self.comics.len(), "id out of bounds");

                if id == self.comics.len() {
                    self.comics.push(ComicState {
                        title: file_name,
                        status: ComicStatus::Waiting,
                        timings: StageTimings::new(),
                        image_processing_start: None,
                        images_processed: 0,
                        total_images: 0,
                    });
                } else {
                    self.comics[id] = ComicState {
                        title: file_name,
                        status: ComicStatus::Waiting,
                        timings: StageTimings::new(),
                        image_processing_start: None,
                        images_processed: 0,
                        total_images: 0,
                    };
                }
            }
            ProgressEvent::ComicStats { id, total_images } => {
                if let Some(comic) = self.comics.get_mut(id) {
                    comic.total_images = total_images;
                }
            }
            ProgressEvent::ComicUpdate { id, status } => {
                if let Some(comic) = self.comics.get_mut(id) {
                    match &status {
                        ComicStatus::StageCompleted { stage, duration } => {
                            comic.timings.add_stage(*stage, *duration);
                            // Not storing this status
                            return;
                        }
                        ComicStatus::ImageProcessingStart { start } => {
                            comic.images_processed = 0;
                            comic.image_processing_start = Some(*start);
                        }
                        ComicStatus::ImageProcessed => {
                            comic.images_processed += 1;
                        }
                        _ => {}
                    }
                    comic.status = status;
                } else {
                    panic!("Comic state not found for id: {}", id);
                }
            }
            ProgressEvent::ProcessingComplete => {
                self.complete = Some(self.start.elapsed());
            }
        }
    }

    pub fn handle_key(&mut self, key: KeyEvent) {
        if key.code == event::KeyCode::Up || key.code == event::KeyCode::Char('k') {
            self.scroll_up();
        } else if key.code == event::KeyCode::Down || key.code == event::KeyCode::Char('j') {
            self.scroll_down();
        }
    }

    pub fn handle_mouse(&mut self, mouse: MouseEvent) {
        match mouse.kind {
            MouseEventKind::ScrollUp => {
                self.scroll_up();
            }
            MouseEventKind::ScrollDown => {
                self.scroll_down();
            }
            _ => {}
        }
    }

    fn scroll_up(&mut self) {
        if self.scroll_offset > 0 {
            self.scroll_offset = self.scroll_offset.saturating_sub(1);
        }
    }

    fn scroll_down(&mut self) {
        if !self.comics.is_empty() {
            self.scroll_offset = self.scroll_offset.saturating_add(1);
        }
    }
}

pub struct ProgressScreen<'a> {
    state: &'a mut ProgressState,
}

impl<'a> ProgressScreen<'a> {
    pub fn new(state: &'a mut ProgressState) -> Self {
        Self { state }
    }
}

impl<'a> Widget for ProgressScreen<'a> {
    fn render(self, area: Rect, buf: &mut Buffer) {
        buf.set_style(area, Style::default().bg(self.state.theme.background));

        let vertical = Layout::vertical([
            Constraint::Length(3),
            Constraint::Min(4),
            Constraint::Length(1),
        ])
        .margin(1);

        let [header_area, main_area, footer_area] = vertical.areas(area);

        let theme = self.state.theme;
        draw_header(buf, self.state, header_area, &theme);
        draw_main_content(buf, self.state, main_area, &theme);
        draw_footer(buf, self.state, footer_area, &theme);
    }
}

fn draw_header(buf: &mut Buffer, state: &ProgressState, header_area: Rect, theme: &Theme) {
    let [title_area, progress] =
        Layout::horizontal([Constraint::Percentage(15), Constraint::Percentage(85)])
            .areas(header_area);

    render_title(theme).render(title_area, buf);

    let total = state.comics.len();

    let successful = state
        .comics
        .iter()
        .filter(|state| matches!(state.current_status(), ComicStatus::Success))
        .count();

    let total_work: usize = state.comics.iter().map(|c| c.total_images).sum();
    let completed_work: usize = state.comics.iter().map(|c| c.images_processed).sum();

    let progress_ratio = if total_work > 0 {
        completed_work as f64 / total_work as f64
    } else {
        0.0
    };
    let elapsed = state.complete.unwrap_or_else(|| state.start.elapsed());

    Gauge::default()
        .gauge_style(Style::default().fg(theme.primary_bg))
        .label(Span::styled(
            format!("{}/{} ({:.1}s)", successful, total, elapsed.as_secs_f64()),
            Style::default().fg(theme.gauge_label),
        ))
        .ratio(progress_ratio)
        .block(themed_block(Some("progress"), theme))
        .render(progress, buf);
}

fn draw_main_content(buf: &mut Buffer, state: &mut ProgressState, area: Rect, theme: &Theme) {
    let [names_area, status_area] =
        Layout::horizontal([Constraint::Percentage(15), Constraint::Percentage(85)]).areas(area);

    let [status_area, scrollbar_area] =
        Layout::horizontal([Constraint::Min(0), Constraint::Length(1)])
            .spacing(1)
            .areas(status_area);

    let names_block = themed_block(Some("files"), theme);

    let status_block = themed_block(Some("status"), theme)
        .title(themed_block_title("total", theme).right_aligned());

    let names_inner_area = names_block.inner(names_area);
    let status_inner_area = status_block.inner(status_area);

    names_block.render(names_area, buf);
    status_block.render(status_area, buf);

    if state.comics.is_empty() {
        return;
    }

    let visible_height = names_inner_area.height as usize;

    let max_scroll = state.comics.len().saturating_sub(visible_height);
    if state.scroll_offset > max_scroll {
        state.scroll_offset = max_scroll;
    }

    let visible_items = {
        let end_idx = (state.scroll_offset + visible_height).min(state.comics.len());
        &state.comics[state.scroll_offset..end_idx]
    };

    let names_layout =
        Layout::vertical(vec![Constraint::Length(1); visible_items.len()]).split(names_inner_area);
    let status_layout =
        Layout::vertical(vec![Constraint::Length(1); visible_items.len()]).split(status_inner_area);

    for (i, comic) in visible_items.iter().enumerate() {
        draw_file_title(buf, comic, names_layout[i], theme);
    }

    for (i, comic) in visible_items.iter().enumerate() {
        draw_file_status(buf, comic, status_layout[i], theme);
    }

    draw_scrollbar(
        buf,
        state,
        scrollbar_area,
        state.comics.len(),
        visible_height,
        theme,
    );
}

fn draw_file_title(buf: &mut Buffer, comic_state: &ComicState, area: Rect, theme: &Theme) {
    Paragraph::new(comic_state.title.clone())
        .style(theme.content)
        .alignment(Alignment::Left)
        .block(Block::default().padding(Padding::horizontal(1)))
        .render(area, buf);
}

fn draw_file_status(buf: &mut Buffer, comic_state: &ComicState, area: Rect, theme: &Theme) {
    match comic_state.current_status() {
        ComicStatus::Waiting => {
            let label = Span::styled("waiting", Style::default().fg(theme.content));
            let gauge = Gauge::default()
                .gauge_style(theme.border)
                .ratio(0.0)
                .label(label);

            gauge.render(area, buf);
        }
        ComicStatus::Progress {
            stage,
            progress,
            start,
        } => {
            let elapsed = start.elapsed();
            let color = stage_color(*stage, theme);
            let label = Span::styled(
                format!("{} {:.1}s", stage, elapsed.as_secs_f64()),
                Style::default().fg(theme.gauge_label),
            );
            let gauge = Gauge::default()
                .gauge_style(Style::default().fg(color))
                .ratio(*progress / 100.0)
                .label(label);

            gauge.render(area, buf);
        }
        ComicStatus::ImageProcessingStart { .. } | ComicStatus::ImageProcessed => {
            let elapsed = comic_state
                .image_processing_start
                .map(|s| s.elapsed())
                .unwrap_or_default();
            let color = stage_color(ComicStage::Process, theme);
            let progress_ratio = if comic_state.total_images > 0 {
                comic_state.images_processed as f64 / comic_state.total_images as f64
            } else {
                0.0
            };
            let label = Span::styled(
                format!(
                    "{:3}/{:3} images {:.1}s",
                    comic_state.images_processed,
                    comic_state.total_images,
                    elapsed.as_secs_f64()
                ),
                Style::default().fg(theme.gauge_label),
            );
            let gauge = Gauge::default()
                .gauge_style(Style::default().fg(color))
                .ratio(progress_ratio)
                .label(label);

            gauge.render(area, buf);
        }
        ComicStatus::StageCompleted { .. } => {
            unreachable!("not storing this status")
        }
        ComicStatus::Success => {
            StageTimingBar::new(&comic_state.timings, theme)
                .width(area.width)
                .render(area, buf);
        }
        ComicStatus::Failed { error, .. } => {
            let error_text = error.to_string();
            let label = Span::styled(error_text, Style::default().fg(theme.content));

            let gauge = Gauge::default()
                .gauge_style(theme.error_bg)
                .ratio(1.0)
                .label(label);

            gauge.render(area, buf);
        }
    }
}

fn draw_scrollbar(
    buf: &mut Buffer,
    state: &mut ProgressState,
    area: Rect,
    total_items: usize,
    visible_height: usize,
    theme: &Theme,
) {
    let show_scrollbar = total_items > visible_height;
    if show_scrollbar {
        let mut scroll_state = ratatui::widgets::ScrollbarState::default()
            .content_length(total_items.saturating_sub(visible_height))
            .position(state.scroll_offset);

        StatefulWidget::render(
            ratatui::widgets::Scrollbar::default()
                .orientation(ratatui::widgets::ScrollbarOrientation::VerticalRight)
                .style(Style::default().fg(theme.border))
                .thumb_style(Style::default().fg(theme.scrollbar_thumb)),
            area,
            buf,
            &mut scroll_state,
        );
    }
}

fn draw_footer(buf: &mut Buffer, state: &ProgressState, area: Rect, theme: &Theme) {
    let show_scrollbar = !state.comics.is_empty();

    let [controls_area, legend_area] =
        Layout::horizontal([Constraint::Fill(1), Constraint::Fill(1)]).areas(area);

    let keys = if show_scrollbar {
        "↑/k: up | ↓/j: down | t: toggle theme | q: quit"
    } else {
        "t: toggle theme | q: quit"
    };

    let keys = Paragraph::new(keys)
        .style(theme.content)
        .alignment(ratatui::layout::Alignment::Center);

    keys.render(controls_area, buf);

    if !state.comics.is_empty() {
        draw_stage_legend(buf, legend_area, theme, state.output_format);
    }
}

fn draw_stage_legend(buf: &mut Buffer, area: Rect, theme: &Theme, output_format: OutputFormat) {
    let stages = match output_format {
        OutputFormat::Mobi => vec![
            ComicStage::Process,
            ComicStage::Package,
            ComicStage::Convert,
        ],
        OutputFormat::Epub => vec![ComicStage::Process, ComicStage::Package],
        OutputFormat::Cbz => vec![ComicStage::Process, ComicStage::Package],
    };

    let constraints = vec![Constraint::Length(16); stages.len()];

    let layout = Layout::horizontal(constraints).flex(ratatui::layout::Flex::Start);

    let areas = layout.split(area);

    for (i, &stage) in stages.iter().enumerate() {
        let color = stage_color(stage, theme);

        let [block_area, text_area] =
            Layout::horizontal([Constraint::Length(2), Constraint::Fill(1)]).areas(areas[i]);

        buf.set_style(block_area, Style::default().bg(color));

        Paragraph::new(stage.to_string())
            .style(theme.content)
            .alignment(Alignment::Left)
            .block(Block::default().padding(Padding::horizontal(1)))
            .render(text_area, buf);
    }
}

fn stage_color(stage: ComicStage, theme: &Theme) -> Color {
    match stage {
        ComicStage::Process => theme.stage_colors.process,
        ComicStage::Package => theme.stage_colors.epub,
        ComicStage::Convert => theme.stage_colors.mobi,
    }
}

struct StageTimingBar<'a> {
    timing: &'a StageTimings,
    theme: &'a Theme,
    width: u16,
}

impl<'a> StageTimingBar<'a> {
    fn new(timing: &'a StageTimings, theme: &'a Theme) -> Self {
        Self {
            timing,
            theme,
            width: 0,
        }
    }

    fn width(mut self, width: u16) -> Self {
        self.width = width;
        self
    }
}

impl<'a> Widget for StageTimingBar<'a> {
    fn render(self, area: Rect, buf: &mut Buffer) {
        let total = self.timing.total().as_secs_f64();
        if total == 0.0 || self.width == 0 {
            return;
        }

        let horizontal = Layout::horizontal([Constraint::Fill(4), Constraint::Length(15)])
            .flex(ratatui::layout::Flex::Start)
            .split(area);

        let bar_area = horizontal[0];
        let total_label_area = horizontal[1];

        if !self.timing.stages.is_empty() {
            // Create Fill constraints proportional to each stage's duration
            let constraints: Vec<Constraint> = self
                .timing
                .stages
                .iter()
                .map(|stage| {
                    Constraint::Fill((stage.duration.as_secs_f64() / total * 100.0).round() as u16)
                })
                .collect();

            let stage_areas = Layout::horizontal(&constraints)
                .flex(ratatui::layout::Flex::Start)
                .split(bar_area);

            for (stage, area) in self.timing.stages.iter().zip(stage_areas.iter()) {
                let color = stage_color(stage.stage, self.theme);

                buf.set_style(*area, Style::default().bg(color));

                if area.width >= 10 {
                    let label = format!("{:.1}s", stage.duration.as_secs_f64());

                    Paragraph::new(label)
                        .style(Style::default().fg(self.theme.gauge_label))
                        .alignment(ratatui::layout::Alignment::Center)
                        .render(*area, buf);
                }
            }
        }

        let total_label = format!("{:.1}s", total);

        Paragraph::new(total_label)
            .style(
                Style::default()
                    .fg(self.theme.gauge_label)
                    .bg(self.theme.primary_bg)
                    .bold(),
            )
            .alignment(ratatui::layout::Alignment::Center)
            .render(total_label_area, buf);
    }
}
