use std::{f64::consts::PI, slice::Chunks};

use chrono::{DateTime, Datelike, Month, Weekday};
use leptos::{html::Canvas, *};
use leptos_use::{use_event_listener, use_resize_observer};
use tracing::{debug, error, trace};
use wasm_bindgen::{JsCast, JsValue};
use web_sys::CanvasRenderingContext2d;

#[derive(Debug, Clone)]
pub struct Graph {
    data: Vec<f64>,
    chunk_size: usize,
    chunk_value_index: usize,
    median_amount: usize,
    line_height: f64,
    padding: f64,
    line_width: f64,
    point_radius: f64,
    width: f64,
    height: f64,
    canvas_width_ratio: f64,
    canvas_height_ratio: f64,
    min_x: f64,
    min_y: f64,
    max_x: f64,
    max_y: f64,
    font: &'static str,
    color_brightest: JsValue,
    color_lowest: JsValue,
    color_accent: JsValue,
    color_lowest_accent: JsValue,
    medians: Vec<GraphText>,
    //parts: Vec<GraphPart>,
    lines: Vec<GraphLine>,
    lines_bg: Vec<GraphBgLine>,
    points: Vec<GraphPoint>,
}

impl Default for Graph {
    fn default() -> Self {
        Self {
            data: Vec::new(),
            line_height: 15.0,
            padding: 100.0,
            line_width: 2.0,
            point_radius: 2.0,
            chunk_size: 2,
            chunk_value_index: 1,
            median_amount: 10,
            width: 0.0,
            height: 0.0,
            canvas_width_ratio: 0.0,
            canvas_height_ratio: 0.0,
            min_x: 0.0,
            min_y: 0.0,
            max_x: 0.0,
            max_y: 0.0,
            font: "0.7rem Arial",
            color_brightest: JsValue::from_str("#FFFFFF"),
            color_lowest: JsValue::from_str("#ffffff1A"),
            color_accent: JsValue::from_str("#925CB3"),
            color_lowest_accent: JsValue::from_str("#925CB34D"),
            medians: Vec::new(),
            //parts: Vec::new(),
            lines: Vec::new(),
            points: Vec::new(),
            lines_bg: Vec::new(),
        }
    }
}

impl Graph {
    pub fn new() -> Self {
        Self {
            ..Self::default()
        }
    }

    pub fn set_data(&mut self, data: Vec<f64>) {
        self.data = data;
    }

    pub fn calculate(&mut self, ctx: &CanvasRenderingContext2d, width: f64, height: f64) {
        let graph = self;
        let data = &graph.data;
        let chunk_size = graph.chunk_size;
        let chunk_value_index = graph.chunk_value_index;
        let median_amount = graph.median_amount;

        let (min_x, max_x, min_y, max_y) = get_min_max(data.chunks(chunk_size).clone());
                let diff_x = (max_x - min_x);
                let diff_x = if diff_x <= 0.0 { 1.0 } else { diff_x };

                let diff_y = (max_y - min_y);
                let diff_y = if diff_y <= 0.0 { 1.0 } else { diff_y };


                let (canvas_width_ratio, canvas_height_ratio) = (
                    (width - graph.padding) / diff_x,
                    (height - graph.padding) / diff_y,
                );

                graph.max_x = max_x;
                graph.max_y = max_y;
                graph.min_x = min_x;
                graph.min_y = min_y;
                graph.width = width;
                graph.height = height;
                graph.canvas_width_ratio = canvas_width_ratio;
                graph.canvas_height_ratio = canvas_height_ratio;

                graph.medians = GraphText::new_medians(
                    &ctx,
                    data,
                    chunk_size,
                    chunk_value_index,
                    median_amount,
                    graph.padding,
                    height as f64,
                );
                graph.lines = GraphLine::from_vec(
                    &ctx,
                    data,
                    chunk_size,
                    chunk_value_index,
                    canvas_width_ratio,
                    canvas_height_ratio,
                    max_x,
                    max_y,
                    min_x,
                    min_y,
                    median_amount,
                    graph.padding,
                    graph.line_height,
                    graph.point_radius,
                    height as f64,
                    width as f64,
                );
                graph.lines_bg = GraphBgLine::from_vec(
                    &ctx,
                    data,
                    chunk_size,
                    chunk_value_index,
                    canvas_width_ratio,
                    canvas_height_ratio,
                    max_x,
                    max_y,
                    min_x,
                    min_y,
                    median_amount,
                    graph.padding,
                    graph.line_height,
                    graph.point_radius,
                    height as f64,
                    width as f64,
                    graph.color_brightest.clone(),
                    graph.color_accent.clone(),
                    graph.color_lowest.clone(),
                    graph.color_lowest_accent.clone(),
                );
                graph.points = GraphPoint::from_vec(
                    &ctx,
                    data,
                    chunk_size,
                    chunk_value_index,
                    canvas_width_ratio,
                    canvas_height_ratio,
                    max_x,
                    max_y,
                    min_x,
                    min_y,
                    median_amount,
                    graph.padding,
                    graph.line_height,
                    graph.point_radius,
                    height as f64,
                    width as f64,
                );
                //debug!("graph: {:#?}", &graph);
    }

