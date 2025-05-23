use eframe::egui;
use futures_lite::StreamExt;
use lapin::{
    options::{BasicConsumeOptions, QueueBindOptions, QueueDeclareOptions},
    types::{AMQPValue, FieldTable},
    uri::AMQPUri,
    Channel, Connection, ConnectionProperties, Consumer, Queue,
};

use serde_json::json;
use tokio::sync::mpsc;
use uuid::Uuid;

/// Carries commands from the UI to the rabbit connection manager
pub enum ConnectionCommand {
    // Used to pass the Egui repaint signaller
    Disconnect,
    Connect(AMQPUri, egui::Context),
    Bind {
        exchange: String,
        routing_key: String,
        options: QueueBindOptions,
        arguments: FieldTable,
    },
    Unbind(Binding),
}

#[derive(Clone, Debug)]
pub struct Binding {
    pub id: Uuid,
    pub exchange: String,
    pub routing_key: String,
    pub arguments: FieldTable,
}

/// Carries messages from the connection/connection manager to UI.
/// This comprises both rabbit data payloads and status changes.
pub enum ConnectionUpdate {
    Connecting,
    Connected,
    Disconnected,
    TextDelivery {
        headers: String,
        content: String,
        content_type: Option<String>,
    },
    /// A binary message, with content redacted
    BinaryDelivery {
        headers: String,
        content_type: Option<String>,
    },
    Bound(Binding),
    Unbound(Binding),
}

pub struct ConnectionManager {
    pub tx: mpsc::UnboundedSender<ConnectionCommand>,
    pub rx: mpsc::UnboundedReceiver<ConnectionUpdate>,
}

impl ConnectionManager {
    pub fn new() -> Self {
        let (tx, rxc) = mpsc::unbounded_channel();
        let (txc, rx) = mpsc::unbounded_channel();

        let _connection_manager_thread = std::thread::spawn(|| {
            let rt = tokio::runtime::Builder::new_current_thread()
                .build()
                .expect("Failed to instantiate async runtime");
            if let Err(e) = rt.block_on(connection_manager_task(rxc, txc)) {
                log::error!("Connection manager error: {}", e);
                std::process::exit(1);
            }
        });

        Self { tx, rx }
    }

    pub fn unbind(&self, binding: Binding) {
        self.tx
            .send(ConnectionCommand::Unbind(binding))
            .expect("Internal channel closed");
    }

    pub fn bind(&self, binding: Binding) {
        self.tx
            .send(crate::rabbit::ConnectionCommand::Bind {
                exchange: binding.exchange,
                routing_key: binding.routing_key,
                options: QueueBindOptions::default(),
                arguments: binding.arguments,
            })
            .expect("Internal channel closed");
    }
}

pub(crate) fn field_table_to_json(field_table: &FieldTable) -> serde_json::Value {
    let mut json_map = serde_json::Map::new();

    for (key, value) in field_table.inner().iter() {
        let json_value = match value {
            AMQPValue::Void => json!(null),
            AMQPValue::ShortShortInt(i) => json!(i),
            AMQPValue::ShortShortUInt(u) => json!(u),
            AMQPValue::ShortInt(i) => json!(i),
            AMQPValue::ShortUInt(u) => json!(u),
            AMQPValue::LongInt(i) => json!(i),
            AMQPValue::LongUInt(u) => json!(u),
            AMQPValue::LongLongInt(i) => json!(i),
            AMQPValue::LongString(ls) => json!(ls.to_string()),
            AMQPValue::ShortString(ss) => json!(ss.to_string()),
            AMQPValue::Boolean(b) => json!(b),
            AMQPValue::Float(f) => json!(f),
            AMQPValue::Double(d) => json!(d),
            AMQPValue::DecimalValue(d) => json!({"scale": d.scale, "value": d.value}),
            AMQPValue::Timestamp(ts) => json!(ts), // TODO consider formatting timestamps
            AMQPValue::ByteArray(bytes) => json!(bytes),
            AMQPValue::FieldArray(arr) => {
                json!(arr
                    .as_slice()
                    .iter()
                    .map(|v| format!("{:?}", v))
                    .collect::<Vec<_>>())
            }
            AMQPValue::FieldTable(ft) => field_table_to_json(ft),
        };
        json_map.insert(key.to_string(), json_value);
    }

    serde_json::Value::Object(json_map)
}

