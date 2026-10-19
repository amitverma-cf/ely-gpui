use ely_gpui::{
    buttons::{ButtonVariant, IconButton},
    forms::{Choice, TextInput},
    layout::Sidebar,
    navigation::{
        Anchor, BackForwardNavigation, GoToLine, NavGroup, NavItem, NavigationMenu, Sections,
        TableOfContents,
    },
    primitives::LucideIcon,
    theme::{ActiveTheme, Radius},
    typography::{Heading, Paragraph},
};
use gpui::{
    App, Entity, InteractiveElement, IntoElement, ParentElement, ScrollHandle, SharedString,
    StatefulInteractiveElement, Styled, Window, div, px,
};

use crate::{
    probe::probe,
    ui::{keep, section, set, specimen},
};

pub fn sidebar(window: &mut Window, cx: &mut App) -> impl IntoElement + use<> {
    let here = keep("nav-here", || SharedString::from("inbox"), window, cx);
    let folded = keep("nav-folded", || false, window, cx);
    let open = keep("nav-open", || true, window, cx);
    let (now, fold, shown) = (here.read(cx).clone(), *folded.read(cx), *open.read(cx));
    let item = |id: &'static str, icon: LucideIcon, label: &'static str| {
        let here = here.clone();
        NavItem::new(id, icon, label)
            .active(now.as_ref() == id)
            .folded(fold)
            .on_click(move |_, cx| set(&here, id.into(), cx))
    };
    let toggle = folded.clone();
    let groups = open.clone();
    section(
        "Sidebar Navigation / NavItem / NavGroup",
        "Places in a side column. Groups fold; folded to a rail, items keep their icons and name themselves on hover.",
        cx,
    )
    .child(probe(
        "nav-sidebar",
        div()
            .flex()
            .w(px(560.0))
            .h(px(300.0))
            .border_1()
            .border_color(cx.theme().colors.border)
            .rounded(cx.theme().radius(Radius::Lg))
            .overflow_hidden()
            .child(
                Sidebar::new("nav-sidebar", fold).child(
                    div()
                        .flex()
                        .flex_col()
                        .gap_3()
                        .p_2()
                        .child(item("inbox", LucideIcon::Inbox, "Inbox").count("12"))
                        .child(item("drafts", LucideIcon::FileText, "Drafts"))
                        .child(
                            NavGroup::new("nav-projects", "Projects", shown)
                                .folded(fold)
                                .on_toggle(move |next, _, cx| set(&groups, next, cx))
                                .child(item("ely", LucideIcon::Box, "Ely"))
                                .child(item("website", LucideIcon::Globe, "Website"))
                                .child(item("research", LucideIcon::BookOpen, "Research")),
                        ),
                ),
            )
            .child(
                div().p_3().child(
                    IconButton::new("nav-fold", LucideIcon::PanelLeft)
                        .variant(ButtonVariant::Ghost)
                        .tooltip(if fold { "Unfold" } else { "Fold" })
                        .on_click(move |_, _, cx| set(&toggle, !fold, cx)),
                ),
            ),
    ))
}

pub fn menu(window: &mut Window, cx: &mut App) -> impl IntoElement + use<> {
    let went = keep("menu-went", || None::<SharedString>, window, cx);
    let caption = went
        .read(cx)
        .clone()
        .map_or("pick a link".to_string(), |value| {
            format!("went to {value}")
        });
    section(
        "NavigationMenu",
        "A click opens an entry's panel; while one is open, hovering another switches. Arrows walk it, Escape closes.",
        cx,
    )
    .child(specimen(
        caption,
        probe(
            "nav-menu",
            NavigationMenu::new("nav-menu")
                .entry(
                    "Product",
                    [
                        Choice::new("tour", "Tour").icon(LucideIcon::Compass).note("What it does, in five minutes"),
                        Choice::new("pricing", "Pricing").icon(LucideIcon::CreditCard).note("Plans for one to a thousand"),
                        Choice::new("changes", "Changelog").icon(LucideIcon::History).note("Every release, dated"),
                        Choice::new("status", "Status").icon(LucideIcon::Activity).note("Uptime, live"),
                    ],
                )
                .entry(
                    "Docs",
                    [
                        Choice::new("guide", "Guide").icon(LucideIcon::BookOpen).note("From install to ship"),
                        Choice::new("api", "API").icon(LucideIcon::Code).note("Every type and call"),
                        Choice::new("examples", "Examples").icon(LucideIcon::Layers).note("Small apps to copy"),
                        Choice::new("faq", "Questions").icon(LucideIcon::CircleHelp).note("Asked and answered"),
                    ],
                )
                .on_select(move |value, _, cx| set(&went, Some(value.clone()), cx)),
        ),
        cx,
    ))
}

