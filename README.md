# Otway

### GUI toolkit library which aims to continue the simplicity of Reclutch

## Views

The `view` module aims to be a high-level idiomatic widget type;

```rust
type CounterState = i32;

fn counter<T: 'static>(parent: ui::CommonRef, aux: &mut ui::Aux<T>) -> view::View<T, CounterState> {
    let mut view = view::View::new(parent, /* CounterState: */ 0);

    let layout = view.child(kit::VStack::new, aux);

    let (count_up, count_label, count_down) =
        (
            // these are still children of `view`, not `layout`.
            view.lay(kit::Button::new, aux, &layout, None),
            view.lay(kit::Label::new, aux, &layout, None),
            view.lay(kit::Button::new, aux, &layout, None)
        );

    view.handle(
        "",
        QueueHandler::new(view.get(count_up).evq()).on("press", |view, _, _| {
            view.set_state(|state| { *state += 1; });
        })
    );

    view.handle(
        "",
        QueueHandler::new(view.get(count_down).evq()).on("press", |view, _, _| {
            view.set_state(|state| { *state -= 1; });
        })
    );

    view.state_changed(|view| {
        view.get(count_label).set_text(format!("Count: {}", view.state().count));
    });

    // invoke state_changed to initialize
    view.set_state(|| {});

    view
}
```

## Otway vs. Thunderclap

To be frank, Thunderclap quickly descended into a messy bundle of Rust workarounds. The whole idea of Reclutch is that it flows well with idiomatic Rust, and Thunderclap ended up with excessive reliance on macros and trait misuse to provide an acceptable experience, i.e. not idiomatic.

Otway is really a second attempt. A lot of "landmines" can be avoided thanks to first experimenting with Thunderclap.
In general, Otway aims to stay as far away from macros, instead using regular Rust to achieve the same ease of use. Further, Otway aims to eliminate the use of "widget traits" (in Thunderclap this meant things like `Layable`, `StoresParentPosition`, etc).

## License

Otway is licensed under either

- [Apache 2.0](https://www.apache.org/licenses/LICENSE-2.0)
- [MIT](https://opensource.org/licenses/MIT)

at your choosing.
