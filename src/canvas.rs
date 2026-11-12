use gtk::{glib, prelude::*, subclass::prelude::*};
use std::cell::RefCell;
type Draw = Box<dyn Fn(&Canvas, &gtk::Snapshot, i32, i32)>;
mod imp {
    use super::*;
    #[derive(Default)]
    pub struct Canvas {
        pub draw: RefCell<Option<Draw>>,
    }
    #[glib::object_subclass]
    impl ObjectSubclass for Canvas {
        const NAME: &'static str = "UmbrellaCanvas";
        type Type = super::Canvas;
        type ParentType = gtk::Widget;
    }
    impl ObjectImpl for Canvas {}
    impl WidgetImpl for Canvas {
        fn snapshot(&self, snapshot: &gtk::Snapshot) {
            let obj = self.obj();
            if let Some(draw) = self.draw.borrow().as_ref() {
                draw(&obj, snapshot, obj.width(), obj.height());
            }
        }
    }
}
glib::wrapper! { pub struct Canvas(ObjectSubclass<imp::Canvas>) @extends gtk::Widget, @implements gtk::Buildable, gtk::ConstraintTarget; }
impl Canvas {
    pub fn new() -> Self {
        glib::Object::new()
    }
    pub fn set_draw_func(&self, draw: impl Fn(&Self, &gtk::Snapshot, i32, i32) + 'static) {
        *self.imp().draw.borrow_mut() = Some(Box::new(draw));
    }
}
