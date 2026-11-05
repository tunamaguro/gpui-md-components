use gpui::{Element, InteractiveElement, IntoElement, Styled};

pub(crate) trait A11yInfoWriter: 'static {
    fn write(&self, node: &mut gpui::accesskit::Node);
}

impl<F> A11yInfoWriter for F
where
    F: Fn(&mut gpui::accesskit::Node) + 'static,
{
    fn write(&self, node: &mut gpui::accesskit::Node) {
        self(node)
    }
}

pub(crate) struct A11yInfoLayer<F, E> {
    writer: F,
    inner: E,
}

impl<F, E> IntoElement for A11yInfoLayer<F, E>
where
    F: A11yInfoWriter,
    E: Element,
{
    type Element = Self;
    fn into_element(self) -> Self::Element {
        self
    }
}

impl<F, E> Element for A11yInfoLayer<F, E>
where
    F: A11yInfoWriter,
    E: Element,
{
    type RequestLayoutState = E::RequestLayoutState;
    type PrepaintState = E::PrepaintState;

    fn id(&self) -> Option<gpui::ElementId> {
        self.inner.id()
    }
    fn source_location(&self) -> Option<&'static std::panic::Location<'static>> {
        self.inner.source_location()
    }
    fn request_layout(
        &mut self,
        id: Option<&gpui::GlobalElementId>,
        inspector_id: Option<&gpui::InspectorElementId>,
        window: &mut gpui::Window,
        cx: &mut gpui::App,
    ) -> (gpui::LayoutId, Self::RequestLayoutState) {
        self.inner.request_layout(id, inspector_id, window, cx)
    }
    fn prepaint(
        &mut self,
        id: Option<&gpui::GlobalElementId>,
        inspector_id: Option<&gpui::InspectorElementId>,
        bounds: gpui::Bounds<gpui::Pixels>,
        request_layout: &mut Self::RequestLayoutState,
        window: &mut gpui::Window,
        cx: &mut gpui::App,
    ) -> Self::PrepaintState {
        self.inner
            .prepaint(id, inspector_id, bounds, request_layout, window, cx)
    }
    fn paint(
        &mut self,
        id: Option<&gpui::GlobalElementId>,
        inspector_id: Option<&gpui::InspectorElementId>,
        bounds: gpui::Bounds<gpui::Pixels>,
        request_layout: &mut Self::RequestLayoutState,
        prepaint: &mut Self::PrepaintState,
        window: &mut gpui::Window,
        cx: &mut gpui::App,
    ) {
        self.inner.paint(
            id,
            inspector_id,
            bounds,
            request_layout,
            prepaint,
            window,
            cx,
        );
    }
    fn a11y_role(&self) -> Option<gpui::accesskit::Role> {
        self.inner.a11y_role()
    }
    fn a11y_synthetic_children(
        &mut self,
        prepaint: &mut Self::PrepaintState,
        builder: &mut gpui::A11ySubtreeBuilder,
    ) {
        self.inner.a11y_synthetic_children(prepaint, builder);
    }
    fn write_a11y_info(&self, node: &mut gpui::accesskit::Node) {
        self.inner.write_a11y_info(node);
        self.writer.write(node);
    }
}

impl<F, E> Styled for A11yInfoLayer<F, E>
where
    F: A11yInfoWriter,
    E: Styled,
{
    fn style(&mut self) -> &mut gpui::StyleRefinement {
        self.inner.style()
    }
}

impl<F, E> gpui::ParentElement for A11yInfoLayer<F, E>
where
    F: A11yInfoWriter,
    E: gpui::ParentElement,
{
    fn extend(&mut self, elements: impl IntoIterator<Item = gpui::AnyElement>) {
        self.inner.extend(elements);
    }
}

impl<F, E> InteractiveElement for A11yInfoLayer<F, E>
where
    F: A11yInfoWriter,
    E: InteractiveElement,
{
    fn interactivity(&mut self) -> &mut gpui::Interactivity {
        self.inner.interactivity()
    }
}

impl<F, E> gpui::StatefulInteractiveElement for A11yInfoLayer<F, E>
where
    F: A11yInfoWriter,
    E: gpui::StatefulInteractiveElement,
{
}

/// 任意の [`Element`] に a11y ノードへの追加の書き込みを付ける
pub(crate) trait A11yInfoExt: Sized {
    /// `writer` は元の要素が書いた後の a11y ノードに適用される
    fn a11y_info<F: A11yInfoWriter>(self, writer: F) -> A11yInfoLayer<F, Self> {
        A11yInfoLayer {
            writer,
            inner: self,
        }
    }
}

impl<E: Element> A11yInfoExt for E {}
