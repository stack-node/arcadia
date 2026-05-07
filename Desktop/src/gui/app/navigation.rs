use arcadia_core::navigation;
use gpui::{div, rgb, Context, Div, FontWeight, ParentElement, Styled, Window};

use super::ArcadiaRoot;

impl ArcadiaRoot {
    pub(crate) fn render_active_content(
        &self,
        window: &mut Window,
        cx: &mut Context<Self>,
        active_page: Option<&'static navigation::NavigationPageDefinition>,
        is_dark: bool,
    ) -> Div {
        if self.active_page_id == "utility.shell" {
            return div()
                .flex_1()
                .h_full()
                .min_h_0()
                .p_2()
                .child(self.shell_panel(window, cx));
        }
        if self.active_page_id == "global.modules" {
            return div().w_full().p_6().child(self.modules_panel(cx, is_dark));
        }
        if self.active_page_id == "network.nodes" {
            return div()
                .w_full()
                .p_6()
                .child(self.lan_nodes_panel(cx, is_dark));
        }
        div()
            .w_full()
            .p_6()
            .flex()
            .flex_col()
            .items_center()
            .gap_3()
            .py_16()
            .child(
                div()
                    .text_3xl()
                    .font_weight(FontWeight::BOLD)
                    .child(self.title.clone()),
            )
            .child(
                div()
                    .text_2xl()
                    .text_color(if is_dark {
                        rgb(0xe5e7eb)
                    } else {
                        rgb(0x1f2937)
                    })
                    .child(active_page.map_or("Page", |page| page.title)),
            )
            .child(
                div()
                    .text_base()
                    .text_color(if is_dark {
                        rgb(0x9ca3af)
                    } else {
                        rgb(0x6b7280)
                    })
                    .child(
                        active_page.map_or("Page definition not found.", |page| page.description),
                    ),
            )
    }

    pub fn is_page_visible(&self, page_id: &str) -> bool {
        let Some(page) = Self::page_by_id(page_id) else {
            return false;
        };
        match page.required_module {
            Some(module_name) => self.is_module_enabled(module_name),
            None => true,
        }
    }

    pub fn active_page_if_visible(&self) -> Option<&'static navigation::NavigationPageDefinition> {
        if self.is_page_visible(self.active_page_id) {
            Self::page_by_id(self.active_page_id)
        } else {
            None
        }
    }

    pub fn visible_groups(&self) -> Vec<&'static navigation::NavigationGroupDefinition> {
        navigation::GROUP_DEFINITIONS
            .iter()
            .filter(|group| {
                group
                    .pages
                    .iter()
                    .any(|page_id| self.is_page_visible(page_id))
            })
            .collect()
    }

    pub fn ensure_valid_navigation_selection(&mut self) {
        let visible_groups = self.visible_groups();
        let group_is_visible = visible_groups
            .iter()
            .any(|group| group.id == self.active_group_id);
        if !group_is_visible {
            if let Some(group) = visible_groups.first() {
                self.active_group_id = group.id;
            } else {
                self.active_group_id = navigation::DEFAULT_GROUP_ID;
            }
        }

        let active_group = visible_groups
            .iter()
            .copied()
            .find(|group| group.id == self.active_group_id)
            .or_else(|| visible_groups.first().copied());
        let page_is_visible = self.is_page_visible(self.active_page_id);
        if !page_is_visible {
            if let Some(group) = active_group {
                if let Some(first_visible_page) = group
                    .pages
                    .iter()
                    .find(|page_id| self.is_page_visible(page_id))
                {
                    self.active_page_id = first_visible_page;
                }
            }
        }
    }

    pub fn page_by_id(page_id: &str) -> Option<&'static navigation::NavigationPageDefinition> {
        navigation::page_by_id(page_id)
    }

    pub fn group_by_id(group_id: &'static str) -> &'static navigation::NavigationGroupDefinition {
        navigation::group_by_id(group_id).unwrap_or(&navigation::GROUP_DEFINITIONS[0])
    }
}
