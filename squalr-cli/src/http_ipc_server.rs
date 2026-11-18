use crate::http_ipc_access::HttpIpcAccess;
use anyhow::{Context, Result, anyhow};
use serde::{Deserialize, Serialize};
use squalr_engine::engine_bindings::executable_command_privileged::execute_privileged_command;
use squalr_engine::engine_privileged_state::EnginePrivilegedState;
use squalr_engine_api::{
    commands::{privileged_command::PrivilegedCommand, privileged_command_result::PrivilegedCommandResult},
    engine::engine_event_envelope::EngineEventEnvelope,
};
use std::{
    collections::{HashMap, VecDeque},
    io::{BufRead, BufReader, Read, Write},
    net::{TcpListener, TcpStream},
    sync::{Arc, Mutex},
    thread,
    time::Duration,
};

const MAX_HEADER_BYTES: u64 = 16 * 1024;
const MAX_ACTIVE_CONNECTIONS: usize = 32;
const MAX_REQUEST_BODY_BYTES: usize = 1024 * 1024;
const MAX_RETAINED_EVENTS: usize = 4096;

pub struct HttpIpcServer {
    bind_address: String,
    engine_privileged_state: Arc<EnginePrivilegedState>,
    event_store: Arc<Mutex<VecDeque<HttpIpcEvent>>>,
    command_execution_lock: Arc<Mutex<()>>,
    access: Arc<HttpIpcAccess>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
struct HttpIpcEvent {
    event_id: u64,
    event: EngineEventEnvelope,
}

#[derive(Debug)]
struct HttpRequest {
    method: String,
    path: String,
    body: Vec<u8>,
    authorization: Option<String>,
}

#[derive(Debug, Serialize)]
struct HttpErrorResponse {
    success: bool,
    error: String,
}

#[derive(Debug, Serialize)]
struct HealthResponse {
    success: bool,
    service: &'static str,
    protocol_version: u32,
    capabilities: [&'static str; 2],
    project_owner: &'static str,
}

#[derive(Debug, Serialize)]
struct CommandResponse {
    success: bool,
    result: PrivilegedCommandResult,
}

#[derive(Debug, Serialize)]
struct EventsResponse {
    success: bool,
    events: Vec<HttpIpcEvent>,
    latest_event_id: u64,
    cursor_expired: bool,
}

impl HttpIpcServer {
    pub fn new(
        bind_address: String,
        engine_privileged_state: Arc<EnginePrivilegedState>,
    ) -> Result<Self> {
        Ok(Self {
            bind_address,
            engine_privileged_state,
            event_store: Arc::new(Mutex::new(VecDeque::new())),
            command_execution_lock: Arc::new(Mutex::new(())),
            access: Arc::new(HttpIpcAccess::from_environment()?),
        })
    }

    pub fn run(&self) -> Result<()> {
        let address = HttpIpcAccess::loopback_address(&self.bind_address)?;
        let listener = TcpListener::bind(address).with_context(|| format!("Failed to bind HTTP IPC server to '{}'.", self.bind_address))?;
        log::info!("HTTP IPC server listening on {}.", self.bind_address);

        self.spawn_event_collector();
        let (slots, used_slots) = crossbeam_channel::bounded(MAX_ACTIVE_CONNECTIONS);
        for incoming_connection in listener.incoming() {
            match incoming_connection {
                Ok(stream) => {
                    if slots.try_send(()).is_err() {
                        continue;
                    }
                    let used_slots = used_slots.clone();
                    let engine_privileged_state = self.engine_privileged_state.clone();
                    let event_store = self.event_store.clone();
                    let command_execution_lock = self.command_execution_lock.clone();
                    let access = self.access.clone();

                    thread::spawn(move || {
                        if let Err(error) = Self::handle_connection(engine_privileged_state, event_store, command_execution_lock, access, stream) {
                            log::warn!("HTTP IPC request failed: {}", error);
                        }
                        let _ = used_slots.try_recv();
                    });
                }
                Err(error) => {
                    log::warn!("HTTP IPC connection accept failed: {}.", error);
                }
            }
        }

        Ok(())
    }

    fn spawn_event_collector(&self) {
        let event_receiver = match self.engine_privileged_state.subscribe_to_engine_events() {
            Ok(event_receiver) => event_receiver,
            Err(error) => {
                log::warn!("HTTP IPC server could not subscribe to engine events: {}.", error);
                return;
            }
        };
        let event_store = self.event_store.clone();

        thread::spawn(move || {
            while let Ok(event) = event_receiver.recv() {
                match event_store.lock() {
                    Ok(mut event_store) => {
                        let event_id = event_store.back().map_or(1, |last| last.event_id + 1);
                        event_store.push_back(HttpIpcEvent { event_id, event });
                        while event_store.len() > MAX_RETAINED_EVENTS {
                            event_store.pop_front();
                        }
                    }
                    Err(error) => {
                        log::error!("HTTP IPC event store lock failed: {}.", error);
                        break;
                    }
                }
            }
        });
    }

