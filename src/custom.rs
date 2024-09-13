use iced::{widget::button, Theme};

pub struct CustomButtonStyle {
    pub color: iced::Color,
}

impl button::StyleSheet for CustomButtonStyle {
    type Style = Theme;

    fn active(&self, _: &Self::Style) -> button::Appearance {
        let mut appearance = button::Appearance {
            background: Some(iced::Background::Color(self.color)),
            ..button::Appearance::default()
        };

        appearance.border.radius = 10.0.into();

        appearance
    }
}
