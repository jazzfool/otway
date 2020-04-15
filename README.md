# Otway

### GUI toolkit library which aims to continue the simplicity of Reclutch

## Counter Example

```rust
type CounterState = i32;

#[derive(Clone, Event)]
enum CounterEvent {
    #[event_key(increment)]
    Increment,
    #[event_key(decrement)]
    Decrement,
}

fn counter<T: 'static>(parent: ui::CommonRef, aux: &mut ui::Aux<T>) -> view::View<T, CounterState, CounterEvent> {
    let mut view = view::View::new(parent, /* CounterState: */ 0);

    let layout = view.child(kit::VStack::new, aux);

    let (count_up, count_label, count_down) =
        (
            // These are still owned by `view`, not `layout`.
            view.lay(kit::Button::new, aux, &layout, None),
            view.lay(kit::Label::new, aux, &layout, None),
            view.lay(kit::Button::new, aux, &layout, None)
        );
    
    // Handle the "press" event for the "count_up" button.
    view.handle(count_up, "press", |view, _, _| {
        view.set_state(|state| *state += 1);
        view.node_ref().emit_owned(CounterEvent::Increment);
    });

    // Handle the "press" event for the "count_down" button.
    view.handle(count_down, "press", |view, _, _| {
        view.set_state(|state| *state -= 1);
        view.node_ref().emit_owned(CounterEvent::Decrement);
    });

    // Callback whenver `set_state` is invoked.
    view.state_changed(|view| {
        view.get(count_label).set_text(format!("Count: {}", view.state().count));
    });

    // Invoke state_changed to initialize label.
    view.set_state(|_| {});

    view
}
```

### Event Queue Synchronization

In a real world application, out-of-order queue updating will rarely occur given that queues are updated as soon as an event is received by the OS, thus giving no chance for multiple events to pile up.
However, in the rare case that this does occur, the queue system is reinforced by `sinq`, which ensures that everything is updated based on the original order that events were emitted in.

### Parallelism

At some point, Otway may internally move from `sinq` to [`revenq`](https://github.com/YZITE/revenq), or parallel queue updating may be implemented in `sinq`.
Either way, hopefully there will be some multi-threading introduced to the update mechanisms.

There are no plans for moving rendering code to a separate thread, given that `winit` schedules repaints excellently already.

### `View` or `Widget`?

If you need custom rendering or custom input handling, use `Widget`.

If you want to compose widgets to make a larger UI, use `View`.

### `Widget`s have no Middleman

`View`s, by their very nature, simplify creating a UI by acting as a proxy interface, and thus require handles to reference children.

`Widget`s, on the other hand, handle everything themselves. They can access their children directly.

### Full and Extensible Theming

Widgets are 100% rendered by themes. Further than that, themes are stringly-typed and composable, meaning you can extend an existing theme to also cover your own custom widgets.

### Open-ended Windowing and Rendering

The only standard interface relating to OS interactions is the window event type, which defines events for things such as clicking, typing, cursor movements, etc.

Everything else is up to the implementor; any windowing API can be used and any graphics backend can be used as long as it implements `reclutch::display::GraphicsDisplay`.

## License

Otway is licensed under either

- [Apache 2.0](https://www.apache.org/licenses/LICENSE-2.0)
- [MIT](https://opensource.org/licenses/MIT)

at your choosing.
