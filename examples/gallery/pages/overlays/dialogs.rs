use gpui::{
    AnyElement, App, Entity, IntoElement, ParentElement, SharedString, Styled, Window, div,
    prelude::*,
};
use mok_gpui::{
    buttons::{Button, ButtonVariant},
    forms::{Input, TextInput},
    overlays::{
        AlertDialog, CommandGroup, CommandItem, CommandPalette, ConfirmDialog, Dialog, PromptDialog,
    },
    primitives::Severity,
    theme::ActiveTheme,
    typography::Caption,
};

use crate::{
    probe::probe,
    ui::{keep, row, section, set, specimen},
};

/// Which dialog is open, if any.
#[derive(Clone, Copy, PartialEq)]
enum Open {
    Profile,
    Alert,
    Confirm,
    Prompt,
    Fullscreen,
}

fn opener(
    state: &Entity<Option<Open>>,
    id: &'static str,
    label: &'static str,
    open: Open,
) -> impl IntoElement {
    let state = state.clone();
    probe(
        id,
        Button::new(id, label).on_click(move |_, _, cx| set(&state, Some(open), cx)),
    )
}

pub fn dialogs(window: &mut Window, cx: &mut App) -> impl IntoElement + use<> {
    let open = keep("overlay-open", || None::<Open>, window, cx);
    let said = keep("overlay-said", || None::<SharedString>, window, cx);
    let (name, email, rename) = {
        let make = |window: &mut Window, cx: &mut App, text: &str| {
            let text = text.to_string();
            cx.new(|cx| {
                let mut input = TextInput::new(window, cx);
                input.set_text(text, cx);
                input
            })
        };
        let fields = window.use_keyed_state("overlay-fields", cx, |window, cx| {
            (
                make(window, cx, "Ada Lovelace"),
                make(window, cx, "ada@mok.dev"),
                make(window, cx, "Roadmap"),
            )
        });
        fields.read(cx).clone()
    };
    let shut = {
        let open = open.clone();
        move |_: &mut Window, cx: &mut App| set(&open, None, cx)
    };
    let caption = said
        .read(cx)
        .clone()
        .map_or("each opens over a scrim; Escape closes it".into(), |said| {
            said.to_string()
        });
    let overlay: Option<AnyElement> = (*open.read(cx)).map(|which| {
        let (saved, confirmed, renamed) = (said.clone(), said.clone(), said.clone());
        match which {
            Open::Profile => Dialog::new("profile", "Edit profile", shut.clone())
                .detail("How others see you across the workspace.")
                .child(
                    div()
                        .flex()
                        .flex_col()
                        .gap_3()
                        .child(Input::new(&name))
                        .child(Input::new(&email)),
                )
                .action(|close| {
                    Button::new("profile-cancel", "Cancel")
                        .variant(ButtonVariant::Ghost)
                        .on_click(move |_, window, cx| close(window, cx))
                })
                .action(move |close| {
                    Button::new("profile-save", "Save")
                        .primary()
                        .on_click(move |_, window, cx| {
                            set(&saved, Some("profile saved".into()), cx);
                            close(window, cx);
                        })
                })
                .into_any_element(),
            Open::Alert => AlertDialog::new(
                "expired",
                Severity::Warning,
                "Session expired",
                "Sign in again to keep your changes. Nothing you wrote is lost.",
                shut.clone(),
            )
            .label("Sign in")
            .into_any_element(),
            Open::Confirm => ConfirmDialog::new(
                "delete",
                "Delete this project?",
                "Its pages, files and history go with it. This cannot be undone.",
                shut.clone(),
            )
            .confirm("Delete project")
            .destructive()
            .on_confirm(move |_, cx| set(&confirmed, Some("project deleted".into()), cx))
            .into_any_element(),
            Open::Prompt => PromptDialog::new("rename", "Rename", &rename, shut.clone())
                .label("New name")
                .submit("Rename")
                .check(|text| match text.trim() {
                    "" => Err("A name cannot be empty.".into()),
                    text if text.contains('/') => Err("Names cannot hold a slash.".into()),
                    _ => Ok(()),
                })
                .on_submit(move |text, _, cx| {
                    set(&renamed, Some(format!("renamed to {text}").into()), cx)
                })
                .into_any_element(),
            Open::Fullscreen => Dialog::new("notes", "Release notes", shut.clone())
                .detail("Version 0.9 · September 2026")
                .fullscreen()
                .child(
                    div()
                        .flex()
                        .flex_col()
                        .gap_3()
                        .max_w(
                            cx.theme()
                                .container_width(mok_gpui::theme::ContainerSize::Sm),
                        )
                        .child(Caption::new("Menus nest, filter and follow their buttons."))
                        .child(Caption::new("Palettes pick on release, as buttons do."))
                        .child(Caption::new("Overlays hand focus back on every close.")),
                )
                .into_any_element(),
        }
    });
    section(
        "Dialog / Modal / AlertDialog / ConfirmDialog / PromptDialog / FullscreenDialog",
        "A card over a scrim; Tab stays inside and focus returns on close. An alert holds until acknowledged, a confirm asks first, a prompt checks its one line, and a fullscreen dialog fills the window.",
        cx,
    )
    .child(specimen(
        caption,
        row()
            .child(opener(&open, "edit-profile", "Edit profile", Open::Profile))
            .child(opener(&open, "alert", "Session expired", Open::Alert))
            .child(opener(&open, "confirm", "Delete project", Open::Confirm))
            .child({
                let (open, rename) = (open.clone(), rename.clone());
                probe(
                    "prompt",
                    Button::new("prompt", "Rename").on_click(move |_, _, cx| {
                        rename.update(cx, |input, cx| input.set_text("Roadmap", cx));
                        set(&open, Some(Open::Prompt), cx);
                    }),
                )
            })
            .child(opener(&open, "fullscreen", "Release notes", Open::Fullscreen)),
        cx,
    ))
    .children(overlay)
}