    fn handle_connection(
        engine_privileged_state: Arc<EnginePrivilegedState>,
        event_store: Arc<Mutex<VecDeque<HttpIpcEvent>>>,
        command_execution_lock: Arc<Mutex<()>>,
        access: Arc<HttpIpcAccess>,
        mut stream: TcpStream,
    ) -> Result<()> {
        stream
            .set_read_timeout(Some(Duration::from_secs(30)))
            .context("Failed to set HTTP IPC read timeout.")?;
        stream.set_write_timeout(Some(Duration::from_secs(30)))?;
        let request = match Self::read_request(&mut stream) {
            Ok(request) => request,
            Err(error) => return Self::write_error(&mut stream, 400, error.to_string()),
        };

        if !access.authorizes(request.authorization.as_deref()) {
            return Self::write_error(&mut stream, 401, "Invalid controller credential.");
        }

        match (request.method.as_str(), request.path_without_query().as_str()) {
            ("GET", "/health") => Self::write_json(
                &mut stream,
                200,
                &HealthResponse {
                    success: true,
                    service: "squalr-http-ipc",
                    protocol_version: 1,
                    capabilities: ["privileged_commands", "event_polling"],
                    project_owner: "client",
                },
            ),
            ("POST", "/command") => {
                if let Err(error) = Self::handle_command(engine_privileged_state, command_execution_lock, &mut stream, request) {
                    Self::write_error(&mut stream, 400, error.to_string())
                } else {
                    Ok(())
                }
            }
            ("GET", "/events") => {
                if let Err(error) = Self::handle_events(event_store, &mut stream, request) {
                    Self::write_error(&mut stream, 400, error.to_string())
                } else {
                    Ok(())
                }
            }
            _ => Self::write_error(&mut stream, 404, "Unknown HTTP IPC endpoint."),
        }
    }

    fn handle_command(
        engine_privileged_state: Arc<EnginePrivilegedState>,
        command_execution_lock: Arc<Mutex<()>>,
        stream: &mut TcpStream,
        request: HttpRequest,
    ) -> Result<()> {
        let privileged_command: PrivilegedCommand =
            serde_json::from_slice(&request.body).map_err(|error| anyhow!("Invalid PrivilegedCommand JSON: {}", error))?;
        let generation = engine_privileged_state.get_process_manager().generation();
        let command_execution_guard = command_execution_lock
            .lock()
            .map_err(|error| anyhow!("Failed to lock HTTP IPC command executor: {}", error))?;
        let result = execute_privileged_command(&engine_privileged_state, privileged_command, Some(generation));
        drop(command_execution_guard);
        let success = result
            .get_privileged_command_response()
            .failure_message()
            .is_none();
        Self::write_json(stream, 200, &CommandResponse { success, result })
    }

    fn handle_events(
        event_store: Arc<Mutex<VecDeque<HttpIpcEvent>>>,
        stream: &mut TcpStream,
        request: HttpRequest,
    ) -> Result<()> {
        let after_event_id = request
            .query_parameters()
            .get("after")
            .map(|cursor| cursor.parse::<u64>())
            .transpose()?
            .unwrap_or(0);
        let batch = {
            let store = event_store
                .lock()
                .map_err(|error| anyhow!("Failed to lock event store: {error}"))?;
            Self::event_batch(&store, after_event_id)
        };
        Self::write_json(stream, 200, &batch)
    }

    fn event_batch(
        store: &VecDeque<HttpIpcEvent>,
        after_event_id: u64,
    ) -> EventsResponse {
        let latest_event_id = store.back().map_or(0, |entry| entry.event_id);
        let cursor_expired = after_event_id > latest_event_id
            || store
                .front()
                .is_some_and(|oldest| oldest.event_id.saturating_sub(1) > after_event_id);
        EventsResponse {
            success: true,
            events: store
                .iter()
                .filter(|entry| entry.event_id > after_event_id)
                .cloned()
                .collect(),
            latest_event_id,
            cursor_expired,
        }
    }

