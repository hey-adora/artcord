use std::{f64::consts::PI, slice::Chunks};

use chrono::{DateTime, Datelike, Month, Weekday};
use leptos::{html::Canvas, *};
use leptos_use::{use_event_listener, use_resize_observer};
use tracing::{debug, error, trace};
use wasm_bindgen::{JsCast, JsValue};
use web_sys::CanvasRenderingContext2d;

pub fn use_graph() -> (NodeRef<Canvas>, RwSignal<Vec<f64>>) {
    let canvas_ref = NodeRef::new();
    let data = RwSignal::new(vec![0.0, 0.0, 10.0, 10.0, 20.0, 10.0]);
    let mouse_pos: RwSignal<Option<(f64, f64)>> = RwSignal::new(None);

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
    });

    let _ = use_event_listener(canvas_ref, ev::mousemove, move |ev| {
        let Some(canvas) = canvas_ref.get_untracked() else {
            error!("error getting canvas context ");
            return;
        };

        mouse_pos.set(Some((ev.offset_x() as f64, ev.offset_y() as f64)));
    });

    let _ = use_event_listener(canvas_ref, ev::mouseleave, move |ev| {
        let Some(canvas) = canvas_ref.get_untracked() else {
            error!("error getting canvas context ");
            return;
        };

        mouse_pos.set(None);
    });

    create_effect(move |_| {
        let Some((ctx, width, height)) = get_ctx(canvas_ref) else {
            return;
        };

        let padding = 100.0;
        let line_height = 15.0;

        data.with(|data| {
            let data_chunks = data.chunks(2);

            let medians = get_medians(data_chunks.clone(), 10);
            let (min_x, max_x, min_y, max_y) = get_min_max(data_chunks.clone());
            let (canvas_x, canvas_y) = (
                (width - padding) / (max_x - min_x),
                (height - padding) / (max_y - min_y),
            );
            let closest_point = mouse_pos.get().and_then(|mouse_pos| {
                get_closest_point(
                    data_chunks.clone(),
                    canvas_x,
                    canvas_y,
                    min_x,
                    min_y,
                    width,
                    height,
                    padding,
                    mouse_pos,
                )
            });

            draw(
                ctx,
                data_chunks,
                medians,
                canvas_x,
                canvas_y,
                max_x,
                max_y,
                min_x,
                min_y,
                width,
                height,
                padding,
                line_height,
                closest_point,
            );
        });
    });

    (canvas_ref, data)
}

