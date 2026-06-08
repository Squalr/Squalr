use anyhow::{Context, Result, anyhow};
use serde::{Deserialize, Serialize};
use squalr_engine::engine_bindings::executable_command_privileged::ExecutableCommandPrivileged;
use squalr_engine::engine_privileged_state::EnginePrivilegedState;
use squalr_engine_api::{
    commands::{privileged_command::PrivilegedCommand, privileged_command_result::PrivilegedCommandResult},
    engine::engine_event_envelope::EngineEventEnvelope,
};
use std::{
    collections::{HashMap, VecDeque},
    io::{BufRead, BufReader, Read, Write},
    net::{TcpListener, TcpStream},
    sync::{
        Arc, Mutex,
        atomic::{AtomicU64, Ordering},
    },
    thread,
    time::Duration,
};

const MAX_REQUEST_BODY_BYTES: usize = 1024 * 1024;
const MAX_RETAINED_EVENTS: usize = 4096;

pub struct HttpIpcServer {
    bind_address: String,
    engine_privileged_state: Arc<EnginePrivilegedState>,
    event_store: Arc<Mutex<VecDeque<HttpIpcEvent>>>,
    command_execution_lock: Arc<Mutex<()>>,
    next_event_id: Arc<AtomicU64>,
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
}

impl HttpIpcServer {
    pub fn new(
        bind_address: String,
        engine_privileged_state: Arc<EnginePrivilegedState>,
    ) -> Self {
        Self {
            bind_address,
            engine_privileged_state,
            event_store: Arc::new(Mutex::new(VecDeque::new())),
            command_execution_lock: Arc::new(Mutex::new(())),
            next_event_id: Arc::new(AtomicU64::new(1)),
        }
    }

    pub fn run(&self) -> Result<()> {
        self.spawn_event_collector();

        let listener = TcpListener::bind(&self.bind_address).with_context(|| format!("Failed to bind HTTP IPC server to '{}'.", self.bind_address))?;
        log::info!("HTTP IPC server listening on {}.", self.bind_address);

        for incoming_connection in listener.incoming() {
            match incoming_connection {
                Ok(stream) => {
                    let engine_privileged_state = self.engine_privileged_state.clone();
                    let event_store = self.event_store.clone();
                    let command_execution_lock = self.command_execution_lock.clone();
                    let next_event_id = self.next_event_id.clone();

                    thread::spawn(move || {
                        if let Err(error) = Self::handle_connection(engine_privileged_state, event_store, command_execution_lock, next_event_id, stream) {
                            log::warn!("HTTP IPC request failed: {}", error);
                        }
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
        let next_event_id = self.next_event_id.clone();

        thread::spawn(move || {
            while let Ok(event) = event_receiver.recv() {
                let event_id = next_event_id.fetch_add(1, Ordering::SeqCst);

                match event_store.lock() {
                    Ok(mut event_store) => {
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
        next_event_id: Arc<AtomicU64>,
        mut stream: TcpStream,
    ) -> Result<()> {
        stream
            .set_read_timeout(Some(Duration::from_secs(30)))
            .context("Failed to set HTTP IPC read timeout.")?;
        let request = Self::read_request(&mut stream)?;

        match (request.method.as_str(), request.path_without_query().as_str()) {
            ("GET", "/health") => Self::write_json(
                &mut stream,
                200,
                &HealthResponse {
                    success: true,
                    service: "squalr-http-ipc",
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
                if let Err(error) = Self::handle_events(event_store, next_event_id, &mut stream, request) {
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
        let should_include_privileged_registry_catalog = privileged_command.should_include_privileged_registry_catalog();
        let _command_execution_guard = command_execution_lock
            .lock()
            .map_err(|error| anyhow!("Failed to lock HTTP IPC command executor: {}", error))?;
        let privileged_command_response = privileged_command.execute(&engine_privileged_state);
        let privileged_registry_catalog = if should_include_privileged_registry_catalog {
            Some(engine_privileged_state.get_privileged_registry_catalog())
        } else {
            None
        };
        let result = PrivilegedCommandResult::new(privileged_command_response, privileged_registry_catalog);

        Self::write_json(stream, 200, &CommandResponse { success: true, result })
    }

    fn handle_events(
        event_store: Arc<Mutex<VecDeque<HttpIpcEvent>>>,
        next_event_id: Arc<AtomicU64>,
        stream: &mut TcpStream,
        request: HttpRequest,
    ) -> Result<()> {
        let after_event_id = request
            .query_parameters()
            .get("after")
            .and_then(|after_event_id| after_event_id.parse::<u64>().ok())
            .unwrap_or(0);
        let events = event_store
            .lock()
            .map_err(|error| anyhow!("Failed to lock HTTP IPC event store: {}", error))?
            .iter()
            .filter(|event| event.event_id > after_event_id)
            .cloned()
            .collect::<Vec<_>>();
        let latest_event_id = next_event_id.load(Ordering::SeqCst).saturating_sub(1);

        Self::write_json(
            stream,
            200,
            &EventsResponse {
                success: true,
                events,
                latest_event_id,
            },
        )
    }

    fn read_request(stream: &mut TcpStream) -> Result<HttpRequest> {
        let mut reader = BufReader::new(stream);
        let mut request_line = String::new();
        reader
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
            reader
                .read_line(&mut header_line)
                .context("Failed to read HTTP IPC request header.")?;
            let trimmed_header_line = header_line.trim_end_matches(['\r', '\n']);

            if trimmed_header_line.is_empty() {
                break;
            }

            if let Some((header_name, header_value)) = trimmed_header_line.split_once(':') {
                headers.insert(header_name.trim().to_ascii_lowercase(), header_value.trim().to_string());
            }
        }

        let content_length = headers
            .get("content-length")
            .and_then(|content_length| content_length.parse::<usize>().ok())
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
