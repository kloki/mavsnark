use std::io;

use mavlink::{MavConnection, common::MavMessage};

pub fn connect(uri: &str) -> io::Result<Box<dyn MavConnection<MavMessage> + Send + Sync>> {
    let mut connection = mavlink::connect::<MavMessage>(uri)
        .map_err(|e| io::Error::other(format!("{uri}: {e}")))?;

    connection.set_protocol_version(mavlink::MavlinkVersion::V2);

    Ok(Box::new(connection))
}
