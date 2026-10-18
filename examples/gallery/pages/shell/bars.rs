use ely_gpui_component::{
    buttons::{Button, IconButton},
    primitives::LucideIcon,
    shell::{
        ActivityBar, ActivityItem, StatusBar, StatusBarItem, TabBar, Toolbar, ToolbarGroup,
        ToolbarSeparator, WindowTab,
    },
    theme::{ActiveTheme, ControlSize, TextSize},
    typography::Caption,
};
use gpui::{App, IntoElement, ParentElement, SharedString, Styled, Window, div, px};

use super::chrome::window_frame;
use crate::{probe::probe, ui::section};

fn tool(id: &'static str, icon: LucideIcon) -> IconButton {
    IconButton::new(id, icon).size(ControlSize::Sm)
}

pub fn toolbar(cx: &App) -> impl IntoElement + use<> {
    section(
        "Toolbar / ToolbarGroup / ToolbarSeparator",
        "Tools in groups, hairlines between, one action apart.",
        cx,
    )
    .child(
        window_frame(560.0, 44.0, cx)
            .justify_center()
            .bg(cx.theme().colors.bg)
            .child(
                Toolbar::new()
                    .w_full()
                    .child(
                        ToolbarGroup::new()
                            .child(tool("tb-undo", LucideIcon::Undo2))
                            .child(tool("tb-redo", LucideIcon::Redo2)),
                    )
                    .child(ToolbarSeparator)
                    .child(
                        ToolbarGroup::new()
                            .child(tool("tb-bold", LucideIcon::Bold))
                            .child(tool("tb-italic", LucideIcon::Italic))
                            .child(tool("tb-underline", LucideIcon::Underline)),
                    )
                    .child(ToolbarSeparator)
                    .child(
                        ToolbarGroup::new()
                            .child(tool("tb-left", LucideIcon::AlignLeft))
                            .child(tool("tb-center", LucideIcon::AlignCenter))
                            .child(tool("tb-right", LucideIcon::AlignRight)),
                    )
                    .child(div().flex_1())
                    .child(
                        Button::new("tb-share", "Share")
                            .size(ControlSize::Sm)
                            .icon(LucideIcon::Share2),
                    ),
            ),
    )
}

pub fn status_bar(cx: &App) -> impl IntoElement + use<> {
    section(
        "StatusBar / StatusBarItem",
        "Small type at the foot. Items that act show it on hover.",
        cx,
    )
    .child(
        window_frame(560.0, 96.0, cx).justify_end().child(
            StatusBar::new()
                .left(
                    StatusBarItem::new("sb-branch")
                        .icon(LucideIcon::GitBranch)
                        .label("main")
                        .tooltip("Switch branch")
                        .on_click(|_, _, _| log::info!("gallery: branch")),
                )
                .left(
                    StatusBarItem::new("sb-sync")
                        .icon(LucideIcon::RefreshCw)
                        .label("Synced"),
                )
                .right(StatusBarItem::new("sb-cursor").label("Ln 12, Col 4"))
                .right(StatusBarItem::new("sb-encoding").label("UTF-8"))
                .right(
                    StatusBarItem::new("sb-bell")
                        .icon(LucideIcon::Bell)
                        .tooltip("No notifications")
                        .on_click(|_, _, _| log::info!("gallery: notifications")),
                ),
        ),
    )
}

