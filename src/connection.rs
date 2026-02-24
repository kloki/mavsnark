use std::{io, sync::Arc};

use mavlink::common::{
    HEARTBEAT_DATA, MavAutopilot, MavMessage, MavModeFlag, MavState, MavType,
};
use mavlink::{MavConnection, MavHeader};

pub fn connect(uri: &str) -> io::Result<Arc<dyn MavConnection<MavMessage> + Send + Sync>> {
    let mut connection =
        mavlink::connect::<MavMessage>(uri).map_err(|e| io::Error::other(format!("{uri}: {e}")))?;

    connection.set_protocol_version(mavlink::MavlinkVersion::V2);

    Ok(Arc::new(connection))
}

pub fn spawn_heartbeat(
    connection: &Arc<dyn MavConnection<MavMessage> + Send + Sync>,
    system_id: u8,
) {
    let conn = Arc::clone(connection);
    tokio::spawn(async move {
        let header = MavHeader {
            system_id,
            component_id: 0,
            sequence: 0,
        };
        let heartbeat = MavMessage::HEARTBEAT(HEARTBEAT_DATA {
            custom_mode: 0,
            mavtype: MavType::MAV_TYPE_GCS,
            autopilot: MavAutopilot::MAV_AUTOPILOT_INVALID,
            base_mode: MavModeFlag::empty(),
            system_status: MavState::MAV_STATE_ACTIVE,
            mavlink_version: 3,
        });
        let mut interval = tokio::time::interval(std::time::Duration::from_secs(1));
        loop {
            interval.tick().await;
            if conn.send(&header, &heartbeat).is_err() {
                break;
            }
        }
    });
}
