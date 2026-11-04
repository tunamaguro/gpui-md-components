//! アプリ全体で共有するテーマ

use std::sync::Arc;

use gpui::{App, Global, Window, WindowAppearance};
use md3_color::SchemeError;

use crate::settings::{AppearanceMode, Brightness, ThemeSettings};
use crate::theme::Theme;

struct GlobalTheme(Arc<Theme>);

impl Global for GlobalTheme {}

/// アプリ全体で共有するテーマを、[`App`] から参照・変更する
///
/// テーマは [`init_md3_theme`](Self::init_md3_theme) か [`update_md3_theme`](Self::update_md3_theme)
/// を最初に呼んだときに作られる。[`AppearanceMode::System`] のテーマは、
/// [`init_md3_theme`](Self::init_md3_theme) に渡したウィンドウの外観に合わせて作り直される
pub trait ThemeExt {
    /// `window` の外観をテーマに反映し、ウィンドウを閉じるまで外観の変化に追従させる
    ///
    /// ウィンドウを開いたときに 1 回呼ぶ。呼ばなかったウィンドウの外観の変化には追従しない
    fn init_md3_theme(&mut self, window: &mut Window);

    /// 今のテーマ
    ///
    /// # Panics
    ///
    /// [`init_md3_theme`](Self::init_md3_theme) と [`update_md3_theme`](Self::update_md3_theme)
    /// のどちらもまだ呼んでいない場合
    fn md3_theme(&self) -> &Arc<Theme>;

    /// `f` で設定を変えてテーマを作り直し、すべてのウィンドウを再描画する
    ///
    /// 変えた設定を解決できなければ [`SchemeError`] を返し、設定とテーマは変えない。
    /// 明示的な明暗を選ぶと、OS が描くウィンドウ枠の外観も合わせる (macOS のみ)
    fn update_md3_theme(&mut self, f: impl FnOnce(&mut ThemeSettings)) -> Result<(), SchemeError>;
}

impl ThemeExt for App {
    fn init_md3_theme(&mut self, window: &mut Window) {
        init_global(self);
        window
            .observe_window_appearance(|window, cx| {
                follow_system(Brightness::from_window_appearance(window.appearance()), cx);
            })
            .detach();
        // 購読は登録時の外観を通知しないため、ここで合わせる
        follow_system(
            Brightness::from_window_appearance(window.appearance()),
            self,
        );
    }

    fn md3_theme(&self) -> &Arc<Theme> {
        &self
            .try_global::<GlobalTheme>()
            .expect("call init_md3_theme or update_md3_theme before reading the theme")
            .0
    }

    fn update_md3_theme(&mut self, f: impl FnOnce(&mut ThemeSettings)) -> Result<(), SchemeError> {
        init_global(self);
        let previous = self.global::<GlobalTheme>().0.clone();
        let mut settings = previous.settings().clone();
        f(&mut settings);

        let forced = |mode| match mode {
            AppearanceMode::System => None,
            AppearanceMode::Light => Some(WindowAppearance::Light),
            AppearanceMode::Dark => Some(WindowAppearance::Dark),
        };
        self.set_window_appearance(forced(settings.appearance));
        // 明示指定の間は App::window_appearance がその指定を返すため、指定を変えてから OS の外観を読む
        let system = Brightness::from_window_appearance(self.window_appearance());
        match Theme::resolve(settings, system) {
            Ok(theme) => {
                self.set_global(GlobalTheme(Arc::new(theme)));
                self.refresh_windows();
                Ok(())
            }
            Err(error) => {
                self.set_window_appearance(forced(previous.settings().appearance));
                Err(error)
            }
        }
    }
}

fn init_global(cx: &mut App) {
    if cx.has_global::<GlobalTheme>() {
        return;
    }
    let system = Brightness::from_window_appearance(cx.window_appearance());
    let theme = Theme::resolve(ThemeSettings::default(), system)
        .expect("the default settings always resolve, so failure is a bug");
    cx.set_global(GlobalTheme(Arc::new(theme)));
}

