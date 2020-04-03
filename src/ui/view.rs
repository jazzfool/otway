use {
    super::*,
    reclutch::{display as gfx, event, verbgraph as vg},
    std::collections::HashMap,
};

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct ChildRef<W>(u64, std::marker::PhantomData<W>);

type StateChangedCallback<T> = Box<dyn FnMut(&mut T)>;

pub struct View<T: 'static, S: 'static> {
    state: S,
    next_child: u64,
    children: HashMap<u64, Box<AuxWidgetChildren<T>>>,
    state_changed: Option<Vec<StateChangedCallback<Self>>>,
    common: CommonRef,
    graph: vg::OptionVerbGraph<Self, Aux<T>>,
}

impl<T: 'static, S: 'static> View<T, S> {
    pub fn new(parent: CommonRef, state: S) -> Self {
        View {
            state,
            next_child: 0,
            children: HashMap::new(),
            state_changed: Some(Vec::new()),
            common: CommonRef::new(parent),
            graph: Some(Default::default()),
        }
    }

    pub fn child<
        W: WidgetChildren<
                UpdateAux = Aux<T>,
                GraphicalAux = Aux<T>,
                DisplayObject = gfx::DisplayCommand,
            > + 'static,
    >(
        &mut self,
        new: impl FnOnce(CommonRef, &mut Aux<T>) -> W,
        aux: &mut Aux<T>,
    ) -> ChildRef<W> {
        self.children
            .insert(self.next_child, Box::new(new(self.common.clone(), aux)));
        self.next_child += 1;
        ChildRef(self.next_child - 1, Default::default())
    }

    pub fn lay<
        W: WidgetChildren<
                UpdateAux = Aux<T>,
                GraphicalAux = Aux<T>,
                DisplayObject = gfx::DisplayCommand,
            > + 'static,
        L: Layout<UpdateAux = Aux<T>, GraphicalAux = Aux<T>, DisplayObject = gfx::DisplayCommand>
            + 'static,
    >(
        &mut self,
        new: impl FnOnce(CommonRef, &mut Aux<T>) -> W,
        aux: &mut Aux<T>,
        layout: &ChildRef<L>,
        config: L::Config,
    ) -> ChildRef<W> {
        self.children
            .insert(self.next_child, Box::new(new(self.common.clone(), aux)));
        let child = ChildRef(self.next_child, Default::default());
        self.next_child += 1;
        let common = self.get::<W>(&child).unwrap().common().clone();
        if let Some(layout) = self.get_mut(&layout) {
            layout.push(common, config);
        }
        child
    }

    #[inline]
    pub fn get<
        W: WidgetChildren<
                UpdateAux = Aux<T>,
                GraphicalAux = Aux<T>,
                DisplayObject = gfx::DisplayCommand,
            > + 'static,
    >(
        &self,
        child: &ChildRef<W>,
    ) -> Option<&W> {
        self.children.get(&child.0)?.as_any().downcast_ref::<W>()
    }

    #[inline]
    pub fn get_mut<
        W: WidgetChildren<
                UpdateAux = Aux<T>,
                GraphicalAux = Aux<T>,
                DisplayObject = gfx::DisplayCommand,
            > + 'static,
    >(
        &mut self,
        child: &ChildRef<W>,
    ) -> Option<&mut W> {
        self.children
            .get_mut(&child.0)?
            .as_any_mut()
            .downcast_mut::<W>()
    }

    pub fn remove<
        W: WidgetChildren<
                UpdateAux = Aux<T>,
                GraphicalAux = Aux<T>,
                DisplayObject = gfx::DisplayCommand,
            > + 'static,
    >(
        &mut self,
        child: &ChildRef<W>,
    ) -> Option<W> {
        self.children
            .remove(&child.0)
            .map(|x| *x.as_any_box().downcast::<W>().unwrap())
    }

    pub fn handler<E: vg::Event + 'static, L: event::EventListen<Item = E> + 'static>(
        &mut self,
        tag: &'static str,
        handler: vg::QueueHandler<Self, Aux<T>, E, L>,
    ) {
        if let Some(mut graph) = self.graph.take() {
            graph = graph.add(tag, handler);
            self.graph = Some(graph);
        }
    }

    #[inline(always)]
    pub fn state(&self) -> &S {
        &self.state
    }

    pub fn set_state<R>(&mut self, set: impl FnOnce(&mut S) -> R) -> R {
        let r = set(&mut self.state);
        let mut handlers = self.state_changed.take().unwrap();
        for handler in &mut handlers {
            (*handler)(self);
        }
        self.state_changed = Some(handlers);
        r
    }

    #[inline]
    pub fn state_changed(&mut self, handler: impl FnMut(&mut Self) + 'static) {
        self.state_changed.as_mut().unwrap().push(Box::new(handler));
    }
}

impl<T: 'static, S: 'static> WidgetChildren for View<T, S> {
    fn children(
        &self,
    ) -> Vec<
        &dyn WidgetChildren<
            UpdateAux = Self::UpdateAux,
            GraphicalAux = Self::GraphicalAux,
            DisplayObject = Self::DisplayObject,
        >,
    > {
        self.children.values().map(|x| &**x).collect()
    }

    fn children_mut(
        &mut self,
    ) -> Vec<
        &mut dyn WidgetChildren<
            UpdateAux = Self::UpdateAux,
            GraphicalAux = Self::GraphicalAux,
            DisplayObject = Self::DisplayObject,
        >,
    > {
        self.children.values_mut().map(|x| &mut **x).collect()
    }
}

impl<T: 'static, S: 'static> Element for View<T, S> {
    fn common(&self) -> &CommonRef {
        &self.common
    }
}

impl<T: 'static, S: 'static> Widget for View<T, S> {
    type UpdateAux = Aux<T>;
    type GraphicalAux = Aux<T>;
    type DisplayObject = gfx::DisplayCommand;

    fn bounds(&self) -> gfx::Rect {
        self.common.get().rect()
    }

    fn update(&mut self, aux: &mut Aux<T>) {
        vg::update_all(self, aux);

        for child in self.children_mut() {
            child.update(aux);
        }
    }
}

impl<T: 'static, S: 'static> vg::HasVerbGraph for View<T, S> {
    fn verb_graph(&mut self) -> &mut vg::OptionVerbGraph<Self, Aux<T>> {
        &mut self.graph
    }
}
