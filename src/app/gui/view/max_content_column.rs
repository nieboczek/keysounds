use iced::{
    Element, Length, Padding, Pixels, Rectangle, Size, Vector,
    advanced::{
        Layout, Widget,
        layout::{Limits, Node},
        mouse, overlay, renderer,
        widget::{Operation, Tree},
    },
    alignment,
};

/// A container that distributes its contents vertically, exactly like
/// `iced::widget::Column`, but whose width is always resolved to the widest child.
///
/// Unlike a regular [`Column`](iced::widget::Column), the width never expands
/// to the free space offered by its ancestor, even if one of its children
/// uses `Length::Fill`.
pub struct MaxContent<'a, Message, Theme, Renderer = iced::Renderer>
where
    Renderer: iced::advanced::Renderer,
{
    spacing: f32,
    padding: Padding,
    max_width: f32,
    align: alignment::Horizontal,
    clip: bool,
    children: Vec<Element<'a, Message, Theme, Renderer>>,
}

impl<'a, Message, Theme, Renderer> MaxContent<'a, Message, Theme, Renderer>
where
    Renderer: iced::advanced::Renderer,
{
    /// Creates an empty [`MaxContent`].
    pub fn new() -> Self {
        Self {
            spacing: 0.0,
            padding: Padding::ZERO,
            max_width: f32::INFINITY,
            align: alignment::Horizontal::Left,
            clip: false,
            children: Vec::new(),
        }
    }

    /// Sets the vertical spacing _between_ elements.
    pub fn spacing(mut self, amount: impl Into<Pixels>) -> Self {
        self.spacing = amount.into().0;
        self
    }

    /// Sets the [`Padding`] of the [`MaxContent`].
    pub fn padding<P: Into<Padding>>(mut self, padding: P) -> Self {
        self.padding = padding.into();
        self
    }

    /// Sets the maximum width of the [`MaxContent`].
    pub fn max_width(mut self, max_width: impl Into<Pixels>) -> Self {
        self.max_width = max_width.into().0;
        self
    }

    /// Sets the horizontal alignment of the children within the computed
    /// `max-content` width.
    pub fn align_x(mut self, align: impl Into<alignment::Horizontal>) -> Self {
        self.align = align.into();
        self
    }

    /// Sets whether the contents of the [`MaxContent`] should be clipped on
    /// overflow.
    pub fn clip(mut self, clip: bool) -> Self {
        self.clip = clip;
        self
    }

    /// Adds an element to the [`MaxContent`].
    pub fn push(mut self, child: impl Into<Element<'a, Message, Theme, Renderer>>) -> Self {
        self.children.push(child.into());
        self
    }

    /// Extends the [`MaxContent`] with the given children.
    pub fn extend(
        self,
        children: impl IntoIterator<Item = Element<'a, Message, Theme, Renderer>>,
    ) -> Self {
        children.into_iter().fold(self, Self::push)
    }
}

impl<Message, Theme, Renderer> Default for MaxContent<'_, Message, Theme, Renderer>
where
    Renderer: iced::advanced::Renderer,
{
    fn default() -> Self {
        Self::new()
    }
}

impl<'a, Message, Theme, Renderer> Widget<Message, Theme, Renderer>
    for MaxContent<'a, Message, Theme, Renderer>