    pub fn draw(&self, ctx: &CanvasRenderingContext2d, mouse_pos: Option<(f64, f64)>) {
       

        let graph = self;
        let height = self.height;
        let width = self.width;
        let canvas_width_ratio = self.canvas_width_ratio;
        let canvas_height_ratio = self.canvas_height_ratio;

        ctx.clear_rect(0.0, 0.0, width, height);
        ctx.set_font(graph.font);
        ctx.set_line_width(graph.line_width);
        
        let closest_point = mouse_pos.and_then(|mouse_pos| {
            get_closest_point(
                self.data.chunks(graph.chunk_size).clone(),
                canvas_width_ratio,
                canvas_height_ratio,
                graph.min_x,
                graph.min_y,
                width,
                height,
                graph.padding,
                mouse_pos,
            )
        });

        ctx.set_fill_style(&graph.color_brightest);
        for median in &graph.medians {
            ctx.fill_text(&median.text, median.x, median.y);
        }

        
        for line in &graph.lines_bg {
            ctx.set_stroke_style(&line.color_line);
            ctx.set_fill_style(&line.color_text);

            ctx.begin_path();
            ctx.move_to(line.from_x, line.from_y);
            ctx.line_to(line.to_x, line.to_y);
            ctx.stroke();
            
            let Some(text) = &line.text else {
                continue;
            };

            for text in text {
                ctx.fill_text(&text.text, text.x, text.y);
            }
        }
        ctx.set_fill_style(&graph.color_brightest);
        ctx.set_stroke_style(&graph.color_accent);
        ctx.begin_path();
        for line in &graph.lines {
            ctx.move_to(line.from_x, line.from_y);
            ctx.line_to(line.to_x, line.to_y);
        }
        ctx.stroke();
        ctx.set_stroke_style(&graph.color_brightest);

        for (i, point) in graph.points.iter().enumerate() {
            ctx.begin_path();
            ctx.arc(
                point.x,
                point.y,
                graph.point_radius,
                0.0,
                PI * 2.0,
            );
            ctx.fill();

            for text in &point.point_text {
                ctx.fill_text(&text.text, text.x, text.y);
            }

            let Some((closest_i, mouse_x, mouse_y)) = closest_point else {
                continue;
            };

            if closest_i != i {
                continue;
            }

            let len = point.mouse_text.len();
            for (i, text) in point.mouse_text.iter().enumerate() {
                let Ok(text_w) = ctx
                    .measure_text(text)
                    .inspect_err(|err| error!("failed to measure text: {:?}", err))
                else {
                    return;
                };

                ctx.begin_path();
                ctx.move_to(point.x, point.y);
                ctx.line_to(mouse_x, mouse_y);
                ctx.stroke();
        
                let text_x = mouse_x - (text_w.width() / 2.0);
                let text_y = mouse_y - graph.line_height * (len - i) as f64;
                ctx.fill_text(&text, text_x, text_y);
            }
        }

       
    }

}