    fn read_request(stream: &mut TcpStream) -> Result<HttpRequest> {
        let mut reader = BufReader::new(stream);
        let mut headers_reader = (&mut reader).take(MAX_HEADER_BYTES);
        let mut request_line = String::new();
        headers_reader
            .read_line(&mut request_line)
            .context("Failed to read HTTP IPC request line.")?;
        let request_line_parts = request_line
            .split_whitespace()
            .map(String::from)
            .collect::<Vec<_>>();

        if request_line_parts.len() < 2 {
            return Err(anyhow!("Invalid HTTP IPC request line."));
        }

        let mut headers = HashMap::new();
        loop {
            let mut header_line = String::new();
            headers_reader
                .read_line(&mut header_line)
                .context("Failed to read HTTP IPC request header.")?;
            if !header_line.ends_with('\n') {
                return Err(anyhow!("Incomplete or oversized HTTP headers."));
            }
            let trimmed_header_line = header_line.trim_end_matches(['\r', '\n']);

            if trimmed_header_line.is_empty() {
                break;
            }

            if let Some((header_name, header_value)) = trimmed_header_line.split_once(':') {
                if headers
                    .insert(header_name.trim().to_ascii_lowercase(), header_value.trim().to_string())
                    .is_some()
                {
                    return Err(anyhow!("Duplicate HTTP header."));
                }
            }
        }

        let content_length = headers
            .get("content-length")
            .map(|content_length| content_length.parse::<usize>())
            .transpose()?
            .unwrap_or(0);

        if content_length > MAX_REQUEST_BODY_BYTES {
            return Err(anyhow!(
                "HTTP IPC request body is too large: {} bytes, max {} bytes.",
                content_length,
                MAX_REQUEST_BODY_BYTES
            ));
        }

        let mut body = vec![0u8; content_length];
        if content_length > 0 {
            reader
                .read_exact(&mut body)
                .context("Failed to read HTTP IPC request body.")?;
        }

        Ok(HttpRequest {
            authorization: headers.remove("authorization"),
            method: request_line_parts[0].clone(),
            path: request_line_parts[1].clone(),
            body,
        })
    }

    fn write_json<T: Serialize>(
        stream: &mut TcpStream,
        status_code: u16,
        body: &T,
    ) -> Result<()> {
        let body = serde_json::to_vec(body).context("Failed to serialize HTTP IPC response.")?;
        let reason = Self::status_reason(status_code);
        let response_header = format!(
            "HTTP/1.1 {} {}\r\nContent-Type: application/json\r\nContent-Length: {}\r\nConnection: close\r\n\r\n",
            status_code,
            reason,
            body.len()
        );

        stream
            .write_all(response_header.as_bytes())
            .context("Failed to write HTTP IPC response header.")?;
        stream
            .write_all(&body)
            .context("Failed to write HTTP IPC response body.")?;
        stream.flush().context("Failed to flush HTTP IPC response.")
    }

    fn write_error(
        stream: &mut TcpStream,
        status_code: u16,
        error: impl Into<String>,
    ) -> Result<()> {
        Self::write_json(
            stream,
            status_code,
            &HttpErrorResponse {
                success: false,
                error: error.into(),
            },
        )
    }

    fn status_reason(status_code: u16) -> &'static str {
        match status_code {
            200 => "OK",
            401 => "Unauthorized",
            400 => "Bad Request",
            404 => "Not Found",
            500 => "Internal Server Error",
            _ => "OK",
        }
    }
}

impl HttpRequest {
    fn path_without_query(&self) -> String {
        self.path
            .split_once('?')
            .map(|(path, _)| path.to_string())
            .unwrap_or_else(|| self.path.clone())
    }

    fn query_parameters(&self) -> HashMap<String, String> {
        let Some((_, query_string)) = self.path.split_once('?') else {
            return HashMap::new();
        };

        query_string
            .split('&')
            .filter_map(|parameter| parameter.split_once('='))
            .map(|(key, value)| (key.to_string(), value.to_string()))
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use squalr_engine_api::events::{engine_event::EngineEventRequest, plugins::changed::plugins_changed_event::PluginsChangedEvent};

    #[test]
    fn event_cursor_reports_eviction_and_the_last_event_in_the_batch() {
        let event = EngineEventEnvelope::new(0, PluginsChangedEvent {}.to_engine_event());
        let store = VecDeque::from([
            HttpIpcEvent {
                event_id: 5,
                event: event.clone(),
            },
            HttpIpcEvent { event_id: 6, event },
        ]);
        let batch = HttpIpcServer::event_batch(&store, 2);
        assert!(batch.cursor_expired);
        assert_eq!(batch.latest_event_id, 6);
        assert_eq!(batch.events.len(), 2);
        let batch = HttpIpcServer::event_batch(&store, 5);
        assert!(!batch.cursor_expired);
        assert_eq!(batch.events[0].event_id, batch.latest_event_id);
        assert!(HttpIpcServer::event_batch(&store, 6).events.is_empty());
        assert!(HttpIpcServer::event_batch(&store, 9).cursor_expired);
    }
}