fn draw(
    ctx: CanvasRenderingContext2d,
    data: Chunks<f64>,
    medians: Vec<f64>,
    canvas_x: f64,
    canvas_y: f64,
    max_x: f64,
    max_y: f64,
    min_x: f64,
    min_y: f64,
    width: f64,
    height: f64,
    padding: f64,
    line_height: f64,
    closest_point: Option<(usize, f64, f64)>,
) {
    ctx.clear_rect(0.0, 0.0, width, height);

    {
        // let x = padding / 2.0;
        // let y = height - (padding / 4.0);

        ctx.set_font("0.7rem Arial");
        let style = JsValue::from_str("white");
        ctx.set_fill_style(&style);
        // let Some(text) = DateTime::from_timestamp_millis(min_x as i64).map(|date| date.to_string())
        // else {
        //     return;
        // };
        // draw_text_left(&ctx, &text, x, y, line_height);
        // let x = width - (padding / 4.0);
        // let Some(text) = DateTime::from_timestamp_millis(max_x as i64).map(|date| date.to_string())
        // else {
        //     return;
        // };
        // draw_text_right(&ctx, &text, x, y, line_height);

        let x = padding / 4.0;
        let mut y = padding / 2.0;
        let step = (height - padding / 2.0) / medians.len() as f64;

        //trace!("medians: {:#?}", &medians);
        for median in medians.into_iter() {
            let text = (median as i64).to_string();
            draw_text_center(&ctx, &text, x, y, line_height);
            y += step;
        }

        // let y = padding / 2.0;

        // let text = (max_y as i64).to_string();
        // draw_text_right(&ctx, &text, x, y, line_height);
    }

    let mut prev_point: Option<(f64, f64, f64, f64)> = None;
    let mut prev_month: Option<&'static str> = None;
    let count = data.clone().count();
    for (i, chunk) in data.clone().enumerate() {
        let org_x = chunk.get(0).cloned();
        let org_y = chunk.get(1).cloned().unwrap_or(0.0);
        let Some(org_x) = org_x else {
            break;
        };
        let point_x = (org_x - min_x) * canvas_x + (padding / 2.0);
        let point_y = (height - ((org_y - min_y) * canvas_y)) - (padding / 2.0);

        let x_as_date = DateTime::from_timestamp_millis(org_x as i64);
        let Some(x_as_date) = x_as_date else {
            continue;
        };
        let weekday = x_as_date.weekday();

        ctx.begin_path();
        let style = JsValue::from_str("white");
        ctx.set_fill_style(&style);
        let radius = 2.0;
        ctx.arc(point_x, point_y, radius, 0.0, PI * 2.0);
        ctx.fill();

        {
            let point_line_y_start = padding / 3.0;
            let point_line_y_end = height - (padding / 2.5);
            ctx.begin_path();
            ctx.move_to(point_x, point_line_y_start);
            ctx.line_to(point_x, point_line_y_end);
            ctx.set_line_width(2.0);
            let style = match weekday {
                Weekday::Sat | Weekday::Sun => JsValue::from_str("#925CB34D"),
                _ => JsValue::from_str("#ffffff1A"),
            };
            ctx.set_stroke_style(&style);
            ctx.stroke();
        }

        if count / 7 > 5 {
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
                draw_text_left(&ctx, month, point_x, point_text_y, line_height);
                // if i == 0 {
                //     draw_text_left(&ctx, month, point_x, point_text_y, line_height);
                // } else if ((i / 7) % 4) == 0 {
                //     draw_text_left(&ctx, month, point_x, point_text_y, line_height);
                // }
                prev_month = Some(month);
            }
        } else {
            let style = match weekday {
                Weekday::Sat | Weekday::Sun => JsValue::from_str("#925CB3"),
                _ => JsValue::from_str("#ffffff"),
            };
            ctx.set_fill_style(&style);
            let point_text_y = height - (padding / 4.0);
            let text = match weekday {
                Weekday::Mon => "M",
                Weekday::Tue => "T",
                Weekday::Thu => "T",
                Weekday::Wed => "W",
                Weekday::Fri => "F",
                Weekday::Sat => "S",
                Weekday::Sun => "S",
            };
            draw_text_center(&ctx, text, point_x, point_text_y, line_height);
        }

        //trace!("graph: cx: {} cy: {} mx: {} my: {} w: {} h: {} x: {} y:{}", canvas_x, canvas_y, max_x, max_y, width, height, x , y);

        if let Some((prev_x, prev_y, prev_org_x, prev_org_y)) = prev_point {
            ctx.begin_path();
            ctx.move_to(prev_x, prev_y);
            ctx.line_to(point_x, point_y);
            ctx.set_line_width(2.0);
            let style = JsValue::from_str("#925CB3");
            ctx.set_stroke_style(&style);
            ctx.stroke();

            {
                let style = JsValue::from_str("#ffffff");
                ctx.set_fill_style(&style);
                let text = format!("{:.0}", prev_org_y);
                //draw_text_center(&ctx, &text, point_x, point_y - radius * 4.0, line_height);
                draw_text_center(&ctx, &text, prev_x, prev_y - radius * 4.0, line_height);
            }
        }

        if i + 1 == count {
            let style = JsValue::from_str("#ffffff");
            ctx.set_fill_style(&style);
            let text = format!("{:.0}", org_y);
            draw_text_center(&ctx, &text, point_x, point_y - radius * 4.0, line_height);
        }

        prev_point = Some((point_x, point_y, org_x, org_y));

        let Some((closest_point, mouse_x, mouse_y)) = closest_point else {
            continue;
        };
        if closest_point != i {
            continue;
        }

        let text = format!("Connections: {:.0}\n{}", org_y, x_as_date);
        draw_text_center(&ctx, &text, mouse_x, mouse_y - radius * 4.0, line_height);
        // let text = text.split("\n");
        // let texts: Vec<&str> = text.collect();
        // let len = texts.len().checked_sub(1).unwrap_or(1);
        // for (i, text) in texts.iter().enumerate() {
        //     let Ok(text_w) = ctx
        //         .measure_text(text)
        //         .inspect_err(|err| error!("failed to measure text: {:?}", err))
        //     else {
        //         return;
        //     };

        //     let text_x = mouse_x - (text_w.width() / 2.0);
        //     let text_y = (mouse_y - (radius * 2.0)) - line_height * (len - i) as f64;

        //     ctx.fill_text(text, text_x, text_y);
        // }
        {
            ctx.begin_path();
            ctx.move_to(point_x, point_y);
            ctx.line_to(mouse_x, mouse_y);
            ctx.set_line_width(1.0);
            let style = JsValue::from_str("white");
            ctx.set_stroke_style(&style);
            ctx.stroke();
        }
    }
    // for (i, chunk) in data.clone().enumerate() {
    //     let org_x = chunk.get(0).cloned();
    //     let org_y = chunk.get(1).cloned().unwrap_or(0.0);
    //     let Some(org_x) = org_x else {
    //         break;
    //     };
    //     let point_x = (org_x - min_x) * canvas_x + (padding / 2.0);
    //     let point_y = (height - ((org_y - min_y) * canvas_y)) - (padding / 2.0);

    // }
}