#[derive(Debug, Clone)]
pub struct GraphPoint {
    x: f64,
    y: f64,
    point_text: Vec<GraphText>,
    mouse_text: Vec<String>,
}

impl GraphPoint {
    pub fn from_vec(
        ctx: &CanvasRenderingContext2d,
        data: &[f64],
        chunksize: usize,
        chunk_value_index: usize,
        canvas_width_ratio: f64,
        canvas_height_ratio: f64,
        max_x: f64,
        max_y: f64,
        min_x: f64,
        min_y: f64,
        amount: usize,
        padding: f64,
        line_height: f64,
        point_radius: f64,
        height: f64,
        width: f64,
    ) -> Vec<Self> {
        let mut lines: Vec<Self> = Vec::new();
        let data = data.chunks(chunksize);

        for (i, chunk) in data.enumerate() {
            let org_x = chunk.get(0).cloned();
            let org_y = chunk.get(1).cloned().unwrap_or(0.0);
            let Some(org_x) = org_x else {
                break;
            };

            let point_x = (org_x - min_x) * canvas_width_ratio + (padding / 2.0);
            let point_y = (height - ((org_y - min_y) * canvas_height_ratio)) - (padding / 2.0);

            lines.push(GraphPoint {
                x: point_x,
                y: point_y,
                point_text: GraphText::new_center(ctx, point_x, point_y - point_radius * 4.0,line_height, &format!("{:.0}", org_y)),
                mouse_text: format!("Connections: {:.0}\n{}", org_y, DateTime::from_timestamp_millis(org_x as i64).unwrap_or_default()).split("\n").map(|v|v.to_string()).collect::<Vec<String>>(),
            });
        }

        lines
    }
}

#[derive(Debug, Clone)]
pub struct GraphLine {
    from_x: f64,
    from_y: f64,
    to_x: f64,
    to_y: f64,
}

impl GraphLine {
    pub fn from_vec(
        ctx: &CanvasRenderingContext2d,
        data: &[f64],
        chunksize: usize,
        chunk_value_index: usize,
        canvas_width_ratio: f64,
        canvas_height_ratio: f64,
        max_x: f64,
        max_y: f64,
        min_x: f64,
        min_y: f64,
        amount: usize,
        padding: f64,
        line_height: f64,
        point_radius: f64,
        height: f64,
        width: f64,
    ) -> Vec<Self> {
        let mut lines: Vec<Self> = Vec::new();
        let data = data.chunks(chunksize);

        let mut prev_point: Option<(f64, f64, f64, f64)> = None;
        for (i, chunk) in data.enumerate() {
            let org_x = chunk.get(0).cloned();
            let org_y = chunk.get(1).cloned().unwrap_or(0.0);
            let Some(org_x) = org_x else {
                break;
            };

            let point_x = (org_x - min_x) * canvas_width_ratio + (padding / 2.0);
            let point_y = (height - ((org_y - min_y) * canvas_height_ratio)) - (padding / 2.0);

            let Some((prev_point_x, prev_point_y, prev_org_x, prev_org_y)) = prev_point else {
                prev_point = Some((point_x, point_y, org_x, org_y));
                continue;
            };

            lines.push(GraphLine {
                from_x: prev_point_x,
                from_y: prev_point_y,
                to_x: point_x,
                to_y: point_y,
            });

            prev_point = Some((point_x, point_y, org_x, org_y));
        }

        lines
    }
}

#[derive(Debug, Clone)]
pub struct GraphBgLine {
    from_x: f64,
    from_y: f64,
    to_x: f64,
    to_y: f64,
    color_text: JsValue,
    color_line: JsValue,
    text: Option<Vec<GraphText>>,
}

