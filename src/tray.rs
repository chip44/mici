use image::{GenericImageView, ImageFormat};
use std::{collections::HashMap, sync::LazyLock};

static TRAY_ICON: LazyLock<ksni::Icon> = LazyLock::new(|| {
    let img = image::load_from_memory_with_format(include_bytes!("../icon.png"), ImageFormat::Png)
        .expect("invalid image");
    let (width, height) = img.dimensions();
    let mut data = img.into_rgba8().into_vec();
    assert_eq!(data.len() % 4, 0);
    for pixel in data.chunks_exact_mut(4) {
        pixel.rotate_right(1); // rgba to argb
    }
    ksni::Icon {
        // why tf is the size i32??
        width: i32::try_from(width).expect("invalid image width"),
        height: i32::try_from(height).expect("invalid image height"),
        data,
    }
});

#[derive(Debug)]
pub struct SystemTray {
    pub nodes: HashMap<u32, String>,
}

impl ksni::Tray for SystemTray {
    fn id(&self) -> String {
        env!("CARGO_PKG_NAME").into()
    }

    fn title(&self) -> String {
        let mut title = match self.nodes.len() {
            0 => unreachable!(),
            1 => "1 app is using your microphone:".to_string(),
            n => format!("{n} apps are using your microphone:"),
        };

        title.reserve(
            self.nodes
                .values()
                .map(|v| v.len() + 1 /* 1 extra for '\n' */)
                .sum::<usize>(),
        );
        for value in self.nodes.values() {
            title.push('\n');
            title.push_str(value);
        }

        title
    }

    fn icon_pixmap(&self) -> Vec<ksni::Icon> {
        vec![TRAY_ICON.clone()]
    }
}