where
    Renderer: iced::advanced::Renderer,
{
    fn children(&self) -> Vec<Tree> {
        self.children.iter().map(Tree::new).collect()
    }

    fn diff(&self, tree: &mut Tree) {
        tree.diff_children(&self.children);
    }

    fn size(&self) -> Size<Length> {
        Size {
            width: Length::Shrink,
            height: Length::Shrink,
        }
    }

    fn layout(&mut self, tree: &mut Tree, renderer: &Renderer, limits: &Limits) -> Node {
        let limits = limits.max_width(self.max_width).shrink(self.padding);
        let spacing = self.spacing;

        // First pass: measure every child at its intrinsic (max-content) size.
        //
        // The width axis must be *compressed*: otherwise a child using
        // `Length::Fill` resolves to the free space offered by the ancestor
        // (its `max`), and the column would grow to the full available width
        // like a regular `Column`. Compression makes fluid lengths collapse to
        // their intrinsic width, so `content_width` becomes the widest content.
        let measure_limits =
            Limits::with_compression(Size::ZERO, limits.max(), Size::new(true, true));

        let mut nodes: Vec<Node> = Vec::with_capacity(self.children.len());
        let mut content_width: f32 = 0.0;
        let mut content_height: f32 = 0.0;

        for (child, tree) in self.children.iter_mut().zip(tree.children.iter_mut()) {
            let node = child
                .as_widget_mut()
                .layout(tree, renderer, &measure_limits);

            content_width = content_width.max(node.size().width);
            content_height += node.size().height + spacing;

            nodes.push(node);
        }

        if content_height > 0.0 {
            content_height -= spacing;
        }

        // Resolve the column's width (clamped to `max_width`/ancestor limits).
        // Children will be stretched to this exact width below.
        let size = limits.resolve(
            Length::Shrink,
            Length::Shrink,
            Size::new(content_width, content_height),
        );
        let target_width = size.width;

        // Second pass: force every child to be exactly as wide as the widest
        // content. We re-layout with a *tight* width (and no width compression)
        // so each child node — and any `Length::Fill` grandchild — stretches to
        // the max-content width. This is like `align-items: stretch` in CSS.
        if !self.children.is_empty() {
            let stretch_limits = Limits::with_compression(
                Size::new(target_width, 0.0),
                Size::new(target_width, limits.max().height),
                Size::new(false, true),
            );

            content_height = 0.0;
            for (node, (child, tree)) in nodes
                .iter_mut()
                .zip(self.children.iter_mut().zip(tree.children.iter_mut()))
            {
                *node = child
                    .as_widget_mut()
                    .layout(tree, renderer, &stretch_limits);
                content_height += node.size().height + spacing;
            }

            if content_height > 0.0 {
                content_height -= spacing;
            }
        }

        let size = limits.resolve(
            Length::Shrink,
            Length::Shrink,
            Size::new(target_width, content_height),
        );

        let align_factor = match self.align {
            alignment::Horizontal::Left => 0.0,
            alignment::Horizontal::Center => 2.0,
            alignment::Horizontal::Right => 1.0,
        };

        let mut y = 0.0;
        for node in nodes.iter_mut() {
            let child_size = node.size();

            let x = if align_factor != 0.0 {
                (size.width - child_size.width) / align_factor
            } else {
                0.0
            };

            node.translate_mut(Vector::new(x + self.padding.left, y + self.padding.top));

            y += child_size.height + spacing;
        }

        Node::with_children(size.expand(self.padding), nodes)
    }

    fn operate(
        &mut self,
        tree: &mut Tree,
        layout: Layout<'_>,
        renderer: &Renderer,
        operation: &mut dyn Operation,
    ) {
        operation.container(None, layout.bounds());
        operation.traverse(&mut |operation| {
            for ((child, tree), layout) in self
                .children
                .iter_mut()
                .zip(&mut tree.children)
                .zip(layout.children())
            {
                child
                    .as_widget_mut()
                    .operate(tree, layout, renderer, operation);
            }
        });
    }

    fn update(
        &mut self,
        tree: &mut Tree,
        event: &iced::Event,
        layout: Layout<'_>,
        cursor: mouse::Cursor,
        renderer: &Renderer,
        clipboard: &mut dyn iced::advanced::Clipboard,
        shell: &mut iced::advanced::Shell<'_, Message>,
        viewport: &Rectangle,
    ) {
        for ((child, tree), layout) in self
            .children
            .iter_mut()
            .zip(&mut tree.children)
            .zip(layout.children())
        {
            child.as_widget_mut().update(
                tree, event, layout, cursor, renderer, clipboard, shell, viewport,
            );
        }
    }

    fn mouse_interaction(
        &self,
        tree: &Tree,
        layout: Layout<'_>,
        cursor: mouse::Cursor,
        viewport: &Rectangle,
        renderer: &Renderer,
    ) -> mouse::Interaction {
        self.children
            .iter()
            .zip(&tree.children)
            .zip(layout.children())
            .map(|((child, tree), layout)| {
                child
                    .as_widget()
                    .mouse_interaction(tree, layout, cursor, viewport, renderer)
            })
            .max()
            .unwrap_or_default()
    }

    fn draw(
        &self,
        tree: &Tree,
        renderer: &mut Renderer,
        theme: &Theme,
        style: &renderer::Style,
        layout: Layout<'_>,
        cursor: mouse::Cursor,
        viewport: &Rectangle,
    ) {
        if let Some(clipped_viewport) = layout.bounds().intersection(viewport) {
            let viewport = if self.clip {
                &clipped_viewport
            } else {
                viewport
            };

            for ((child, tree), layout) in self
                .children
                .iter()
                .zip(&tree.children)
                .zip(layout.children())
                .filter(|(_, layout)| layout.bounds().intersects(viewport))
            {
                child
                    .as_widget()
                    .draw(tree, renderer, theme, style, layout, cursor, viewport);
            }
        }
    }

    fn overlay<'b>(
        &'b mut self,
        tree: &'b mut Tree,
        layout: Layout<'b>,
        renderer: &Renderer,
        viewport: &Rectangle,
        translation: Vector,
    ) -> Option<overlay::Element<'b, Message, Theme, Renderer>> {
        overlay::from_children(
            &mut self.children,
            tree,
            layout,
            renderer,
            viewport,
            translation,
        )
    }
}

impl<'a, Message, Theme, Renderer> From<MaxContent<'a, Message, Theme, Renderer>>
    for Element<'a, Message, Theme, Renderer>
where
    Message: 'a,
    Theme: 'a,
    Renderer: iced::advanced::Renderer + 'a,
{
    fn from(content: MaxContent<'a, Message, Theme, Renderer>) -> Self {
        Self::new(content)
    }
}

