use adw::prelude::*;

// Set a widget to valid or error state
pub fn widget_set_valid(widget: &gtk::Widget, valid: bool) {
    let state = if valid {
        widget.remove_css_class("error");
        widget.add_css_class("success");
        gtk::AccessibleInvalidState::False
    } else {
        widget.add_css_class("error");
        widget.remove_css_class("success");
        gtk::AccessibleInvalidState::True
    };
    widget.update_state(&[gtk::accessible::State::Invalid(state)]);
}