fn follow_system(system: Brightness, cx: &mut App) {
    let theme = &cx.global::<GlobalTheme>().0;
    if theme.settings().appearance != AppearanceMode::System || theme.brightness() == system {
        return;
    }
    // 外観だけを変えた設定の解決は、今のテーマを作れた時点で成功する
    if let Ok(theme) = Theme::resolve(theme.settings().clone(), system) {
        cx.set_global(GlobalTheme(Arc::new(theme)));
        cx.refresh_windows();
    }
}

#[cfg(test)]
mod tests {
    use gpui::{Empty, TestAppContext, WindowHandle};
    use md3_color::{Argb, ColorRole, SpecVersion, Variant};

    use super::*;

    fn open_window(cx: &mut TestAppContext) -> WindowHandle<Empty> {
        let window = cx.add_window(|_, _| Empty);
        window
            .update(cx, |_, window, cx| {
                cx.init_md3_theme(window);
            })
            .expect("開いたウィンドウは更新できる");
        window
    }

    fn current(cx: &mut TestAppContext) -> Arc<Theme> {
        cx.update(|cx| cx.md3_theme().clone())
    }

    fn resolved(settings: ThemeSettings, system: Brightness) -> Theme {
        Theme::resolve(settings, system).expect("テストの設定は解決できる")
    }

    #[gpui::test]
    fn system_の間は_os_の外観に追従する(cx: &mut TestAppContext) {
        open_window(cx);
        let light = resolved(ThemeSettings::default(), Brightness::Light);
        assert_eq!(*current(cx), light);
        cx.update(|cx| follow_system(Brightness::Dark, cx));
        let dark = resolved(ThemeSettings::default(), Brightness::Dark);
        assert_eq!(*current(cx), dark);
    }

    #[gpui::test]
    fn 初期化したウィンドウの外観に合わせる(cx: &mut TestAppContext) {
        // テスト用のウィンドウの外観はライトなので、先にテーマをダークにしておく
        cx.update(|cx| {
            init_global(cx);
            follow_system(Brightness::Dark, cx);
        });
        open_window(cx);
        let light = resolved(ThemeSettings::default(), Brightness::Light);
        assert_eq!(*current(cx), light);
    }

    #[gpui::test]
    fn os_の外観の変化は明示指定を上書きしない(cx: &mut TestAppContext) {
        open_window(cx);
        cx.update(|cx| cx.update_md3_theme(|settings| settings.appearance = AppearanceMode::Light))
            .expect("明暗だけを変えた設定は解決できる");
        cx.update(|cx| follow_system(Brightness::Dark, cx));
        let settings = ThemeSettings {
            appearance: AppearanceMode::Light,
            ..ThemeSettings::default()
        };
        assert_eq!(*current(cx), resolved(settings, Brightness::Dark));
        assert_eq!(
            current(cx).color(ColorRole::Surface),
            resolved(ThemeSettings::default(), Brightness::Light).color(ColorRole::Surface)
        );
    }

    #[gpui::test]
    fn 外観が変わっても差し替えた色を保つ(cx: &mut TestAppContext) {
        open_window(cx);
        let red = Argb::from_rgb(0xff, 0, 0);
        cx.update(|cx| {
            cx.update_md3_theme(|settings| settings.color.dark.set(ColorRole::Primary, red))
        })
        .expect("色を差し替えた設定は解決できる");
        cx.update(|cx| follow_system(Brightness::Dark, cx));
        assert_eq!(
            current(cx).color(ColorRole::Primary),
            crate::theme::to_hsla(red)
        );
    }

    #[gpui::test]
    fn 解決できない設定はテーマを変えない(cx: &mut TestAppContext) {
        open_window(cx);
        let before = current(cx);
        let result = cx.update(|cx| {
            cx.update_md3_theme(|settings| {
                settings.color.variant = Variant::Cmf;
                settings.color.spec_version = SpecVersion::Spec2021;
            })
        });
        assert!(result.is_err());
        assert_eq!(current(cx), before);
    }
}