pub fn history(window: &mut Window, cx: &mut App) -> impl IntoElement + use<> {
    let at = keep("history-at", || 3usize, window, cx);
    let now = *at.read(cx);
    let places = [
        ("home", "Home", LucideIcon::House),
        ("search", "Search: gpui", LucideIcon::Search),
        ("repo", "zed-industries/zed", LucideIcon::GitBranch),
        ("file", "crates/gpui/src/window.rs", LucideIcon::FileText),
    ];
    let label = places[now].1;
    section(
        "BackForwardNavigation / History",
        "Back and forward step one place; the clock lists every place, newest first.",
        cx,
    )
    .child(specimen(
        format!("at {label}"),
        probe(
            "history",
            BackForwardNavigation::new(
                "history",
                places.map(|(value, label, icon)| Choice::new(value, label).icon(icon)),
                now,
            )
            .on_go(move |to, _, cx| set(&at, to, cx)),
        ),
        cx,
    ))
}

const CHAPTERS: [(&str, &str, usize); 5] = [
    ("start", "Getting started", 0),
    ("install", "Install", 1),
    ("first", "A first window", 1),
    ("theme", "Theme and tokens", 0),
    ("motion", "Motion", 0),
];

pub fn contents(window: &mut Window, cx: &mut App) -> impl IntoElement + use<> {
    let scroll = keep("toc-scroll", ScrollHandle::new, window, cx)
        .read(cx)
        .clone();
    let sections: Entity<Sections> = {
        let scroll = scroll.clone();
        keep("toc-sections", move || Sections::new(scroll), window, cx)
    };
    let body = "Each section is long enough to scroll past. The line in the table follows the section in view; a click brings a section to the top.";
    let document = CHAPTERS.iter().map(|(value, title, depth)| {
        Anchor::new(&sections, *value).child(
            div()
                .flex()
                .flex_col()
                .gap_2()
                .pb_8()
                .child(if *depth == 0 {
                    Heading::h4(*title)
                } else {
                    Heading::h5(*title)
                })
                .child(Paragraph::new(body))
                .child(Paragraph::new(body)),
        )
    });
    let toc = CHAPTERS.iter().fold(
        TableOfContents::new("toc", &sections),
        |toc, (value, title, depth)| toc.entry(*value, *title, *depth),
    );
    section(
        "Anchor / TableOfContents",
        "Anchors mark where sections start. The table marks the one in view and brings any to the top.",
        cx,
    )
    .child(
        div()
            .flex()
            .items_start()
            .gap_6()
            .child(probe("toc", div().w(px(200.0)).child(toc)))
            .child(
                div()
                    .id("toc-document")
                    .w(px(420.0))
                    .h(px(260.0))
                    .overflow_y_scroll()
                    .track_scroll(&scroll)
                    .children(document),
            ),
    )
}

pub fn go_to_line(window: &mut Window, cx: &mut App) -> impl IntoElement + use<> {
    let state: Entity<TextInput> = window.use_keyed_state("goto-line", cx, |window, cx| {
        TextInput::new(window, cx).placeholder("Line or line:column")
    });
    let went = keep("goto-went", || None::<(usize, Option<usize>)>, window, cx);
    let caption = match *went.read(cx) {
        None => "Enter jumps".to_string(),
        Some((line, None)) => format!("jumped to line {line}"),
        Some((line, Some(column))) => format!("jumped to {line}:{column}"),
    };
    section(
        "JumpTo / GoToLine",
        "Line, or line and column. The note says where you will land, or why you cannot.",
        cx,
    )
    .child(specimen(
        caption,
        probe(
            "goto",
            div().w(px(260.0)).child(
                GoToLine::new("goto", &state, 240)
                    .on_jump(move |line, column, _, cx| set(&went, Some((line, column)), cx)),
            ),
        ),
        cx,
    ))
}
