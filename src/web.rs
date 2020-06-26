// TODO: Remove the "web" part 
 // should not be part of library?
 // maybe instead just have lifecylce reduced to enter+leave, then the rest is through publish-subscribe of ANY
 // next.outer  layer can then build draw/update abstraction around it
 // the idea of this crate is just to have this image of globally available state
 // this approach keeps maximum flexibility, including the possiblity to have frames controlled by another crate
 // and still I can use the activities because they are just migically always available
 // and the beneft is that handlers can be registered anywhere. The handler for animation-frames is then just a special case.
// use stdweb;

// /// Calls webnut::draw() in every animation frame as managed by the browser. (Using requestAnimationFrame)
// pub fn auto_draw() {
//     stdweb::web::window().request_animation_frame(|_| crate::draw());
// }

// /// Calls webnut::update() in intervals managed by the browser. (Using setInterval)
// /// The defined interval will be the maximum number of calls but may be less if the computation takes too long
// pub fn auto_update(delay_ms: u32) {
//     let callback = crate::update;

//     js!( @(no_return)
//         setInterval( @{callback}, @{delay_ms});
//     );
// }