impl GraphBgLine {
    pub fn from_vec(
        ctx: &CanvasRenderingContext2d,
        data: &[f64],
        chunksize: usize,
        chunk_value_index: usize,
        canvas_width_ratio: f64,
        canvas_height_ratio: f64,
        max_x: f64,
        max_y: f64,
        min_x: f64,
        min_y: f64,
        amount: usize,
        padding: f64,
        line_height: f64,
        point_radius: f64,
        height: f64,
        width: f64,
        color_text_one: JsValue,
        color_text_two: JsValue,
        color_line_one: JsValue,
        color_line_two: JsValue,
    ) -> Vec<Self> {
        let mut lines: Vec<Self> = Vec::new();
        let data = data.chunks(chunksize);
        let data_len = data.len();

        let mut prev_month: Option<&'static str> = None;
        for (i, chunk) in data.enumerate() {
            let org_x = chunk.get(0).cloned();
            let org_y = chunk.get(1).cloned().unwrap_or(0.0);
            let Some(org_x) = org_x else {
                break;
            };

            let point_x = (org_x - min_x) * canvas_width_ratio + (padding / 2.0);
            let point_y = (height - ((org_y - min_y) * canvas_height_ratio)) - (padding / 2.0);

            let point_line_y_start = padding / 3.0;
            let point_line_y_end = height - (padding / 2.5);
            let point_text_y = height - (padding / 4.0);

            let x_as_date = DateTime::from_timestamp_millis(org_x as i64);
            let Some(x_as_date) = x_as_date else {
                continue;
            };
            let weekday = x_as_date.weekday();

            let (color_text, color_line) = match weekday {
                Weekday::Sat | Weekday::Sun => (color_text_two.clone(), color_line_two.clone()),
                _ => (color_text_one.clone(), color_line_one.clone()),
            };


            let text: Option<Vec<GraphText>> = if data_len / 7 > 5 {
                
                let month = u8::try_from(x_as_date.month())
                    .ok()
                    .and_then(|month| Month::try_from(month).ok())
                    .map(|month| month.name())
                    .unwrap_or(".");
                let point_text_y = height - (padding / 4.0);
                let same_month = prev_month
                    .map(|prev_month| prev_month == month)
                    .unwrap_or(false);
                if !same_month {
                    let text = GraphText::new_right(ctx, point_x, point_text_y,line_height, month);
     
                    prev_month = Some(month);
                    Some(text)
                } else {
                    None
                }
            } else {
                let text = match weekday {
                    Weekday::Mon => "M",
                    Weekday::Tue => "T",
                    Weekday::Thu => "T",
                    Weekday::Wed => "W",
                    Weekday::Fri => "F",
                    Weekday::Sat => "S",
                    Weekday::Sun => "S",
                };
                Some(GraphText::new_center(ctx, point_x, point_text_y,line_height, text))
            };

            lines.push(GraphBgLine {
                from_x: point_x,
                from_y: point_line_y_start,
                to_x: point_x,
                to_y: point_line_y_end,
                color_text,
                color_line,
                text,
            });
        }

        lines
    }
}

#[derive(Debug, Clone)]
pub struct GraphText {
    x: f64,
    y: f64,
    text: String,
}

impl GraphText {
    pub fn new_medians(
        ctx: &CanvasRenderingContext2d,
        value: &[f64],
        chunksize: usize,
        chunk_value_index: usize,
        amount: usize,
        padding: f64,
        height: f64,
    ) -> Vec<Self> {
        let data = value.chunks(chunksize);
        let mut medians: Vec<Self> = Vec::with_capacity(amount);

        let mut prev_value: Option<f64> = None;
        let mut data = data
            .clone()
            .map(|data| data.get(chunk_value_index).cloned().unwrap_or(0.0))
            .collect::<Vec<f64>>();
        data.sort_by(|a, b| b.partial_cmp(a).unwrap_or(std::cmp::Ordering::Equal));
        data.retain(|value| {
            let same = prev_value.map(|prev_y| prev_y == *value).unwrap_or(false);
            prev_value = Some(*value);
            !same
        });

        let data_len = data.len();

      

        let median_step = data_len / amount;
        let median_step = if median_step == 0 { 1 } else { median_step };
        let median_len = data.iter().step_by(median_step).count();
    
        let x = padding / 4.0;
        let mut y = padding / 2.0;
        let text_step = (height - padding / 2.0) / median_len as f64;

        for value in data.into_iter().step_by(median_step) {
            let text = (value as i64).to_string();
            let Ok(text_w) = ctx
                .measure_text(&text)
                .inspect_err(|err| error!("failed to measure text: {:?}", err))
            else {
                return vec![];
            };

            let x = x - (text_w.width() / 2.0);

            medians.push(GraphText {
                x,
                y,
                text,
            });
            y += text_step;
        }

        medians
    }

