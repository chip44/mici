use std::{cell::RefCell, error::Error};

use libspa::utils::dict::DictRef;
use pipewire::{context::Context, keys::MEDIA_CLASS, main_loop::MainLoop};

struct PipeWireManager {
    nodes: Vec<u32>,
}

impl PipeWireManager {
    fn new() -> PipeWireManager {
        PipeWireManager { nodes: Vec::new() }
    }

    fn on_global_event(&mut self, id: u32, props: Option<&DictRef>) {
        let Some(props) = props else { return };

        let Some(media_class) = props.get(&MEDIA_CLASS) else {
            return;
        };
        if media_class != "Stream/Input/Audio" {
            return;
        }

        println!("id: {id:?}");
        println!("props: {props:?}");
        println!("media_class: {media_class:?}");
        self.nodes.push(id);

        println!("TODO: turn on indicator");
    }

    fn on_global_remove_event(&mut self, id: u32) {
        println!("id: {id:?}");

        self.nodes.retain(|&x| x != id);
        println!("current nodes: {:?}", self.nodes);

        println!("TODO: turn off indicator");
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
