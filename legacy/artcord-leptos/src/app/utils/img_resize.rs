use chrono::Utc;
use leptos::*;
use leptos::{window, RwSignal, SignalGetUntracked};
use std::fmt::Debug;
use wasm_bindgen::JsValue;
use web_sys::Location;
use tracing::{trace, debug};

pub const NEW_IMG_HEIGHT: u32 = 250;

pub trait GalleryImg {
    fn get_size(&self) -> (u32, u32);
    fn get_pos(&self) -> (f32, f32);
    fn set_pos(&mut self, left: f32, top: f32, new_width: f32, new_height: f32);
}

pub fn resize_img<T: GalleryImg + Debug>(
    top: &mut f32,
    max_width: u32,
    new_row_start: usize,
    new_row_end: usize,
    imgs: &mut [T],
) {
    let mut total_ratio: f32 = 0f32;

    for i in new_row_start..(new_row_end + 1) {
        let (width, height) = imgs[i].get_size();
        total_ratio += width as f32 / height as f32;
    }
    let optimal_height: f32 = max_width as f32 / total_ratio;
    let mut left: f32 = 0.0;

    for i in new_row_start..(new_row_end + 1) {
        let (width, height) = imgs[i].get_size();
        let new_width = optimal_height * (width as f32 / height as f32);
        let new_height = optimal_height;
        imgs[i].set_pos(left, *top, new_width, new_height);
        left += new_width;
    }
    *top += optimal_height;
}

pub fn resize_img2<T: GalleryImg + Debug>(
    top: &mut f32,
    max_width: u32,
    new_row_start: usize,
    new_row_end: usize,
    imgs: &mut [T],
) {
    let mut optimal_count =
        (max_width as i32 / NEW_IMG_HEIGHT as i32) - (new_row_end - new_row_start) as i32;
    if optimal_count < 0 {
        optimal_count = 0;
    }
    let mut total_ratio: f32 = optimal_count as f32;
    if max_width < NEW_IMG_HEIGHT * 3 {
        total_ratio = 0.0;
    }

    for i in new_row_start..(new_row_end + 1) {
        let (width, height) = imgs[i].get_size();
        total_ratio += width as f32 / height as f32;
    }
    let optimal_height: f32 = max_width as f32 / total_ratio;
    let mut left: f32 = 0.0;

    for i in new_row_start..(new_row_end + 1) {
        let (width, height) = imgs[i].get_size();
        let new_width = optimal_height * (width as f32 / height as f32);
        let new_height = optimal_height;
        imgs[i].set_pos(left, *top, new_width, new_height);
        left += new_width;
    }

    *top += optimal_height;
}

pub fn resize_imgs<T: GalleryImg + Debug>(new_height: u32, max_width: u32, imgs: &mut [T]) -> () {
    debug!("utils: resizing started: count: {}", imgs.len());
    let loop_start = 0;
    let loop_end = imgs.len();
    let mut new_row_start: usize = 0;
    let mut new_row_end: usize = if loop_end > 0 { loop_end - 1 } else { 0 };
    let mut current_row_filled_width: u32 = 0;
    let mut top: f32 = 0.0;

    for index in loop_start..loop_end {
        let org_img = &mut imgs[index];
        let (width, height) = org_img.get_size();
        let ratio: f32 = width as f32 / height as f32;
        let height_diff: u32 = if height < new_height {
            0
        } else {
            height - new_height
        };
        let new_width: u32 = width - (height_diff as f32 * ratio) as u32;
        if (current_row_filled_width + new_width) <= max_width {
            current_row_filled_width += new_width;
            new_row_end = index;
            if index == loop_end - 1 {
                resize_img2(&mut top, max_width, new_row_start, new_row_end, imgs);
            }
        } else {
            if index != 0 {
                resize_img(&mut top, max_width, new_row_start, new_row_end, imgs);
            }
            new_row_start = index;
            new_row_end = index;
            current_row_filled_width = new_width;
            if index == loop_end - 1 {
                resize_img2(&mut top, max_width, new_row_start, new_row_end, imgs);
            }
        }
    }

    debug!("utils: resizing ended: count: {}", imgs.len());
}


pub fn calc_fit_count(width: u32, height: u32) -> u32 {
    (width * height) / (NEW_IMG_HEIGHT * NEW_IMG_HEIGHT)
}