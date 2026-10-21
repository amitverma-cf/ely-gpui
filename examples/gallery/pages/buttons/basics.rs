use gpui::{AnyElement, App, ElementId, IntoElement, ParentElement, Styled, div};
use mok_gpui::{
    buttons::{Button, ButtonGroup, ButtonVariant, IconButton},
    primitives::LucideIcon,
    theme::ControlSize,
};

use crate::ui::{code, row, section, specimen, specimens};

const VARIANTS: [(ButtonVariant, &str); 7] = [
    (ButtonVariant::Primary, "Primary"),
    (ButtonVariant::Secondary, "Secondary"),
    (ButtonVariant::Outline, "Outline"),
    (ButtonVariant::Ghost, "Ghost"),
    (ButtonVariant::Subtle, "Subtle"),
    (ButtonVariant::Danger, "Danger"),
    (ButtonVariant::Link, "Link"),
];

pub fn basics(cx: &App) -> AnyElement {
    let sizes = [
        (ControlSize::Sm, "Small"),
        (ControlSize::Md, "Medium"),
        (ControlSize::Lg, "Large"),
    ];

    div()
        .child(
            section("Button", "Primary is ink. Color is saved for meaning.", cx)
                .child(row().children(VARIANTS.map(|(variant, name)| {
                    Button::new(name, name)
                        .variant(variant)
                        .on_click(move |_, _, _| log::info!("gallery: {name} clicked"))
                })))
                .child(row().children(VARIANTS.map(|(variant, name)| {
                    Button::new(ElementId::Name(format!("{name}-off").into()), name)
                        .variant(variant)
                        .disabled(true)
                }))),
        )
        .child(
            section(
                "Sizes",
                "Heights follow density. Tab moves the focus ring.",
                cx,
            )
            .child(row().children(sizes.map(|(size, name)| {
                Button::new(ElementId::Name(format!("size-{name}").into()), name)
                    .primary()
                    .size(size)
            }))),
        )
        .child(
            section(
                "With icons",
                "Leading icon for verbs. Trailing for menus.",
                cx,
            )
            .child(
                row()
                    .child(
                        Button::new("new", "New file")
                            .primary()
                            .icon(LucideIcon::Plus),
                    )
                    .child(Button::new("share", "Share").icon(LucideIcon::Share2))
                    .child(
                        Button::new("sort", "Sort")
                            .variant(ButtonVariant::Ghost)
                            .trailing_icon(LucideIcon::ChevronDown),
                    )
                    .child(
                        Button::new("delete", "Delete")
                            .variant(ButtonVariant::Danger)
                            .icon(LucideIcon::Trash2),
                    ),
            ),
        )
        .child(
            section("Icon button", "Square. Ghost by default.", cx).child(specimens().children(
                sizes.map(|(size, name)| {
                    specimen(
                        name,
                        row()
                            .gap_1()
                            .child(
                                IconButton::new(
                                    ElementId::Name(format!("{name}-a").into()),
                                    LucideIcon::Bold,
                                )
                                .size(size),
                            )
                            .child(
                                IconButton::new(
                                    ElementId::Name(format!("{name}-b").into()),
                                    LucideIcon::Italic,
                                )
                                .size(size),
                            )
                            .child(
                                IconButton::new(
                                    ElementId::Name(format!("{name}-c").into()),
                                    LucideIcon::Underline,
                                )
                                .size(size)
                                .variant(ButtonVariant::Subtle),
                            )
                            .child(
                                IconButton::new(
                                    ElementId::Name(format!("{name}-d").into()),
                                    LucideIcon::Link,
                                )
                                .size(size)
                                .variant(ButtonVariant::Outline),
                            ),
                        cx,
                    )
                }),
            )),
        )
        .child(
            section(
                "ShortcutHint",
                "The keystroke rides along, quiet, in the platform's spelling.",
                cx,
            )
            .child(
                row()
                    .child(Button::new("hint-save", "Save").shortcut("cmd-s"))
                    .child(
                        Button::new("hint-find", "Find")
                            .variant(ButtonVariant::Ghost)
                            .icon(LucideIcon::Search)
                            .shortcut("cmd-shift-f"),
                    ),
            )
            .child(code("Button::new(id, label).shortcut(\"cmd-s\")", cx)),
        )
        .child(
            section("ButtonGroup", "Buttons joined edge to edge.", cx).child(
                row()
                    .child(
                        ButtonGroup::new()
                            .button(Button::new("group-day", "Day"))
                            .button(Button::new("group-week", "Week"))
                            .button(Button::new("group-month", "Month")),
                    )
                    .child(
                        ButtonGroup::new()
                            .button(
                                Button::new("group-prev", "Previous")
                                    .variant(ButtonVariant::Outline)
                                    .icon(LucideIcon::ChevronLeft),
                            )
                            .button(
                                Button::new("group-next", "Next")
                                    .variant(ButtonVariant::Outline)
                                    .trailing_icon(LucideIcon::ChevronRight),
                            ),
                    ),
            ),
        )
        .into_any_element()
}
