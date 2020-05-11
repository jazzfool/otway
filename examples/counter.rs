use otway::{
    app,
    prelude::*,
    theme,
    ui::{self, layout, view::View},
};

type AuxData = ();
type Aux = app::AppAux<AuxData>;
type AppAux = app::AppData<AuxData>;

struct IncrementEvent;
struct DecrementEvent;

fn counter(parent: ui::CommonRef, aux: &mut Aux) -> View<AppAux, i32> {
    let mut view = View::new(parent, aux, 0);

    let mut vstack = layout::VStack::new().into_node(None);
    let mut hstack = layout::HStack::new().into_node(None);

    let label = view
        .label(aux)
        .layout(&mut vstack, Some((0.0, 5.0).into()))
        .into_inner();

    view.button(aux)
        .text("Increment")
        .layout(&mut hstack, Some((0.0, 5.0).into()))
        .press(|view, aux, _| {
            view.set_state(|x| *x += 1);
            aux.emit(view, IncrementEvent);
        });

    view.button(aux)
        .text("Decrement")
        .layout(&mut hstack, Some((0.0, 5.0).into()))
        .press(|view, aux, _| {
            view.set_state(|x| *x -= 1);
            aux.emit(view, DecrementEvent);
        });

    vstack.push(hstack, None);

    view
        .label(aux)
        .text("This should be a counter with 2 buttons and label, centered in the middle of the window. If you resize the window, it should stay centered. If any of this is not happening for you, please open an issue on GitHub!")
        .max_width(150.0)
        .layout(&mut vstack, Some((10.0, 0.0).into()))
        .into_inner();

    view.set_layout(vstack);

    view.state_changed(move |view| {
        let count = *view.state();
        view.get_mut(label)
            .unwrap()
            .set_text(format!("Count: {}", count));
        layout::update_layout(view);
    });

    view.set_state(|_| {});

    let mut rb = layout::RelativeBox::new(layout::RelativeBoxConfig::center()).into_node(None);
    rb.push(&view, ());
    aux.central_widget.with(|x| x.set_layout(rb));
    view.set_layout_mode(ui::LayoutMode::Shrink);

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
