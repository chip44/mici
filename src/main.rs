use ksni::blocking::TrayMethods;
use libspa::utils::dict::DictRef;
use pipewire::{
    context::Context,
    keys::{APP_NAME, MEDIA_CLASS, NODE_NAME},
    main_loop::MainLoop,
};
use std::{cell::RefCell, collections::HashMap, error::Error};

mod tray;
use tray::SystemTray;

struct PipeWireManager {
    tray_handle: Option<ksni::blocking::Handle<SystemTray>>,
}

impl PipeWireManager {
    const fn new() -> Self {
        Self { tray_handle: None }
    }

    fn on_global_event(&mut self, id: u32, props: Option<&DictRef>) {
        let Some(props) = props else { return };

        if props.get(&MEDIA_CLASS) != Some("Stream/Input/Audio") {
            return;
        }

        let unknown_name_fallback = format!("<unknown app {id}>");
        let node_name = props
            .get(&NODE_NAME)
            .or_else(|| props.get(&APP_NAME))
            .unwrap_or(&unknown_name_fallback)
            .to_string();

        // create or update the tray
        if let Some(handle) = self.tray_handle.as_mut() {
            handle.update(|tray: &mut SystemTray| {
                assert!(
                    !tray.nodes.contains_key(&id),
                    "node with id {id} already exists",
                );
                tray.nodes.insert(id, node_name)
            });
        } else {
            // initialize the tray with 1 node
            let mut nodes = HashMap::with_capacity(1);
            nodes.insert(id, node_name);
            let tray = SystemTray { nodes };
            self.tray_handle = Some(tray.spawn().expect("error spawning system tray"));
        }
    }

    fn on_global_remove_event(&mut self, id: u32) {
        let mut should_shutdown = false;

        if let Some(handle) = self.tray_handle.as_mut() {
            handle.update(|tray: &mut SystemTray| {
                // if we're about to remove the last node, return early to shutdown the tray
                if tray.nodes.len() == 1 {
                    should_shutdown = true;
                    return;
                }

                tray.nodes.remove(&id);
            });
        }

        if should_shutdown {
            if let Some(handle) = self.tray_handle.take() {
                handle.shutdown();
            }
        }
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
                .on_global_event(global.id, global.props);
        })
        .global_remove(|global_id| {
            pipewire_manager
                .borrow_mut()
                .on_global_remove_event(global_id);
        })
        .register();

    mainloop.run();

    Ok(())
}