    pub fn new_center(
        ctx: &CanvasRenderingContext2d,
        x: f64,
        y: f64,
        line_height: f64,
        text: &str,
    ) -> Vec<GraphText> {
        let mut output: Vec<GraphText> = Vec::new();
        let text = text.split("\n");
        let texts: Vec<&str> = text.collect();
        let len = texts.len().checked_sub(1).unwrap_or(1);
        for (i, text) in texts.iter().enumerate() {
            let Ok(text_w) = ctx
                .measure_text(text)
                .inspect_err(|err| error!("failed to measure text: {:?}", err))
            else {
                return vec![];
            };

            let text_x = x - (text_w.width() / 2.0);
            let text_y = y - line_height * (len - i) as f64;

            output.push(GraphText {
                x: text_x,
                y: text_y,
                text: text.to_string(),
            });
        }
        output
    }

    pub fn new_left(
        ctx: &CanvasRenderingContext2d,
        x: f64,
        y: f64,
        line_height: f64,
        text: &str,
    ) -> Vec<GraphText> {
        let mut output: Vec<GraphText> = Vec::new();
        let text = text.split("\n");
        let texts: Vec<&str> = text.collect();
        let len = texts.len().checked_sub(1).unwrap_or(1);
        for (i, text) in texts.iter().enumerate() {
            let text_x = x;
            let text_y = y - line_height * (len - i) as f64;

            output.push(GraphText {
                x: text_x,
                y: text_y,
                text: text.to_string(),
            });
        }
        output
    }

    pub fn new_right(
        ctx: &CanvasRenderingContext2d,
        x: f64,
        y: f64,
        line_height: f64,
        text: &str,
    ) -> Vec<GraphText> {
        let mut output: Vec<GraphText> = Vec::new();
        let text = text.split("\n");
        let texts: Vec<&str> = text.collect();
        let len = texts.len().checked_sub(1).unwrap_or(1);
        for (i, text) in texts.iter().enumerate() {
            let Ok(text_w) = ctx
            .measure_text(text)
            .inspect_err(|err| error!("failed to measure text: {:?}", err))
            else {
                return vec![];
            };
            let text_x = x + text_w.width();
            let text_y = y - line_height * (len - i) as f64;

            output.push(GraphText {
                x: text_x,
                y: text_y,
                text: text.to_string(),
            });
        }
        output
    }
}

pub fn use_graph() -> (NodeRef<Canvas>, RwSignal<Vec<f64>>) {
    let canvas_ref = NodeRef::new();
    let data = RwSignal::new(vec![0.0, 0.0, 10.0, 10.0, 20.0, 10.0]);
    let graph = RwSignal::new(Graph::new());

    let _ = watch(
        move || data.get(),
        move |value, prev_num, _| {
            let canvas_elm: Option<HtmlElement<Canvas>> = canvas_ref.get_untracked();
            let Some(canvas) = canvas_elm else {
                error!("error getting canvas context ");
                return;
            };
            let Some((ctx, width, height)) = get_ctx(canvas) else {
                return;
            };

            graph.update(move |graph| {
                graph.set_data(value.clone());
                graph.calculate(&ctx, width, height);
                graph.draw(&ctx, None);
            });
        },
        false,
    );

    use_resize_observer(canvas_ref, move |entries, observer| {
        let canvas_elm: Option<HtmlElement<Canvas>> = canvas_ref.get_untracked();
        let Some(canvas) = canvas_elm else {
            error!("error getting canvas context ");
            return;
        };

        let rect = entries[0].content_rect();
        let width = rect.width();
        let height = rect.height();

        canvas.set_width(width as u32);
        canvas.set_height(height as u32);

        
        let Some((ctx, width, height)) = get_ctx(canvas) else {
            return;
        };
        graph.update_untracked(|graph| {
            graph.calculate(&ctx, width, height);
            graph.draw(&ctx, None);
        });
    });

    let _ = use_event_listener(canvas_ref, ev::mousemove, move |ev| {
        let Some(canvas) = canvas_ref.get_untracked() else {
            error!("error getting canvas context ");
            return;
        };

        let Some((ctx, width, height)) = get_ctx(canvas) else {
            return;
        };
        graph.with_untracked(|graph| {
            graph.draw(&ctx, Some((ev.offset_x() as f64, ev.offset_y() as f64)));
        });
    });

    let _ = use_event_listener(canvas_ref, ev::mouseleave, move |ev| {
        let Some(canvas) = canvas_ref.get_untracked() else {
            error!("error getting canvas context ");
            return;
        };

        let Some((ctx, width, height)) = get_ctx(canvas) else {
            return;
        };
        graph.with_untracked(|graph| {
            graph.draw(&ctx, None);
        });
    });

    (canvas_ref, data)
}

