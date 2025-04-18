use image::GenericImageView;
use std::sync::LazyLock;

static TRAY_ICON: LazyLock<ksni::Icon> = LazyLock::new(|| {
    let img =
        image::load_from_memory_with_format(include_bytes!("../icon.png"), image::ImageFormat::Png)
            .expect("valid image");
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
pub struct SystemTray {}

impl ksni::Tray for SystemTray {
    fn id(&self) -> String {
        env!("CARGO_PKG_NAME").into()
    }

    fn title(&self) -> String {
        self.id().into() // TODO: show app names that use the microphone
    }

    fn icon_pixmap(&self) -> Vec<ksni::Icon> {
        vec![(*TRAY_ICON).clone()] // TODO: don't clone?
    }
}
