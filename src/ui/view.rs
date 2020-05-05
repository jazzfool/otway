use {super::*, std::collections::HashMap};

/// Holds a strongly-typed ID of a child within a view.
#[derive(Derivative)]
#[derivative(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
// regular `derive` will place unnecessary bounds on `W`.
// ugly, but surprisingly the least ugly solution to this problem.
#[derivative(Debug(bound = ""))]
#[derivative(Clone(bound = ""))]
#[derivative(Copy(bound = ""))]
#[derivative(PartialEq(bound = ""))]
#[derivative(Eq(bound = ""))]
#[derivative(PartialOrd(bound = ""))]
#[derivative(Ord(bound = ""))]
#[derivative(Hash(bound = ""))]
pub struct ChildRef<W>(u64, u64, std::marker::PhantomData<W>);

impl<W> Id for ChildRef<W> {
    #[inline]
    fn id(&self) -> u64 {
        self.1
    }
}

type StateChangedCallback<T> = Box<dyn Fn(&mut T)>;

/// Simplified widget interface to create a stateful and eventful composition of child widgets.
///
/// # Generics
/// - `T`; The inner auxiliary type (i.e. the `T` in `Aux<T>`).
/// - `S`; The state of this view. In essence, this is any data that represents an instantaneous phase of your UI.
/// However, this isn't a strict requirement. If you simply want to store additional data associated with a view,
/// this can also be used for that.
/// - `E`; Event type emitted by the event node. By default, this is `NoEvent` (i.e. no events will be emitted).
/// This can be overriden to be any static type that inherits `reclutch::verbgraph::Event`.
pub struct View<T: 'static, S: 'static> {
    state: S,
    next_child: u64,
    children: HashMap<u64, Box<AuxWidgetChildren<T>>>,
    state_changed: Option<Vec<StateChangedCallback<Self>>>,
    common: CommonRef,
    listener: Listener<Self, Aux<T>>,
}

impl<T: 'static, S: 'static> View<T, S> {
    /// Creates a new view with an initial state.
    pub fn new(parent: CommonRef, aux: &mut Aux<T>, state: S) -> Self {
        View {
            state,
            next_child: 0,
            children: HashMap::new(),
            state_changed: Some(Vec::new()),
            common: CommonRef::new(parent),
            listener: aux.listen(),
        }
    }

    /// Creates a child and returns a reference to it.
    pub fn child<W: WidgetChildren<T> + 'static>(
        &mut self,
        new: impl FnOnce(CommonRef, &mut Aux<T>) -> W,
        aux: &mut Aux<T>,
    ) -> ChildRef<W> {
        self.children
            .insert(self.next_child, Box::new(new(self.common.clone(), aux)));
        self.next_child += 1;
        ChildRef(
            self.next_child - 1,
            self.children
                .get(&(self.next_child - 1))
                .as_ref()
                .unwrap()
                .as_ref()
                .id(),
            Default::default(),
        )
    }

    /// Creates a child and makes it a layout child of another widget.
    pub fn lay<W: WidgetChildren<T> + 'static, L: Layout<T> + 'static>(
        &mut self,
        new: impl FnOnce(CommonRef, &mut Aux<T>) -> W,
        aux: &mut Aux<T>,
        layout: ChildRef<L>,
        config: L::Config,
    ) -> ChildRef<W> {
        self.children
            .insert(self.next_child, Box::new(new(self.common.clone(), aux)));
        let child = ChildRef(
            self.next_child,
            self.children.get(&self.next_child).as_ref().unwrap().id(),
            Default::default(),
        );
        self.next_child += 1;
        let common = self.get::<W>(child).unwrap().common().clone();
        if let Some(layout) = self.get_mut(layout) {
            layout.push(common, config);
        }
        child
    }

    /// Returns a immutable reference to a child widget.
    #[inline]
    pub fn get<W: WidgetChildren<T> + 'static>(&self, child: ChildRef<W>) -> Option<&W> {
        self.children.get(&child.0)?.as_any().downcast_ref::<W>()
    }

    /// Returns a mutable reference to a child widget.
    #[inline]
    pub fn get_mut<W: WidgetChildren<T> + 'static>(
        &mut self,
        child: ChildRef<W>,
    ) -> Option<&mut W> {
        self.children
            .get_mut(&child.0)?
            .as_any_mut()
            .downcast_mut::<W>()
    }

    /// Removes a child widget.
    pub fn remove<W: WidgetChildren<T> + 'static>(&mut self, child: ChildRef<W>) -> Option<W> {
        self.children
            .remove(&child.0)
            .map(|x| *x.as_any_box().downcast::<W>().unwrap())
    }

    /// Returns `true` if this view has a given child widget, otherwise `false`.
    pub fn has<W: WidgetChildren<T> + 'static>(&self, child: ChildRef<W>) -> bool {
        self.children.contains_key(&child.0)
    }

    /// Handles an event from a child node.
    pub fn handle<W: WidgetChildren<T> + 'static, Eo: 'static>(
        &mut self,
        child: ChildRef<W>,
        handler: impl FnMut(&mut Self, &mut Aux<T>, &Eo) + 'static,
    ) {
        let id = self.get(child).map(|x| x.common().with(|x| x.id()));
        if let Some(id) = id {
            self.listener.on(id, handler);
        }
    }

    /// Returns an immutable reference to the inner listener.
    #[inline]
    pub fn listener(&self) -> &Listener<Self, Aux<T>> {
        &self.listener
    }

    /// Returns a mutable reference to the inner listener.
    #[inline]
    pub fn listener_mut(&mut self) -> &mut Listener<Self, Aux<T>> {
        &mut self.listener
    }

    /// Return an immutable reference to the state.
    ///
    /// To mutate the state, use `set_state`.
    #[inline]
    pub fn state(&self) -> &S {
        &self.state
    }

    /// Mutates the state through a closure.
    /// Any value returned from the closure is returned by this function.
    /// This will trigger `state_changed` callbacks.
    pub fn set_state<R>(&mut self, set: impl FnOnce(&mut S) -> R) -> R {
        let r = set(&mut self.state);
        let mut handlers = self.state_changed.take().unwrap();
        for handler in &mut handlers {
            (*handler)(self);
        }
        self.state_changed = Some(handlers);
        r
    }

    /// Adds a callback for state changes.
    ///
    /// This is unrelated to the event queue system.
    #[inline]
    pub fn state_changed(&mut self, handler: impl Fn(&mut Self) + 'static) {
        self.state_changed.as_mut().unwrap().push(Box::new(handler));
    }
}

impl<T: 'static, S: 'static> WidgetChildren<T> for View<T, S> {
    fn children(&self) -> Vec<&dyn WidgetChildren<T>> {
        self.children.values().map(|x| &**x).collect()
    }

    fn children_mut(&mut self) -> Vec<&mut dyn WidgetChildren<T>> {
        self.children.values_mut().map(|x| &mut **x).collect()
    }
}

impl<T: 'static, S: 'static> Element for View<T, S> {
    type Aux = T;

    #[inline]
    fn common(&self) -> &CommonRef {
        &self.common
    }

    #[inline]
    fn update(&mut self, aux: &mut Aux<T>) {
        dispatch(self, aux, |x| &mut x.listener);
    }
}
