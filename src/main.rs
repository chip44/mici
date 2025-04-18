use image::GenericImageView;
use ksni::blocking::TrayMethods;
use libspa::utils::dict::DictRef;
use pipewire::{context::Context, keys::MEDIA_CLASS, main_loop::MainLoop};
use std::{cell::RefCell, error::Error, sync::LazyLock};

struct PipeWireManager {
    nodes: Vec<u32>,
    tray_handle: Option<ksni::blocking::Handle<MyTray>>,
}

impl PipeWireManager {
    fn new() -> Self {
        Self {
            nodes: Vec::new(),
            tray_handle: None,
        }
    }

    fn on_global_event(&mut self, id: u32, props: Option<&DictRef>) {
        let Some(props) = props else { return };

        let Some(media_class) = props.get(&MEDIA_CLASS) else {
            return;
        };
        if media_class != "Stream/Input/Audio" {
            return;
        }

        if self.nodes.len() == 0 {
            println!("initializing tray");
            assert!(self.tray_handle.is_none());
            let tray = MyTray {};
            self.tray_handle = Some(tray.spawn().unwrap());
        }

        println!("id: {id:?}");
        println!("props: {props:?}");
        println!("media_class: {media_class:?}");
        self.nodes.push(id);
    }

    fn on_global_remove_event(&mut self, id: u32) {
        println!("id: {id:?}");

        self.nodes.retain(|&x| x != id);
        println!("current nodes: {:?}", self.nodes);

        if self.nodes.len() == 0 {
            if let Some(handle) = self.tray_handle.take() {
                println!("closing tray");
                handle.shutdown();
            }
        }
    }
}

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
struct MyTray {}

impl ksni::Tray for MyTray {
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

fn main() -> Result<(), Box<dyn Error>> {
    let mainloop = MainLoop::new(None)?;
    let context = Context::new(&mainloop)?;
    let core = context.connect(None)?;
    let registry = core.get_registry()?;

    let pipewire_manager = PipeWireManager::new();
    // create the manager on the heap and leak it to ensure 'static lifetime
    // this is a deliberate choice to work with callbacks that require 'static
    let pipewire_manager = Box::leak(Box::new(RefCell::new(pipewire_manager)));

    let _listener = registry
        .add_listener_local()
        .global(|global| {
            pipewire_manager
                .borrow_mut()
                .on_global_event(global.id, global.props)
        })
        .global_remove(|global_id| {
            pipewire_manager
                .borrow_mut()
                .on_global_remove_event(global_id)
        })
        .register();

    mainloop.run();

    Ok(())
}
