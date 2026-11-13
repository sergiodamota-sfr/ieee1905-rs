use rbus_core::{RBusEvent, RBusEventRef, RBusHandle};
use std::ffi::CString;

fn on_event(event_data: RBusEventRef<'_>) {
    println!(
        "{:p}: {:?}: {:?}",
        event_data.handle().to_raw(),
        event_data.name(),
        event_data.kind()
    );
    if let Some(data) = event_data.data() {
        for property in data.get_properties().iter() {
            println!("  {:?} = {:?}", property.get_name(), property.get_value());
        }
    }
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let event_name = std::env::args()
        .nth(1)
        .ok_or("usage: cargo run -p rbus-core --example subscribe_event -- EVENT_NAME")?;
    let event_name = CString::new(event_name)?;
    let bus = RBusHandle::open(c"rbus-event-listener")?;
    let subscription = RBusEvent::subscribe(&bus, &event_name, on_event)?;

    println!("Subscribed. Press Enter to unsubscribe and exit.");
    std::io::stdin().read_line(&mut String::new())?;
    drop(subscription);
    Ok(())
}
