use otway::{
    app, kit,
    prelude::*,
    reclutch::display as gfx,
    theme,
    ui::{self, view::View},
};

type AuxData = ();
type Aux = app::AppAux<AuxData>;
type AppAux = app::AppData<AuxData>;

struct IncrementEvent;
struct DecrementEvent;

fn counter(parent: ui::CommonRef, aux: &mut Aux) -> View<AppAux, i32> {
    let mut view = View::new(parent, aux, 0);

    let incr = view
        .button_ext("Increment", aux)
        .press(|view, aux, _| {
            view.set_state(|x| *x += 1);
            aux.emit(view, IncrementEvent);
        })
        .inner();

    let decr = view
        .button_ext("Decrement", aux)
        .press(|view, aux, _| {
            view.set_state(|x| *x -= 1);
            aux.emit(view, DecrementEvent);
        })
        .inner();

    let label = view.label_ext("bro", aux).size(42.0).into_ref();

    let mut vstack = ui::layout::Node::new(ui::layout::vstack::VStack::new(), None);
    let mut hstack = ui::layout::Node::new(ui::layout::hstack::HStack::new(), None);
    hstack.push(view.get(incr).unwrap(), None);
    hstack.push(view.get(decr).unwrap(), None);
    vstack.push(view.get(label).unwrap(), None);
    vstack.push(hstack, None);

    view.common().with(|x| x.set_layout(vstack));

    ui::layout::update_layout(&view);

    view.state_changed(move |view| {
        let count = *view.state();
        view.get_mut(label)
            .unwrap()
            .set_text(format!("Count = {}", count));
    });

    view.set_state(|_| {});

    view
}

fn main() -> Result<(), app::AppError> {
    app::run(
        counter,
        (),
        |display| Box::new(theme::flat::FlatTheme::new(display, None, None).unwrap()),
        Default::default(),
    )
}