/// Creates an empty [`MaxContent`] column with the given elements.
pub fn max_content_column<'a, Message, Theme, Renderer>(
    children: impl IntoIterator<Item = Element<'a, Message, Theme, Renderer>>,
) -> MaxContent<'a, Message, Theme, Renderer>
where
    Renderer: iced::advanced::Renderer,
{
    MaxContent::new().extend(children)
}

#[cfg(test)]
mod tests {
    use super::*;
    use iced::widget::{button, column, container, row, scrollable, text};

    type Renderer = ();

    fn layout_node(
        widget: &mut dyn iced::advanced::Widget<(), iced::Theme, Renderer>,
        limits: Limits,
    ) -> Node {
        let mut tree = Tree::new(&mut *widget);
        widget.layout(&mut tree, &(), &limits)
    }

    fn measure_column(
        widget: &mut MaxContent<'_, (), iced::Theme, Renderer>,
        limits: Limits,
    ) -> Size {
        layout_node(widget, limits).size()
    }

    #[test]
    fn shrink_children_hug_content() {
        let mut w = MaxContent::<(), iced::Theme, Renderer>::new()
            .push(text("short"))
            .push(text("a much longer label"));
        let size = measure_column(&mut w, Limits::new(Size::ZERO, Size::new(800.0, 600.0)));
        println!("short text column width = {}", size.width);
        assert!(size.width < 800.0);
    }

    #[test]
    fn fill_child_collapses() {
        let mut w = MaxContent::<(), iced::Theme, Renderer>::new()
            .push(text("short"))
            .push(container(text("also short")).width(Length::Fill));
        let size = measure_column(&mut w, Limits::new(Size::ZERO, Size::new(800.0, 600.0)));
        println!("fill child width = {}", size.width);
        assert!(size.width < 800.0);
    }

    #[test]
    fn filter_chain_structure_hugs() {
        let w = MaxContent::<(), iced::Theme, Renderer>::new();
        let child_a: Element<'_, (), iced::Theme, Renderer> = container(button(
            column([
                text("Preset one").into(),
                text("-> a filter").into(),
                text("k").into(),
            ])
            .spacing(4),
        ))
        .into();
        let child_b: Element<'_, (), iced::Theme, Renderer> = container(button(
            column([
                text("A much longer preset name").into(),
                text("-> a much longer filter that is very descriptive").into(),
                text("Ctrl+Shift+K").into(),
            ])
            .spacing(4),
        ))
        .into();
        let size = measure_column(
            &mut w.push(child_a).push(child_b),
            Limits::new(Size::ZERO, Size::new(800.0, 600.0)),
        );
        println!("filter chain column width = {}", size.width);
        assert!(size.width < 800.0);
    }

    #[test]
    fn full_row_presets_hug() {
        let a: Element<'_, (), iced::Theme, Renderer> = container(button(column([
            text("A preset").into(),
            text("-> a filter").into(),
        ])))
        .into();
        let b: Element<'_, (), iced::Theme, Renderer> = container(button(column([
            text("Much longer preset").into(),
            text("-> a much longer filter description").into(),
        ])))
        .into();
        let presets: Element<'_, (), iced::Theme, Renderer> =
            scrollable(max_content_column([a, b])).spacing(4).into();

        let filters: Element<'_, (), iced::Theme, Renderer> =
            scrollable(column([text("filter").into()]).width(Length::Fill)).into();

        let mut row: Element<'_, (), iced::Theme, Renderer> = row([presets, filters]).into();

        let node = layout_node(
            row.as_widget_mut(),
            Limits::new(Size::ZERO, Size::new(800.0, 600.0)),
        );
        let presets_child = &node.children()[0];
        println!("full row presets width = {}", presets_child.size().width);
        println!("full row total width = {}", node.size().width);
        assert!(presets_child.size().width < 800.0);
    }

    #[test]
    fn all_children_are_same_width() {
        fn item(lines: [&'static str; 3]) -> Element<'static, (), iced::Theme, Renderer> {
            container(button(column(lines.map(|l| text(l).into()))).width(Length::Fill)).into()
        }

        let mut w = MaxContent::<(), iced::Theme, Renderer>::new();
        w = w
            .push(item(["None", "(empty)", "Ctrl+Alt+J"]))
            .push(item([
                "Reverb",
                "-> room_size: 0.9, damping: 0.35, wet: 0.25",
                "Ctrl+Alt+H",
            ]))
            .push(item([
                "Shittify",
                "-> strength: 8, cutoff: 12000",
                "Ctrl+Alt+G",
            ]));

        let node = layout_node(&mut w, Limits::new(Size::ZERO, Size::new(800.0, 600.0)));
        println!("column width = {}", node.size().width);
        for (i, child) in node.children().iter().enumerate() {
            let card = child.size().width;
            // container -> content (button)
            let button = child.children()[0].size().width;
            println!("child {i}: card = {card}, inner button = {button}");
            assert_eq!(card, node.size().width);
            assert_eq!(button, card);
        }
    }
}