fn get_closest_point(
    data: Chunks<f64>,
    canvas_x: f64,
    canvas_y: f64,
    min_x: f64,
    min_y: f64,
    width: f64,
    height: f64,
    padding: f64,
    (mouse_x, mouse_y): (f64, f64),
) -> Option<(usize, f64, f64)> {
    let mut closest_point: Option<usize> = None;
    let mut distance: Option<f64> = None;
    for (i, chunk) in data.enumerate() {
        let x = chunk.get(0).cloned();
        let y = chunk.get(1).cloned().unwrap_or(0.0);
        let Some(x) = x else {
            break;
        };

        let x = (x - min_x) * canvas_x + (padding / 2.0);
        let y = (height - ((y - min_y) * canvas_y)) - (padding / 2.0);

        let new_distance = ((x - mouse_x).powi(2) + (y - mouse_y).powi(2)).sqrt();
        if let Some(text_render_distance) = &mut distance {
            if *text_render_distance > new_distance {
                *text_render_distance = new_distance;
                closest_point = Some(i);
            }
        } else {
            distance = Some(new_distance);
            closest_point = Some(i);
        }
    }

    closest_point.map(|closest_point| (closest_point, mouse_x, mouse_y))
}

pub fn get_min_max(data: Chunks<f64>) -> (f64, f64, f64, f64) {
    let mut min_x = data
        .clone()
        .into_iter()
        .next()
        .and_then(|item| item.first().cloned())
        .unwrap_or(0.0);
    let mut max_x = 0.0;
    let mut min_y = data
        .clone()
        .into_iter()
        .next()
        .and_then(|item| item.get(1).cloned())
        .unwrap_or(0.0);
    let mut max_y = 0.0;
    for (i, chunk) in data.enumerate() {
        let x = chunk.get(0).cloned();
        let y = chunk.get(1).cloned().unwrap_or(0.0);
        let Some(x) = x else {
            break;
        };

        if x > max_x {
            max_x = x;
        }

        if y > max_y {
            max_y = y;
        }

        if x < min_x {
            min_x = x;
        }

        if y < min_y {
            min_y = y;
        }
    }
    (min_x, max_x, min_y, max_y)
}

fn get_ctx(canvas: HtmlElement<Canvas>) -> Option<(CanvasRenderingContext2d, f64, f64)> {

    let ctx = canvas.get_context("2d");
    let ctx = match ctx {
        Ok(ctx) => ctx,
        Err(err) => {
            error!("error getting canvas context {:?}", err);
            return None;
        }
    };
    let Some(ctx) = ctx else {
        error!("error getting canvas context ");
        return None;
    };

    let ctx = ctx.dyn_into::<web_sys::CanvasRenderingContext2d>();
    let ctx = match ctx {
        Ok(ctx) => ctx,
        Err(err) => {
            error!("error getting canvas context {:?}", err);
            return None;
        }
    };

    let width = canvas.width() as f64;
    let height = canvas.height() as f64;

    Some((ctx, width, height))
}
