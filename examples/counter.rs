use otway::{
    kit,
    ui::{self, view::View},
};

fn counter(parent: ui::CommonRef, aux: &mut ui::Aux<()>) -> View<(), i32> {
    let mut view = View::new(parent, 0);

    // let layout = view.child(kit::VStack::new, aux);

    // let btn = view.lay(kit::Button::new, aux, &layout, 0);

    view
}

fn main() {}
