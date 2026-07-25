use iced::Element;
use iced::{
    Length, Point, Rectangle, Size,
    advanced::{
        Layout, Overlay as OverlayTrait, Widget,
        layout::{Limits, Node},
        mouse, overlay, renderer,
        widget::{self, Tree},
    },
};

struct OverlayTag;

pub struct Overlay<'a, Message, Theme, Renderer = iced::Renderer>
where
    Renderer: iced::advanced::Renderer,
{
    base: Element<'a, Message, Theme, Renderer>,
    visible: bool,
    content_fn: Box<dyn Fn() -> Element<'a, Message, Theme, Renderer> + 'a>,
    content: Option<Element<'a, Message, Theme, Renderer>>,
}

impl<'a, Message, Theme, Renderer> Overlay<'a, Message, Theme, Renderer>
where
    Renderer: iced::advanced::Renderer,
{
    pub fn new(
        base: impl Into<Element<'a, Message, Theme, Renderer>>,
        on_top: impl Fn() -> Element<'a, Message, Theme, Renderer> + 'a,
        visible: bool,
    ) -> Self {
        Self {
            base: base.into(),
            visible,
            content_fn: Box::new(on_top),
            content: None,
        }
    }
}

impl<'a, Message, Theme, Renderer> Widget<Message, Theme, Renderer>
    for Overlay<'a, Message, Theme, Renderer>
where
    Renderer: iced::advanced::Renderer,
{
    fn tag(&self) -> widget::tree::Tag {
        widget::tree::Tag::of::<OverlayTag>()
    }

    fn children(&self) -> Vec<Tree> {
        vec![Tree::new(&self.base), Tree::empty()]
    }

    fn diff(&self, tree: &mut Tree) {
        while tree.children.len() < 2 {
            tree.children.push(Tree::empty());
        }
        tree.children[0].diff(&self.base);
    }

    fn size(&self) -> Size<Length> {
        self.base.as_widget().size()
    }

    fn layout(&mut self, tree: &mut Tree, renderer: &Renderer, limits: &Limits) -> Node {
        while tree.children.len() < 2 {
            tree.children.push(Tree::empty());
        }
        self.base
            .as_widget_mut()
            .layout(&mut tree.children[0], renderer, limits)
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
        self.base.as_widget().draw(
            &tree.children[0],
            renderer,
            theme,
            style,
            layout,
            cursor,
            viewport,
        );
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
        while tree.children.len() < 2 {
            tree.children.push(Tree::empty());
        }

        if self.visible {
            let content = self.content.get_or_insert_with(&self.content_fn);
            tree.children[1].diff(&*content);
            let limits = Limits::new(Size::ZERO, viewport.size());
            let node = content
                .as_widget_mut()
                .layout(&mut tree.children[1], renderer, &limits);
            let size = node.size();
            let x = layout.bounds().x + (layout.bounds().width - size.width) / 2.0;
            let y = (layout.bounds().y + layout.bounds().height - size.height).max(0.0);

            if cursor.is_over(Rectangle::new(Point::new(x, y), size)) {
                return;
            }
        }

        self.base.as_widget_mut().update(
            &mut tree.children[0],
            event,
            layout,
            cursor,
            renderer,
            clipboard,
            shell,
            viewport,
        );
    }

    fn overlay<'b>(
        &'b mut self,
        tree: &'b mut Tree,
        layout: Layout<'b>,
        _renderer: &Renderer,
        _viewport: &Rectangle,
        _translation: iced::Vector,
    ) -> Option<overlay::Element<'b, Message, Theme, Renderer>> {
        while tree.children.len() < 2 {
            tree.children.push(Tree::empty());
        }

        if !self.visible {
            return None;
        }

        let content = self.content.get_or_insert_with(&self.content_fn);
        tree.children[1].diff(&*content);

        Some(
            Content {
                base_bounds: layout.bounds(),
                tree: &mut tree.children[1],
                content,
            }
            .overlay(),
        )
    }
}

impl<'a, Message, Theme, Renderer> From<Overlay<'a, Message, Theme, Renderer>>
    for Element<'a, Message, Theme, Renderer>
where
    Message: 'a,
    Theme: 'a,
    Renderer: iced::advanced::Renderer + 'a,
{
    fn from(overlay: Overlay<'a, Message, Theme, Renderer>) -> Self {
        Element::new(overlay)
    }
}

struct Content<'a, 'b, Message, Theme, Renderer>
where
    Renderer: iced::advanced::Renderer,
{
    base_bounds: Rectangle,
    tree: &'a mut Tree,
    content: &'a mut Element<'b, Message, Theme, Renderer>,
}

impl<'a, 'b, Message, Theme, Renderer> Content<'a, 'b, Message, Theme, Renderer>
where
    Renderer: iced::advanced::Renderer,
{
    fn overlay(self) -> overlay::Element<'a, Message, Theme, Renderer> {
        overlay::Element::new(Box::new(self))
    }
}

impl<'a, 'b, Message, Theme, Renderer> OverlayTrait<Message, Theme, Renderer>
    for Content<'a, 'b, Message, Theme, Renderer>
where
    Renderer: iced::advanced::Renderer,
{
    fn layout(&mut self, renderer: &Renderer, bounds: Size) -> Node {
        let limits = Limits::new(Size::ZERO, bounds);
        let node = self
            .content
            .as_widget_mut()
            .layout(self.tree, renderer, &limits);
        let size = node.size();
        let x = self.base_bounds.x + (self.base_bounds.width - size.width) / 2.0;
        let y = (self.base_bounds.y + self.base_bounds.height - size.height).max(0.0);
        node.move_to(Point::new(x, y))
    }

    fn draw(
        &self,
        renderer: &mut Renderer,
        theme: &Theme,
        style: &renderer::Style,
        layout: Layout<'_>,
        cursor: mouse::Cursor,
    ) {
        self.content.as_widget().draw(
            self.tree,
            renderer,
            theme,
            style,
            layout,
            cursor,
            &layout.bounds(),
        );
    }

    fn update(
        &mut self,
        event: &iced::Event,
        layout: Layout<'_>,
        cursor: mouse::Cursor,
        renderer: &Renderer,
        clipboard: &mut dyn iced::advanced::Clipboard,
        shell: &mut iced::advanced::Shell<'_, Message>,
    ) {
        if cursor.is_over(layout.bounds()) {
            self.content.as_widget_mut().update(
                self.tree,
                event,
                layout,
                cursor,
                renderer,
                clipboard,
                shell,
                &layout.bounds(),
            );
            shell.capture_event();
        }
    }

    fn mouse_interaction(
        &self,
        layout: Layout<'_>,
        cursor: mouse::Cursor,
        renderer: &Renderer,
    ) -> mouse::Interaction {
        if cursor.is_over(layout.bounds()) {
            self.content.as_widget().mouse_interaction(
                self.tree,
                layout,
                cursor,
                &layout.bounds(),
                renderer,
            )
        } else {
            mouse::Interaction::None
        }
    }

    fn operate(
        &mut self,
        layout: Layout<'_>,
        renderer: &Renderer,
        operation: &mut dyn widget::Operation,
    ) {
        self.content
            .as_widget_mut()
            .operate(self.tree, layout, renderer, operation);
    }
}