async fn connection_manager_task(
    mut rx: mpsc::UnboundedReceiver<ConnectionCommand>,
    tx: mpsc::UnboundedSender<ConnectionUpdate>,
) -> anyhow::Result<()> {
    let options =
        ConnectionProperties::default().with_executor(tokio_executor_trait::Tokio::current());

    let queue_declare_options = QueueDeclareOptions {
        passive: false,
        durable: false,
        auto_delete: true,
        exclusive: true,
        nowait: false,
    };

    // Need to restructure into an outer loop that waits on connection requests, and an inner loop that consumes from the queue
    // and services other messages.
    'not_connected: loop {
        let mut consumer: Consumer;
        let connection: Connection;
        let channel: Channel;
        let queue: Queue;
        let egui_ctx: egui::Context;

        match rx.recv().await {
            Some(ConnectionCommand::Connect(uri, ctx)) => {
                let _ = tx.send(ConnectionUpdate::Connecting);
                connection = Connection::connect_uri(uri, options.clone()).await?;
                channel = connection.create_channel().await?;
                queue = channel
                    .queue_declare("", queue_declare_options.clone(), FieldTable::default())
                    .await?;
                let opts = BasicConsumeOptions {
                    exclusive: true,
                    no_ack: true,
                    ..Default::default()
                };
                consumer = channel
                    .basic_consume(queue.name().as_str(), "", opts, FieldTable::default())
                    .await?;
                egui_ctx = ctx;

                let _ = tx.send(ConnectionUpdate::Connected);
            }
            None => {
                log::debug!("Connection manager incoming channel closed; assume caller exited.");
                return Ok(());
            }
            _invalid => {
                log::warn!("Received a command other than 'connect' whilst not connected.");
                continue;
            }
        }

        '_connected: loop {
            egui_ctx.request_repaint();
            tokio::select! {
                r = rx.recv() => match r {
                    Some(ConnectionCommand::Disconnect) => {
                        let _ = tx.send(ConnectionUpdate::Disconnected);
                        continue 'not_connected;
                    }
                    Some(ConnectionCommand::Connect(_, _)) => {
                        log::warn!("Ignoring a 'connect' command whilst already connected");
                    }
                    Some(ConnectionCommand::Bind {
                        exchange,
                        routing_key,
                        options,
                        arguments,
                    }) => {
                            channel.queue_bind(
                                queue.name().as_str(),
                                &exchange,
                                &routing_key,
                                options.clone(),
                                arguments.clone(),
                            )
                            .await?;
                        tx.send(ConnectionUpdate::Bound(Binding {  exchange, routing_key, arguments, id: Uuid::new_v4() })).expect("Internal channel closed");
                    }
                    Some(ConnectionCommand::Unbind(binding) ) => {
                            channel.queue_unbind(queue.name().as_str(), &binding.exchange, &binding.routing_key, binding.arguments.clone())
                                .await?;
                            tx.send(ConnectionUpdate::Unbound(binding)).expect("Internal connection closed")
                    }
                    None => {
                        log::debug!(
                            "Connection manager incoming channel closed; assume caller exited."
                        );
                        return Ok(());
                    }
                },
                r = consumer.next() => match r{
                    None => {
                        let _ = tx.send(ConnectionUpdate::Disconnected);
                        continue 'not_connected;
                    }
                    Some(Ok(msg)) => {
                        let headers = match msg.properties.headers() {
                            Some(headers) => field_table_to_json(headers).to_string(),
                            None => serde_json::json!({}).to_string(),
                        };

                        // TODO think about leveraging the content-type here if it's available.
                        let content_type: Option<String> = msg.properties.content_type().as_ref().map(|c|c.to_string());
                        let update = match String::from_utf8(msg.data) {
                            Ok(content) => ConnectionUpdate::TextDelivery { headers, content, content_type },
                            Err(_) => ConnectionUpdate::BinaryDelivery{ headers, content_type},
                        };
                        tx.send(update).expect("Internal channel closed");
                    }
                    Some(Err(_)) => {
                        let _ = tx.send(ConnectionUpdate::Disconnected).expect("Internal channel closed");
                        continue 'not_connected;
                    }
                }
            }
        }
    }
}
