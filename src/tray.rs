use image::{GenericImageView, ImageFormat};
use std::{collections::HashMap, sync::LazyLock};

static TRAY_ICON: LazyLock<ksni::Icon> = LazyLock::new(|| {
    let img = image::load_from_memory_with_format(include_bytes!("../icon.png"), ImageFormat::Png)
        .unwrap();
    let (width, height) = img.dimensions();
    let mut data = img.into_rgba8().into_vec();
    assert_eq!(data.len() % 4, 0);
    for pixel in data.chunks_exact_mut(4) {
        pixel.rotate_right(1) // rgba to argb
    }
    ksni::Icon {
        width: width as i32,
        height: height as i32,
        data,
    }
});

#[derive(Debug)]
pub struct SystemTray {
    pub nodes_names: HashMap<u32, String>,
}

impl ksni::Tray for SystemTray {
    fn id(&self) -> String {
        env!("CARGO_PKG_NAME").into()
    }

    fn title(&self) -> String {
        let total_nodes = self.nodes_names.len();
        let mut title = match total_nodes {
            0 => unreachable!(),
            1 => "1 app is using your microphone:".to_string(),
            _ => format!("{total_nodes} apps are using your microphone:"),
        };

        title.reserve(
            self.nodes_names
                .values()
                .map(|v| v.len() + 2)
                .sum::<usize>(),
        );
        for value in self.nodes_names.values() {
            title.push_str("\n");
            title.push_str(value);
        }

        title
    }

    fn icon_pixmap(&self) -> Vec<ksni::Icon> {
        vec![TRAY_ICON.clone()]
    }
}
