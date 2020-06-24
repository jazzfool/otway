# Otway

### GUI toolkit library which aims to continue the simplicity of Reclutch

## Design Goals

- **Open-ended input/eventing:** There are no restrictions on the types of user input that can be received. You can plug in your own windowing code and emit custom input events (e.g. stylus input), then write custom event handlers which can be attached to widgets. Since it's based on Reclutch, you can also write your own renderer by implementing `GraphicsDisplay`.
- **First-class support for custom widgets:** This is driven by the principle that all the surrounding systems used by the toolkit should also be useful to custom widgets. For example, things like the theming system and input handlers can support custom widgets.
- **Accessibility and replaceability of "under-the-hood" components:** Nothing is "baked in". Not even input handling. If you have a custom solution to widget mouse/keyboard events, handling widget focus, etc. then all of that is easily replaceable. These systems run alongside the widget event handling and are inspired very much by the concept of "systems" in ECS.
- **High-level abstractions:** As nice as complete control over the UI is, it can be tedious. In that regard, it is easy to write abstractions to hide the details and focus on the content. One such abstraction is already provided; `view`.

<img src=".media/todos.png" width="40%"/><br>
<img src=".media/counter.png" width="40%"/>

## License

Otway is licensed under either

- [Apache 2.0](https://www.apache.org/licenses/LICENSE-2.0)
- [MIT](https://opensource.org/licenses/MIT)

at your choosing.