pub fn activity_bar(window: &mut Window, cx: &mut App) -> impl IntoElement + use<> {
    let selected =
        window.use_keyed_state("activity-selected", cx, |_, _| SharedString::from("files"));
    let current = selected.read(cx).clone();
    let theme = cx.theme();
    section(
        "ActivityBar / NavigationRail",
        "An icon rail. The mark grows in beside the chosen view.",
        cx,
    )
    .child(
        window_frame(560.0, 400.0, cx)
            .flex_row()
            .child(probe(
                "activity",
                ActivityBar::new("activity-bar")
                    .item(ActivityItem::new("files", LucideIcon::Folder, "Explorer"))
                    .item(ActivityItem::new("search", LucideIcon::Search, "Search"))
                    .item(
                        ActivityItem::new("git", LucideIcon::GitBranch, "Source Control").badge(3),
                    )
                    .item(ActivityItem::new("debug", LucideIcon::Bug, "Run and Debug"))
                    .item(ActivityItem::new(
                        "extensions",
                        LucideIcon::Puzzle,
                        "Extensions",
                    ))
                    .footer(ActivityItem::new("account", LucideIcon::User, "Account"))
                    .footer(ActivityItem::new(
                        "settings",
                        LucideIcon::Settings,
                        "Settings",
                    ))
                    .selected(current.clone())
                    .on_select(move |id, _, cx| {
                        let id = id.clone();
                        selected.update(cx, |selected, cx| {
                            *selected = id;
                            cx.notify();
                        })
                    }),
            ))
            .child(
                div()
                    .flex_1()
                    .p_5()
                    .bg(theme.colors.bg)
                    .text_size(theme.text_size(TextSize::Sm))
                    .text_color(theme.colors.fg_muted)
                    .child(SharedString::from(format!("View: {current}"))),
            ),
    )
    .child(Caption::new("NavigationRail is the same rail."))
}

#[derive(Clone)]
struct Tabs {
    open: Vec<(SharedString, SharedString, LucideIcon)>,
    selected: SharedString,
    next: u32,
}

pub fn tab_bar(window: &mut Window, cx: &mut App) -> impl IntoElement + use<> {
    let state = window.use_keyed_state("window-tabs", cx, |_, _| Tabs {
        open: vec![
            ("inbox".into(), "Inbox".into(), LucideIcon::Inbox),
            ("roadmap".into(), "Roadmap".into(), LucideIcon::Map),
            (
                "notes".into(),
                "Meeting notes".into(),
                LucideIcon::NotebookPen,
            ),
        ],
        selected: "roadmap".into(),
        next: 1,
    });
    let tabs = state.read(cx).clone();
    let (select, close, add, reorder) = (state.clone(), state.clone(), state.clone(), state);
    let mut bar = TabBar::new("window-tab-bar")
        .selected(tabs.selected.clone())
        .on_select(move |id, _, cx| {
            let id = id.clone();
            select.update(cx, |tabs, cx| {
                tabs.selected = id;
                cx.notify();
            })
        })
        .on_close(move |id, _, cx| {
            let id = id.clone();
            close.update(cx, |tabs, cx| {
                tabs.open.retain(|(tab, _, _)| *tab != id);
                if tabs.selected == id {
                    tabs.selected = tabs
                        .open
                        .first()
                        .map(|(tab, _, _)| tab.clone())
                        .unwrap_or_default();
                }
                cx.notify();
            })
        })
        .on_add(move |_, cx| {
            add.update(cx, |tabs, cx| {
                let id = SharedString::from(format!("draft-{}", tabs.next));
                tabs.open.push((
                    id.clone(),
                    format!("Draft {}", tabs.next).into(),
                    LucideIcon::FileText,
                ));
                tabs.next += 1;
                tabs.selected = id;
                cx.notify();
            })
        })
        .on_reorder(move |from, to, _, cx| {
            reorder.update(cx, |tabs, cx| {
                let moved = tabs.open.remove(from);
                tabs.open.insert(to, moved);
                cx.notify();
            })
        });
    for (id, title, icon) in tabs.open {
        bar = bar.tab(WindowTab::new(id, title).icon(icon));
    }
    section(
        "TabBar",
        "Window tabs. New ones rise in; drag one onto another to reorder.",
        cx,
    )
    .child(
        window_frame(560.0, 120.0, cx)
            .child(
                div()
                    .p_1p5()
                    .child(probe("tab-bar", div().w(px(548.0)).child(bar))),
            )
            .child(div().flex_1().bg(cx.theme().colors.bg)),
    )
}
