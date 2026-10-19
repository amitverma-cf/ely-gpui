use ely_gpui::{
    Assets,
    assets::Image,
    primitives::{FocusScope, HeroIcon, HugeIcon, Icon, LucideIcon},
    theme::{ActiveTheme, IconSize},
};
use gpui::{
    App, AppContext, Context, FocusHandle, FontWeight, IntoElement, ParentElement, Render, Styled,
    Window, WindowOptions, div,
};

struct ConfigDemo {
    focus: FocusHandle,
}

impl Render for ConfigDemo {
    fn render(&mut self, _: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let theme = cx.theme();
        FocusScope::new(&self.focus)
            .root()
            .size_full()
            .flex()
            .flex_col()
            .gap_4()
            .p_8()
            .bg(theme.colors.bg)
            .text_color(theme.colors.fg)
            .child(
                div()
                    .flex()
                    .flex_col()
                    .gap_1()
                    .child(
                        div()
                            .font_weight(FontWeight::BOLD)
                            .text_lg()
                            .child("Ely Root Configuration & Providers"),
                    )
                    .child(div().text_color(theme.colors.fg_muted).child(
                        "Compile-time baked tokens, multi-pack icons, and type-safe assets.",
                    )),
            )
            .child(
                div()
                    .flex()
                    .items_center()
                    .gap_4()
                    .p_4()
                    .bg(theme.colors.surface)
                    .rounded_md()
                    .child(
                        div()
                            .flex()
                            .items_center()
                            .gap_2()
                            .child(Icon::new(LucideIcon::Sparkles).size(IconSize::Md))
                            .child("LucideIcon::Sparkles"),
                    )
                    .child(
                        div()
                            .flex()
                            .items_center()
                            .gap_2()
                            .child(Icon::new(HeroIcon::Sparkles).size(IconSize::Md))
                            .child("HeroIcon::Sparkles"),
                    )
                    .child(
                        div()
                            .flex()
                            .items_center()
                            .gap_2()
                            .child(Icon::new(HugeIcon::AiChat01).size(IconSize::Md))
                            .child("HugeIcon::AiChat01"),
                    ),
            )
            .child(
                div()
                    .flex()
                    .flex_col()
                    .gap_2()
                    .p_4()
                    .bg(theme.colors.sunken)
                    .rounded_md()
                    .child(
                        div()
                            .font_weight(FontWeight::SEMIBOLD)
                            .child("Type-Safe Image Assets:"),
                    )
                    .child(div().child(format!("Image::Logo path: {}", Image::Logo.path())))
                    .child(div().child(format!("Image::Logo mime: {}", Image::Logo.mime_type())))
                    .child(div().child(format!(
                        "Image::Logo embedded bytes: {} bytes",
                        Image::Logo.bytes().len()
                    ))),
            )
            .child(
                div()
                    .flex()
                    .gap_4()
                    .child(
                        div()
                            .p_2()
                            .border_1()
                            .border_color(theme.colors.border)
                            .rounded_sm()
                            .child(format!(
                                "Sidebar collapsed: {:?}px",
                                theme.sidebar_width(false)
                            )),
                    )
                    .child(
                        div()
                            .p_2()
                            .border_1()
                            .border_color(theme.colors.border)
                            .rounded_sm()
                            .child(format!("Dialog width: {:?}px", theme.dialog_width())),
                    )
                    .child(
                        div()
                            .p_2()
                            .border_1()
                            .border_color(theme.colors.border)
                            .rounded_sm()
                            .child(format!("Titlebar height: {:?}px", theme.titlebar_height())),
                    ),
            )
    }
}

fn main() {
    gpui_platform::application()
        .with_assets(Assets)
        .run(|cx: &mut App| {
            ely_gpui::init(cx).expect("Ely failed to start");
            cx.open_window(WindowOptions::default(), |window, cx| {
                cx.new(|cx| {
                    let focus = cx.focus_handle();
                    window.focus(&focus, cx);
                    ConfigDemo { focus }
                })
            })
            .expect("window opens");
            cx.activate(true);
        });
}
