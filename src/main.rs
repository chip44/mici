use std::error::Error;

use libspa::utils::dict::DictRef;
use pipewire::{context::Context, keys::MEDIA_CLASS, main_loop::MainLoop};

fn on_global_event(id: u32, props: Option<&DictRef>) {
    let props = match props {
        Some(p) => p,
        None => return,
    };

    let media_class = match props.get(&MEDIA_CLASS) {
        Some(m) => m,
        None => return,
    };
    if media_class != "Stream/Input/Audio" {
        return;
    }

    println!("id: {:?}", id);
    println!("props: {:?}", props);
    println!("media_class: {:?}", media_class);

    println!("TODO: turn on indicator");
}

fn on_global_remove_event(id: u32) {
    println!("id: {:?}", id);
    println!("TODO: turn off indicator");
}

fn main() -> Result<(), Box<dyn Error>> {
    let mainloop = MainLoop::new(None)?;
    let context = Context::new(&mainloop)?;
    let core = context.connect(None)?;
    let registry = core.get_registry()?;

    let _listener = registry
        .add_listener_local()
        .global(|global| on_global_event(global.id, global.props))
        .global_remove(|global_id| on_global_remove_event(global_id))
        .register();

    mainloop.run();

    Ok(())
}
