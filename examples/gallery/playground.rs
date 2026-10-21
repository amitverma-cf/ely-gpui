use gpui::{
    Context, FontWeight, InteractiveElement, IntoElement, ParentElement, SharedString,
    StatefulInteractiveElement, Styled, div, px,
};
use mok_gpui::{
    buttons::{Button, ButtonVariant, IconButton},
    forms::Switch,
    primitives::{Icon, LucideIcon},
    theme::{ActiveTheme, ControlSize, Density, TextSize, Theme},
};

use crate::shell::Gallery;

pub fn render_playground_drawer(_gallery: &Gallery, cx: &mut Context<Gallery>) -> impl IntoElement {
    let theme = cx.theme();
    let colors = &theme.colors;

    div()
        .id("playground-drawer")
        .flex_none()
        .w(px(280.0))
        .h_full()
        .bg(colors.surface)
        .border_l_1()
        .border_color(colors.border)
        .flex()
        .flex_col()
        .overflow_y_scroll()
        // Drawer Header
        .child(
            div()
                .flex()
                .items_center()
                .justify_between()
                .px_4()
                .py_3()
                .border_b_1()
                .border_color(colors.border)
                .child(
                    div()
                        .flex()
                        .items_center()
                        .gap_2()
                        .child(Icon::new(LucideIcon::SlidersHorizontal).color(colors.fg))
                        .child(
                            div()
                                .text_size(theme.text_size(TextSize::Sm))
                                .font_weight(FontWeight::SEMIBOLD)
                                .text_color(colors.fg)
                                .child("Theme Playground"),
                        ),
                )
                .child(
                    IconButton::new("close-playground", LucideIcon::X)
                        .variant(ButtonVariant::Ghost)
                        .size(ControlSize::Sm)
                        .on_click(cx.listener(|gallery, _, _, cx| {
                            gallery.toggle_playground(cx);
                        })),
                ),
        )
        // Body controls
        .child(
            div()
                .p_4()
                .flex()
                .flex_col()
                .gap_5()
                // Section: Theme Presets
                .child(
                    div()
                        .flex()
                        .flex_col()
                        .gap_2()
                        .child(
                            div()
                                .text_size(theme.text_size(TextSize::Xs))
                                .font_weight(FontWeight::SEMIBOLD)
                                .text_color(colors.fg_muted)
                                .child("PRESET PALETTES"),
                        )
                        .child(
                            div().flex().flex_wrap().gap_1p5().children(
                                ["mok", "zinc", "slate", "nord", "catppuccin", "tokyo-night"]
                                    .iter()
                                    .map(|&preset| {
                                        let label = match preset {
                                            "mok" => "Mok",
                                            "zinc" => "Zinc",
                                            "slate" => "Slate",
                                            "nord" => "Nord",
                                            "catppuccin" => "Catppuccin",
                                            "tokyo-night" => "Tokyo Night",
                                            other => other,
                                        };
                                        Button::new(
                                            SharedString::from(format!("preset-{}", preset)),
                                            label,
                                        )
                                        .variant(ButtonVariant::Subtle)
                                        .size(ControlSize::Sm)
                                        .on_click(
                                            move |_, _, cx| {
                                                Theme::set_preset(preset, cx);
                                            },
                                        )
                                    }),
                            ),
                        ),
                )
                // Section: Layout Density
                .child(
                    div()
                        .flex()
                        .flex_col()
                        .gap_2()
                        .child(
                            div()
                                .text_size(theme.text_size(TextSize::Xs))
                                .font_weight(FontWeight::SEMIBOLD)
                                .text_color(colors.fg_muted)
                                .child("DENSITY"),
                        )
                        .child(
                            div()
                                .flex()
                                .gap_1p5()
                                .child(
                                    Button::new("density-compact", "Compact")
                                        .variant(if theme.density == Density::Compact {
                                            ButtonVariant::Primary
                                        } else {
                                            ButtonVariant::Subtle
                                        })
                                        .size(ControlSize::Sm)
                                        .on_click(|_, _, cx| {
                                            Theme::update(cx, |t| t.density = Density::Compact);
                                        }),
                                )
                                .child(
                                    Button::new("density-standard", "Standard")
                                        .variant(if theme.density == Density::Standard {
                                            ButtonVariant::Primary
                                        } else {
                                            ButtonVariant::Subtle
                                        })
                                        .size(ControlSize::Sm)
                                        .on_click(|_, _, cx| {
                                            Theme::update(cx, |t| t.density = Density::Standard);
                                        }),
                                )
                                .child(
                                    Button::new("density-comfortable", "Comfortable")
                                        .variant(if theme.density == Density::Comfortable {
                                            ButtonVariant::Primary
                                        } else {
                                            ButtonVariant::Subtle
                                        })
                                        .size(ControlSize::Sm)
                                        .on_click(|_, _, cx| {
                                            Theme::update(cx, |t| t.density = Density::Comfortable);
                                        }),
                                ),
                        ),
                )
                // Section: Radius Scale
                .child(
                    div()
                        .flex()
                        .flex_col()
                        .gap_2()
                        .child(
                            div()
                                .text_size(theme.text_size(TextSize::Xs))
                                .font_weight(FontWeight::SEMIBOLD)
                                .text_color(colors.fg_muted)
                                .child("CORNER RADIUS"),
                        )
                        .child(
                            div().flex().gap_1p5().children(
                                [(0.5, "0.5x"), (1.0, "1.0x"), (1.5, "1.5x"), (2.0, "2.0x")]
                                    .iter()
                                    .map(|&(scale, label)| {
                                        let active = (theme.radius_scale - scale).abs() < 0.05;
                                        Button::new(
                                            SharedString::from(format!("radius-{}", label)),
                                            label,
                                        )
                                        .variant(if active {
                                            ButtonVariant::Primary
                                        } else {
                                            ButtonVariant::Subtle
                                        })
                                        .size(ControlSize::Sm)
                                        .on_click(
                                            move |_, _, cx| {
                                                Theme::update(cx, |t| t.radius_scale = scale);
                                            },
                                        )
                                    }),
                            ),
                        ),
                )
                // Section: Font Scale
                .child(
                    div()
                        .flex()
                        .flex_col()
                        .gap_2()
                        .child(
                            div()
                                .text_size(theme.text_size(TextSize::Xs))
                                .font_weight(FontWeight::SEMIBOLD)
                                .text_color(colors.fg_muted)
                                .child("FONT SCALE"),
                        )
                        .child(
                            div().flex().gap_1p5().children(
                                [(0.9, "0.9x"), (1.0, "1.0x"), (1.1, "1.1x"), (1.2, "1.2x")]
                                    .iter()
                                    .map(|&(scale, label)| {
                                        let active = (theme.font_scale - scale).abs() < 0.05;
                                        Button::new(
                                            SharedString::from(format!("font-{}", label)),
                                            label,
                                        )
                                        .variant(if active {
                                            ButtonVariant::Primary
                                        } else {
                                            ButtonVariant::Subtle
                                        })
                                        .size(ControlSize::Sm)
                                        .on_click(
                                            move |_, _, cx| {
                                                Theme::update(cx, |t| t.font_scale = scale);
                                            },
                                        )
                                    }),
                            ),
                        ),
                )
                // Section: Accessibility Toggles
                .child(
                    div()
                        .flex()
                        .flex_col()
                        .gap_3()
                        .pt_2()
                        .border_t_1()
                        .border_color(colors.border)
                        .child(
                            div()
                                .flex()
                                .items_center()
                                .justify_between()
                                .child(
                                    div()
                                        .text_size(theme.text_size(TextSize::Sm))
                                        .text_color(colors.fg)
                                        .child("High Contrast"),
                                )
                                .child(
                                    Switch::new("toggle-high-contrast", theme.high_contrast())
                                        .on_change(|on, _, cx| {
                                            Theme::set_high_contrast(on, cx);
                                        }),
                                ),
                        )
                        .child(
                            div()
                                .flex()
                                .items_center()
                                .justify_between()
                                .child(
                                    div()
                                        .text_size(theme.text_size(TextSize::Sm))
                                        .text_color(colors.fg)
                                        .child("Color-Blind Safe"),
                                )
                                .child(
                                    Switch::new("toggle-color-blind", theme.color_blind_safe())
                                        .on_change(|on, _, cx| {
                                            Theme::set_color_blind_safe(on, cx);
                                        }),
                                ),
                        ),
                )
                // Reset button
                .child(
                    div().pt_3().child(
                        Button::new("reset-playground", "Reset to Defaults")
                            .variant(ButtonVariant::Ghost)
                            .size(ControlSize::Sm)
                            .on_click(|_, _, cx| {
                                Theme::set_preset("mok", cx);
                                Theme::update(cx, |t| {
                                    t.radius_scale = 1.0;
                                    t.font_scale = 1.0;
                                    t.density = Density::Comfortable;
                                });
                            }),
                    ),
                ),
        )
}
