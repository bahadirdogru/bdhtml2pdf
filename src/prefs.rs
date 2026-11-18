//! Stylo feature switches.
//!
//! Stylo (servo build) gates a few modern selector / at-rule features behind
//! runtime prefs that Blitz does not enable. Modern CSS is this project's main
//! goal, so turn them on before the first document is created.

use std::sync::Once;

pub fn init() {
    static INIT: Once = Once::new();
    INIT.call_once(|| {
        style_config::set_pref!("layout.css.has-selector.enabled", true);
        style_config::set_pref!("layout.css.nth-child-of.enabled", true);
        style_config::set_pref!("layout.css.container-queries.enabled", true);
    });
}