fn draw_text_center(ctx: &CanvasRenderingContext2d, text: &str, x: f64, y: f64, line_height: f64) {
    let text = text.split("\n");
    let texts: Vec<&str> = text.collect();
    let len = texts.len().checked_sub(1).unwrap_or(1);
    for (i, text) in texts.iter().enumerate() {
        let Ok(text_w) = ctx
            .measure_text(text)
            .inspect_err(|err| error!("failed to measure text: {:?}", err))
        else {
            return;
        };

        let text_x = x - (text_w.width() / 2.0);
        let text_y = y - line_height * (len - i) as f64;

        ctx.fill_text(text, text_x, text_y);
    }
}

fn draw_text_left(ctx: &CanvasRenderingContext2d, text: &str, x: f64, y: f64, line_height: f64) {
    let text = text.split("\n");
    let texts: Vec<&str> = text.collect();
    let len = texts.len().checked_sub(1).unwrap_or(1);
    for (i, text) in texts.iter().enumerate() {
        let text_x = x;
        let text_y = y - line_height * (len - i) as f64;

        ctx.fill_text(text, text_x, text_y);
    }
}

fn draw_text_right(ctx: &CanvasRenderingContext2d, text: &str, x: f64, y: f64, line_height: f64) {
    let text = text.split("\n");
    let texts: Vec<&str> = text.collect();
    let len = texts.len().checked_sub(1).unwrap_or(1);
    for (i, text) in texts.iter().enumerate() {
        let Ok(text_w) = ctx
            .measure_text(text)
            .inspect_err(|err| error!("failed to measure text: {:?}", err))
        else {
            return;
        };
        let text_x = x - text_w.width();
        let text_y = y - line_height * (len - i) as f64;

        ctx.fill_text(text, text_x, text_y);
    }
}

fn get_medians(data: Chunks<f64>, amount: usize) -> Vec<f64> {
    //let chunk_len = data.len();
    let mut medians: Vec<f64> = Vec::with_capacity(amount);
    let mut prev_point: Option<f64> = None;
   
    //debug!("graph: step is {}",step);

    let mut prev_y_item: Option<f64> = None;
    let mut data_sroted_by_y = data
        .clone()
        .map(|data| data.get(1).cloned().unwrap_or(0.0))
        .collect::<Vec<f64>>();
    data_sroted_by_y.sort_by(|a, b| b.partial_cmp(a).unwrap_or(std::cmp::Ordering::Equal));
    data_sroted_by_y.retain(|y| {
        let same = prev_y_item.map(|prev_y| prev_y == *y).unwrap_or(false);
        prev_y_item = Some(*y);
        !same
    });
    
    let dat_y_len = data_sroted_by_y.len();
    let step = dat_y_len / amount;
    let step = if step == 0 { 1 } else { step };

    //debug!("graph: step {}", step);
    //debug!("graph: median y data {:#?}", &data_sroted_by_y);
    //let prev_item: Option<>
    let mut skip_used = false;
    for (i, y) in data_sroted_by_y.into_iter().enumerate() {
        // let x = chunk.get(0).cloned();
        // let y = chunk.get(1).cloned().unwrap_or(0.0);
        // let Some(x) = x else {
        //     break;
        // };

        if i == 0 {
            medians.push(y);
            prev_point = Some(y);
            continue;
        }

        let same_as_prev = prev_point
            .map(|prev_point| prev_point == y)
            .unwrap_or(false);
        if same_as_prev {
            prev_point = Some(y);
            skip_used = true;
            //debug!("graph: skip set {}", skip_used);
            continue;
        }

        if i + 1 == dat_y_len {
            medians.push(y);
            prev_point = Some(y);
            continue;
        }

        //debug!("graph: {} {}", skip_used, i % step == 0);
        if skip_used || (i % step == 0) {
            medians.push(y);
            skip_used = false;
        }

        prev_point = Some(y);
    }
    // let mut medians = data
    //     .into_iter()
    //     .step_by(len.div_ceil(amount))
    //     .map(|item| item.get(1).cloned())
    //     .collect::<Option<Vec<f64>>>()
    //     .unwrap_or_default();
    // medians.sort_by(|a, b| b.partial_cmp(a).unwrap_or(std::cmp::Ordering::Equal));
    medians
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

fn get_ctx(canvas_ref: NodeRef<Canvas>) -> Option<(CanvasRenderingContext2d, f64, f64)> {
    let Some(canvas) = canvas_ref.get() else {
        error!("error getting canvas context ");
        return None;
    };

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
