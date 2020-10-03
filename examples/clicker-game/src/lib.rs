mod utils;

use wasm_bindgen::prelude::wasm_bindgen;
use web_sys::Element;

// When the `wee_alloc` feature is enabled, use `wee_alloc` as the global
// allocator.
#[cfg(feature = "wee_alloc")]
#[global_allocator]
static ALLOC: wee_alloc::WeeAlloc = wee_alloc::WeeAlloc::INIT;

#[wasm_bindgen]
pub fn init() {
    utils::set_panic_hook();

    let apples = 1;
    let trees = 0;

    let window = web_sys::window().unwrap();
    let document = window.document().unwrap();
    let main = document.get_elements_by_tag_name("main").item(0).unwrap();
    let dynamic_text: Element = document.create_element("p").unwrap();
    main.prepend_with_node_1(&dynamic_text).unwrap();
    let game_state = GameState {
        dynamic_text,
        apples,
        trees,
    };

    let game = nuts::new_activity(game_state);

    game.subscribe(GameState::update_text);
    game.subscribe(GameState::buy);
    game.subscribe(GameState::collect_apples);
    nuts::publish(UpdateTextEvent);

    set_timer();
}

struct GameState {
    dynamic_text: Element,
    apples: i32,
    trees: i32,
}

struct UpdateTextEvent;
struct BuyEvent;
struct CollectEvent;

#[wasm_bindgen]
pub fn buy() {
    nuts::publish(BuyEvent);
}

impl GameState {
    fn update_text(&mut self, _: &UpdateTextEvent) {
        self.dynamic_text.set_inner_html(&format!(
            "You have {} apples and {} trees.",
            self.apples, self.trees
        ));
    }
    fn buy(&mut self, _: &BuyEvent) {
        if self.apples > 0 {
            self.trees += 1;
            self.apples -= 1;
            nuts::publish(UpdateTextEvent);
        }
    }
    fn collect_apples(&mut self, _: &CollectEvent) {
        self.apples += self.trees;
        nuts::publish(UpdateTextEvent);
    }
}

use stdweb::js;
fn set_timer() {
    js! {
        setInterval(
            @{||nuts::publish(CollectEvent)},
            5000
        )
    }
}