pub fn command_palette(window: &mut Window, cx: &mut App) -> impl IntoElement + use<> {
    let open = keep("palette-open", || false, window, cx);
    let chosen = keep("palette-chosen", || None::<SharedString>, window, cx);
    let is_open = *open.read(cx);

    let overlay = if is_open {
        let (open_close, chosen_set) = (open.clone(), chosen.clone());
        let open_for_close = open_close.clone();
        let close_fn = move |_: &mut Window, cx: &mut App| set(&open_for_close, false, cx);
        let make_item = |id: &'static str,
                         label: &'static str,
                         shortcut: Option<&'static str>,
                         chosen_ref: Entity<Option<SharedString>>,
                         open_ref: Entity<bool>| {
            let item = CommandItem::new(id, label);
            let item = if let Some(s) = shortcut {
                item.shortcut(s)
            } else {
                item
            };
            item.on_select(move |_window, cx| {
                set(&chosen_ref, Some(label.into()), cx);
                set(&open_ref, false, cx);
            })
        };

        Some(
            CommandPalette::new("gallery-command-palette", close_fn)
                .placeholder("Type a command or search...")
                .group(CommandGroup::new(
                    "Navigation",
                    vec![
                        make_item(
                            "nav-home",
                            "Go to Dashboard",
                            Some("⌘D"),
                            chosen_set.clone(),
                            open_close.clone(),
                        ),
                        make_item(
                            "nav-settings",
                            "Open Settings",
                            Some("⌘,"),
                            chosen_set.clone(),
                            open_close.clone(),
                        ),
                        make_item(
                            "nav-docs",
                            "Read Documentation",
                            None,
                            chosen_set.clone(),
                            open_close.clone(),
                        ),
                    ],
                ))
                .group(CommandGroup::new(
                    "Theme & Appearance",
                    vec![
                        make_item(
                            "theme-dark",
                            "Switch to Dark Mode",
                            None,
                            chosen_set.clone(),
                            open_close.clone(),
                        ),
                        make_item(
                            "theme-preset",
                            "Cycle Theme Presets",
                            Some("⌘T"),
                            chosen_set.clone(),
                            open_close.clone(),
                        ),
                        make_item(
                            "theme-contrast",
                            "Toggle High Contrast",
                            None,
                            chosen_set.clone(),
                            open_close.clone(),
                        ),
                    ],
                ))
                .into_any_element(),
        )
    } else {
        None
    };

    let caption = chosen.read(cx).clone().map_or(
        "Modal command palette with search, group headers, and shortcut badges.".into(),
        |c| format!("Last selected: {c}"),
    );

    let open_btn = {
        let open = open.clone();
        probe(
            "command-palette-open",
            Button::new("command-palette-open", "Open Command Palette")
                .primary()
                .on_click(move |_, _, cx| set(&open, true, cx)),
        )
    };

    section(
        "CommandPalette",
        "Fast modal quick-search and command execution overlay with keyboard navigation.",
        cx,
    )
    .child(specimen(caption, row().child(open_btn), cx))
    .children(overlay)
}

pub fn pointers(cx: &App) -> impl IntoElement + use<> {
    section(
        "Tooltip / Sheet / Drawer / DropdownPanel / InlinePopup / ToastViewport",
        "They live with the pieces they are made of.",
        cx,
    )
    .child(Caption::new(
        "Tooltip is primitives::Tooltip, on Primitives, chapter 1. Sheet and Drawer are layout::Sheet and layout::Drawer, on Layout, chapter 3. DropdownPanel and InlinePopup are the Popover above. ToastViewport holds toasts, so it lands with Feedback, chapter 10.",
    ))
}
