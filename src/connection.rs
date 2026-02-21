use mavlink::{MavConnection, common::MavMessage};

pub fn connect(uri: &str) -> Box<dyn MavConnection<MavMessage> + Send + Sync> {
    let mut connection = mavlink::connect::<MavMessage>(uri)
        .unwrap_or_else(|e| panic!("failed to connect to {uri}: {e}"));

    connection.set_protocol_version(mavlink::MavlinkVersion::V2);

    Box::new(connection)
}
