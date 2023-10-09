use leptos::html::{ElementDescriptor, Section};
use leptos::{create_rw_signal, window, NodeRef, RwSignal, SignalGet, SignalSet};
use leptos_use::use_window_scroll;
use wasm_bindgen::JsValue;

#[derive(Clone, Copy, PartialEq, Debug)]
pub enum ScrollSection {
    HomeTop,
    Home,
    About,
    GalleryTop,
    Gallery,
}

#[derive(Copy, Clone, Debug)]
pub struct GlobalState {
    pub section: RwSignal<ScrollSection>,
}

impl GlobalState {
    pub fn new() -> Self {
        Self {
            section: create_rw_signal(ScrollSection::HomeTop),
        }
    }
}

pub trait OffsetTop {
    fn y(&self) -> i32;
}

impl OffsetTop for leptos::HtmlElement<Section> {
    fn y(&self) -> i32 {
        self.offset_top()
    }
}

pub struct ScrollDetect<'a, T: ElementDescriptor + 'static> {
    pub node_ref: NodeRef<T>,
    pub offset: i32,
    pub path: &'a str,
    pub id: ScrollSection,
}

impl<'a, T: ElementDescriptor> ScrollDetect<'a, T> {
    pub fn new(id: ScrollSection, node_ref: NodeRef<T>, offset: i32, path: &'a str) -> Self {
        Self {
            node_ref,
            offset,
            path,
            id,
        }
    }
}

impl<'a, T: ElementDescriptor + Clone> ScrollDetect<'a, T> {
    pub fn calc_section(
        section: RwSignal<ScrollSection>,
        default: ScrollSection,
        scroll_items: &[ScrollDetect<'a, T>],
    ) -> ()
    where
        leptos::HtmlElement<T>: OffsetTop,
    {
        let current_section: ScrollSection = section.get();

        for scroll_item in scroll_items {
            let (x, y) = use_window_scroll();
            let element_y = get_element_y(scroll_item.node_ref) - scroll_item.offset;
            //log!("{:?} : {} <= {}", scroll_item.id, element_y, y());
            if element_y as f64 <= y() {
                //log!("{:?} == {:?}", scroll_item.id, current_section);
                if scroll_item.id != current_section {
                    //log!("SET FROM {:?} TO {:?}", current_section, scroll_item.id);
                    section.set(scroll_item.id);
                    silent_navigate(scroll_item.path, "", scroll_item.path);
                }
                return ();
            }
        }

        if current_section != default {
            //log!("BOOM {:?} == {:?}", ScrollSection::None, current_section);
            section.set(default);
        }
    }
}

pub fn get_element_y<T: ElementDescriptor + Clone>(element: NodeRef<T>) -> i32
where
    leptos::HtmlElement<T>: OffsetTop,
{
    let test = element.get();
    let a = test.unwrap().y();

    let mut section_y: i32 = 0;
    if let Some(section) = element.get() {
        let a = section.y();
        section_y = a;
    }
    section_y
}

pub fn silent_navigate(state: &str, unused: &str, url: &str) {
    let a = window().history();
    if let Ok(a) = a {
        a.push_state_with_url(&JsValue::from(state), unused, Some(url))
            .unwrap();
    }
}

fn get_window_path() -> String {
    let location = window().location();
    let path = location.pathname();
    let hash = location.hash();
    if let (Ok(path), Ok(hash)) = (path, hash) {
        format!("{}{}", path, hash)
    } else {
        String::from("/")
    }
}
