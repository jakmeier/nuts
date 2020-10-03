use wasm_bindgen::prelude::{wasm_bindgen, Closure};
use wasm_bindgen::JsCast;
mod utils;

use web_sys::Element;

// When the `wee_alloc` feature is enabled, use `wee_alloc` as the global
// allocator.
#[cfg(feature = "wee_alloc")]
#[global_allocator]
static ALLOC: wee_alloc::WeeAlloc = wee_alloc::WeeAlloc::INIT;

#[wasm_bindgen]
pub fn init() {
    utils::set_panic_hook();

    // window and document have to be fetched from JS world
    let window = web_sys::window().unwrap();
    let document = window.document().unwrap();

    let main = document.get_elements_by_tag_name("main").item(0).unwrap();

    let dynamic_text: Element = document.create_element("p").unwrap();
    main.prepend_with_node_1(&dynamic_text).unwrap();
    unsafe {
        DYNAMIC_TEXT = Some(dynamic_text);
        update_text();
    }

    set_timer();
}

static mut DYNAMIC_TEXT: Option<Element> = None;
static mut APPLES: i32 = 1;
static mut TREES: i32 = 0;

pub unsafe fn update_text() {
    if let Some(dynamic_text) = DYNAMIC_TEXT.as_mut() {
        dynamic_text.set_inner_html(&format!("You have {} apples and {} trees.", APPLES, TREES));
    }
}

#[wasm_bindgen]
pub fn buy() {
    unsafe {
        if APPLES > 0 {
            TREES += 1;
            APPLES -= 1;
            update_text();
        }
    }
}

fn collect_apples() {
    unsafe {
        APPLES += TREES;
        update_text();
    }
}

fn set_timer() {
    let window = web_sys::window().unwrap();
    let boxed_function = Box::new(collect_apples);
    let closure = Closure::wrap(boxed_function as Box<dyn Fn()>);

    // setInterval() has many different names in Rust because there is no overloading
    window
        .set_interval_with_callback_and_timeout_and_arguments_0(
            closure.as_ref().unchecked_ref(),
            5000,
        )
        .unwrap();
    // Leak memory on purpose
    closure.forget();
}

/*
 * Safe implementation of the same as above
 */

use std::cell::RefCell;
use std::sync::atomic::{AtomicI32, Ordering};
use std::thread_local;
thread_local!(
    static SAFE_DYNAMIC_TEXT: RefCell<Option<Element>> = RefCell::new(None);
);

static SAFE_APPLES: AtomicI32 = AtomicI32::new(1);
static SAFE_TREES: AtomicI32 = AtomicI32::new(0);

#[wasm_bindgen]
pub fn safe_init() {
    utils::set_panic_hook();

    // window and document have to be fetched from JS world
    let window = web_sys::window().unwrap();
    let document = window.document().unwrap();

    let main = document.get_elements_by_tag_name("main").item(0).unwrap();

    let dynamic_text: Element = document.create_element("p").unwrap();
    main.prepend_with_node_1(&dynamic_text).unwrap();

    SAFE_DYNAMIC_TEXT
        .with(|maybe_dynamic_text| *maybe_dynamic_text.borrow_mut() = Some(dynamic_text));
    safe_update_text();

    safe_set_timer();
}

pub fn safe_update_text() {
    SAFE_DYNAMIC_TEXT.with(|maybe_dynamic_text| {
        if let Some(dynamic_text) = maybe_dynamic_text.borrow().as_ref() {
            dynamic_text.set_inner_html(&format!(
                "You have {} apples and {} trees.",
                SAFE_APPLES.load(Ordering::Relaxed),
                SAFE_TREES.load(Ordering::Relaxed)
            ))
        }
    })
}

#[wasm_bindgen]
pub fn safe_buy() {
    if SAFE_APPLES.load(Ordering::Relaxed) > 0 {
        SAFE_TREES.fetch_add(1, Ordering::Relaxed);
        SAFE_APPLES.fetch_add(-1, Ordering::Relaxed);
        safe_update_text();
    }
}

fn safe_collect_apples() {
    SAFE_APPLES.fetch_add(SAFE_TREES.load(Ordering::Relaxed), Ordering::Relaxed);
    safe_update_text();
}

fn safe_set_timer() {
    let window = web_sys::window().unwrap();
    let boxed_function = Box::new(safe_collect_apples);
    let closure = Closure::wrap(boxed_function as Box<dyn Fn()>);

    // setInterval() has many different names in Rust because there is no overloading
    window
        .set_interval_with_callback_and_timeout_and_arguments_0(
            closure.as_ref().unchecked_ref(),
            5000,
        )
        .unwrap();
    // Leak memory on purpose
    closure.forget();
}
