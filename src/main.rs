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
    nodes: Vec<u32>,
    tray_handle: Option<ksni::blocking::Handle<SystemTray>>,
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

        let unknown_name_fallback = format!("<unknown app {id}>");
        let node_name = props
            .get(&NODE_NAME)
            .or_else(|| props.get(&APP_NAME))
            .or_else(|| Some(&unknown_name_fallback))
            .unwrap();

        if self.nodes.len() == 0 {
            println!("initializing tray");
            assert!(self.tray_handle.is_none());
            let mut nodes_names = HashMap::new();
            nodes_names.insert(id, node_name.to_string());
            let tray = SystemTray { nodes_names };
            self.tray_handle = Some(tray.spawn().unwrap());
        } else {
            self.tray_handle
                .as_mut()
                .unwrap()
                .update(|tray: &mut SystemTray| {
                    assert!(!tray.nodes_names.contains_key(&id));
                    tray.nodes_names.insert(id, node_name.to_string())
                });
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
        } else {
            self.tray_handle
                .as_mut()
                .unwrap()
                .update(|tray: &mut SystemTray| {
                    tray.nodes_names.remove(&id);
                });
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
