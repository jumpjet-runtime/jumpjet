use std::io::{Read, Write};
use std::net::{TcpListener, TcpStream};
use std::rc::Rc;
use std::sync::Arc;
use std::sync::atomic::{AtomicI64, Ordering};

use gimli::{Dwarf, EndianRcSlice, Reader, RunTimeEndian};
use object::{File, Object, ObjectSection};
use serde_json::Value;
use wasmtime::AsContext; // Using File directly to solve unused object warning slightly

use crate::debug::context::DebugContext;
use crate::runtime::JumpjetRuntimeState;

pub const DEFAULT_DAP_PORT: u16 = 54321;

/// Global sequence counter for events/responses sent by the adapter.
static SEQ_COUNTER: AtomicI64 = AtomicI64::new(1);

fn next_seq() -> i64 {
    SEQ_COUNTER.fetch_add(1, Ordering::Relaxed)
}

pub struct DapConnection {
    pub stream: TcpStream,
    pub buffer: Vec<u8>,
}

impl DapConnection {
    pub fn new(stream: TcpStream) -> Self {
        Self {
            stream,
            buffer: Vec::new(),
        }
    }
}

fn find_inner_module_range(binary: &[u8]) -> Option<std::ops::Range<usize>> {
    // Try to parse as Wasm Component to find inner Module
    let parser = wasmparser::Parser::new(0);
    for payload in parser.parse_all(binary) {
        match payload {
            Ok(wasmparser::Payload::ModuleSection {
                unchecked_range: range,
                ..
            }) => {
                return Some(range);
            }
            _ => {}
        }
    }
    None
}

fn find_address_for_line(binary: &[u8], filename: &str, line: i64) -> Option<u32> {
    eprintln!(
        "find_address_for_line: searching for {} line {} in binary of size {} bytes",
        filename,
        line,
        binary.len()
    );

    let range = match File::parse(binary) {
        Ok(_) => 0..binary.len(),
        Err(e) => {
            eprintln!(
                "find_address_for_line: failed to parse binary as object file: {:?}. Checking for Wasm Component...",
                e
            );
            match find_inner_module_range(binary) {
                Some(r) => {
                    eprintln!(
                        "find_address_for_line: found inner module at range {:?}, recursing...",
                        r
                    );
                    r
                }
                None => {
                    eprintln!(
                        "find_address_for_line: no inner module found or object parse failed."
                    );
                    return None;
                }
            }
        }
    };

    let inner_binary = &binary[range.start..range.end];
    let file = match File::parse(inner_binary) {
        Ok(f) => f,
        Err(e) => {
            eprintln!("find_address_for_line: inner module parse failed: {:?}", e);
            return None;
        }
    };

    let endian = if file.is_little_endian() {
        RunTimeEndian::Little
    } else {
        RunTimeEndian::Big
    };

    let load_section =
        |id: gimli::SectionId| -> Result<EndianRcSlice<RunTimeEndian>, gimli::Error> {
            let data = file
                .section_by_name(id.name())
                .and_then(|section| section.uncompressed_data().ok())
                .unwrap_or(std::borrow::Cow::Borrowed(&[][..]));
            Ok(EndianRcSlice::new(Rc::from(&*data), endian))
        };

    let dwarf = match Dwarf::load(&load_section) {
        Ok(d) => d,
        Err(e) => {
            eprintln!("find_address_for_line: failed to load DWARF info: {:?}", e);
            return None;
        }
    };

    eprintln!("find_address_for_line: DWARF loaded, iterating units...");

    // Iterate over compilation units
    let mut iter = dwarf.units();
    while let Ok(Some(header)) = iter.next() {
        let unit = dwarf.unit(header).ok()?;

        // Get the line program for this unit
        if let Some(program) = unit.line_program.clone() {
            let mut rows = program.rows();
            while let Ok(Some((header, row))) = rows.next_row() {
                if let Some(file_entry) = row.file(header) {
                    // path_name returns AttributeValue, no ? needed
                    if let Some(path) = dwarf.attr_string(&unit, file_entry.path_name()).ok() {
                        let path_cow = path.to_string_lossy().ok()?; // Returns Result<Cow>, needs handling
                        // Simple substring match for filename (should be improved for full paths)
                        if path_cow.contains(filename) {
                            if let Some(l) = row.line() {
                                if l.get() == line as u64 {
                                    let addr = row.address();
                                    eprintln!(
                                        "Found verification: {} line {} -> {:#x}",
                                        filename, line, addr
                                    );
                                    return Some(addr as u32);
                                }
                            }
                        }
                    }
                }
            }
        }
    }

    eprintln!("Failed to find address for {} line {}", filename, line);
    None
}

pub fn start_dap_server(port: u16) -> std::io::Result<TcpListener> {
    let addr = format!("127.0.0.1:{}", port);
    eprintln!("DAP server binding to {}", addr);
    let listener = TcpListener::bind(&addr)?;
    listener.set_nonblocking(true)?;
    eprintln!("DAP server bound successfully");
    Ok(listener)
}

pub enum ControlFlow {
    Continue,
    StayPaused,
}

pub fn handle_dap_event(
    store: wasmtime::StoreContextMut<'_, JumpjetRuntimeState>,
    connection: &mut DapConnection,
    initial_stop: Option<dapts::StoppedEventReason>,
    backtrace: Option<&wasmtime::WasmBacktrace>,
    binary: Arc<Vec<u8>>,
) -> anyhow::Result<()> {
    let mut cached_stack = None;
    if let Some(bt) = backtrace {
        let mut frames = Vec::new();
        for (index, frame) in bt.frames().iter().enumerate() {
            let func_name = frame.func_name().unwrap_or("unknown");
            let mut source = None;
            let mut line = 0;
            let mut column = 0;

            for sym in frame.symbols().iter().take(1) {
                let sym: &wasmtime::FrameSymbol = sym;
                if let Some(f) = sym.file() {
                    // Try to make absolute path?
                    // DAP expects absolute path or path relative to workspace.
                    // If it's just filename, it might not work?
                    // But `find_address_for_line` uses filename match.
                    // Let's use whatever `sym.file()` gives.
                    source = Some(f.to_string());
                }
                if let Some(l) = sym.line() {
                    line = l as i64;
                }
                if let Some(c) = sym.column() {
                    column = c as i64;
                }
            }

            frames.push(crate::debug::context::StackFrameInfo {
                id: index as i64,
                name: func_name.to_string(),
                source,
                line,
                column,
                instruction_pointer_reference: None,
            });
        }
        cached_stack = Some(frames);
    }

    let mut context = DebugContext::new(store, cached_stack);
    // We can't use BufReader here because we need to persist the buffer.
    // Instead we read from connection.stream into connection.buffer
    let mut writer = connection.stream.try_clone()?;

    eprintln!("DAP connection handler started");

    let mut is_stopped = initial_stop.is_some();

    // Determine initial stop reason from DebugEvent if present
    if let Some(reason) = initial_stop {
        let stopped_body = dapts::StoppedEvent {
            reason,
            description: None,
            thread_id: Some(1),
            preserve_focus_hint: None,
            text: None,
            all_threads_stopped: Some(true),
            hit_breakpoint_ids: None,
        };
        send_event(&mut writer, "stopped", &stopped_body)?;
    }

    loop {
        // Read available data from stream into buffer
        // We use non-blocking read
        let mut temp_buf = [0u8; 4096];
        match connection.stream.read(&mut temp_buf) {
            Ok(0) => break, // EOF
            Ok(n) => {
                connection.buffer.extend_from_slice(&temp_buf[..n]);
            }
            Err(ref e) if e.kind() == std::io::ErrorKind::WouldBlock => {
                // No more data to read right now
            }
            Err(e) => {
                eprintln!("DAP connection read error: {:?}", e);
                break;
            }
        }

        // Try to parse messages from buffer
        // loop to handle multiple messages in buffer
        loop {
            match try_parse_message(&mut connection.buffer) {
                Ok(Some(msg)) => {
                    match handle_message(&mut context, &mut writer, msg, binary.clone(), is_stopped)
                    {
                        Ok(ControlFlow::Continue) => return Ok(()), // Break outer loop? continue running guest
                        Ok(ControlFlow::StayPaused) => {
                            if !is_stopped {
                                // We transitioned from running to stopped
                                let stopped_body = dapts::StoppedEvent {
                                    reason: dapts::StoppedEventReason::Pause,
                                    description: None,
                                    thread_id: Some(1),
                                    preserve_focus_hint: None,
                                    text: None,
                                    all_threads_stopped: Some(true),
                                    hit_breakpoint_ids: None,
                                };
                                send_event(&mut writer, "stopped", &stopped_body)?;
                                is_stopped = true;
                            }
                            // Continue processing messages
                        }
                        Err(e) => {
                            eprintln!("Error handling DAP message: {:?}", e);
                        }
                    }
                }
                Ok(None) => break, // Incomplete message
                Err(e) => {
                    eprintln!("DAP parse error: {:?}. Clearing buffer.", e);
                    connection.buffer.clear(); // Recover by clearing? Or disconnect?
                    break;
                }
            }
        }

        // If we processed all messages and are "StayPaused" (is_stopped), we should probably block/sleep?
        // But if we are running (is_stopped = false), we must return to guest.

        if is_stopped {
            // If stopped, we loop here waiting for user input.
            // We should sleep to avoid busy loop.
            std::thread::sleep(std::time::Duration::from_millis(10));
            continue; // Outer loop (read from stream again)
        } else {
            // If running, we return to guest loop
            break;
        }
    }

    Ok(())
}

fn try_parse_message(buffer: &mut Vec<u8>) -> anyhow::Result<Option<Value>> {
    // Check for "Content-Length: " header
    let header_end = match buffer.windows(4).position(|w| w == b"\r\n\r\n") {
        Some(pos) => pos,
        None => return Ok(None),
    };

    let headers = std::str::from_utf8(&buffer[..header_end])?;
    let mut content_length = 0;

    for line in headers.lines() {
        if let Some(rest) = line.strip_prefix("Content-Length: ") {
            if let Ok(len) = rest.trim().parse::<usize>() {
                content_length = len;
            }
        }
    }

    if content_length == 0 {
        // Should ignore/consume headers? Or error?
        // DAP spec implies Content-Length is required.
        // Remove headers and continue?
        buffer.drain(..header_end + 4);
        return Ok(None);
    }

    let total_len = header_end + 4 + content_length;
    if buffer.len() < total_len {
        return Ok(None);
    }

    // We have a full message
    let msg_bytes = &buffer[header_end + 4..total_len];
    let value: Value = serde_json::from_slice(msg_bytes)?;
    eprintln!("DAP received message: {}", value);

    // Create new buffer without the consumed message (inefficient but safe)
    // buffer.drain(..total_len); // drain is valid
    buffer.drain(..total_len);

    Ok(Some(value))
}

fn handle_message(
    context: &mut DebugContext,
    writer: &mut TcpStream,
    message: Value,
    binary: Arc<Vec<u8>>,
    is_stopped: bool,
) -> anyhow::Result<ControlFlow> {
    // Parse as dapts::Request — a flat struct with {seq, command, arguments}
    let request: dapts::Request = serde_json::from_value(message)
        .map_err(|e| anyhow::anyhow!("Failed to parse DAP request: {}", e))?;

    let seq = request.seq;
    let command = request.command.clone();

    eprintln!("DAP handling command: {} (seq: {})", command, seq);

    match command.as_str() {
        "initialize" => {
            // Respond with capabilities
            let caps = dapts::Capabilities {
                supports_configuration_done_request: Some(true),
                supports_function_breakpoints: Some(false),
                supports_conditional_breakpoints: Some(false),
                supports_evaluate_for_hovers: Some(true),
                ..Default::default()
            };
            let response = dapts::Response::success(seq, caps);
            send_response(writer, &command, response)?;

            // Send "initialized" event (body is optional capabilities, use null)
            send_event(writer, "initialized", &serde_json::Value::Null)?;

            // Should stay paused/active to wait for config?
            // Usually Initialize -> initialized -> launch -> ...
            // If we return Continue here, we might exit the loop if called with None.
            // But Initialize usually happens at start.
            Ok(ControlFlow::StayPaused)
        }
        "launch" | "attach" => {
            // Acknowledge, body is empty for launch/attach
            let response = dapts::Response::new(seq, true, None, None::<()>);
            send_response(writer, &command, response)?;
            Ok(ControlFlow::StayPaused) // Wait for config
        }
        "setBreakpoints" => {
            let args: dapts::SetBreakpointsArguments =
                serde_json::from_value(request.arguments.clone())
                    .map_err(|e| anyhow::anyhow!("Failed to parse setBreakpoints args: {}", e))?;

            let source_path = args.source.path.clone().unwrap_or_default();
            // Basic filename extraction
            let filename = std::path::Path::new(&source_path)
                .file_name()
                .and_then(|f| f.to_str())
                .unwrap_or("");

            // Attempt to pre-load module if necessary
            let mut module_cache: Option<wasmtime::Module> = None;

            let mut breakpoints = Vec::new();
            if let Some(source_bps) = args.breakpoints {
                for src_bp in source_bps {
                    let line = src_bp.line;

                    // improved logic: find address using DWARF
                    if let Some(addr) = find_address_for_line(&binary, filename, line as i64) {
                        // First try standard add_breakpoint (uses frames)
                        if context.add_breakpoint(addr).unwrap_or(false) {
                            breakpoints.push(dapts::Breakpoint {
                                verified: true,
                                line: Some(line),
                                message: None,
                                ..Default::default()
                            });
                        } else {
                            // If frames failed (e.g. not trapped), try loading module from binary
                            if module_cache.is_none() {
                                let engine = context.store.as_context().engine().clone();
                                let mut module_binary = &binary[..];
                                if let Some(range) = find_inner_module_range(&binary) {
                                    module_binary = &binary[range.start..range.end];
                                }

                                // Attempt to parse module
                                if let Ok(m) = wasmtime::Module::new(&engine, module_binary) {
                                    eprintln!(
                                        "Successfully loaded module from binary for breakpoint setting"
                                    );
                                    module_cache = Some(m);
                                } else {
                                    eprintln!("Failed to load module from binary");
                                }
                            }

                            let verified = if let Some(m) = &module_cache {
                                context.add_breakpoint_with_module(m, addr).unwrap_or(false)
                            } else {
                                false
                            };

                            breakpoints.push(dapts::Breakpoint {
                                verified,
                                line: if verified { Some(line) } else { None },
                                message: if verified {
                                    None
                                } else {
                                    Some(
                                        "Could not verify breakpoint location (no module)"
                                            .to_string(),
                                    )
                                },
                                ..Default::default()
                            });
                        }
                    } else {
                        // Address not found in DWARF
                        breakpoints.push(dapts::Breakpoint {
                            verified: false,
                            line: None,
                            message: Some(
                                "Could not verify breakpoint location (DWARF lookup failed)"
                                    .to_string(),
                            ),
                            ..Default::default()
                        });
                    }
                }
            }

            let body = dapts::SetBreakpointsResponse { breakpoints };
            let response = dapts::Response::success(seq, body);
            send_response(writer, &command, response)?;

            // If running, continue running. If stopped, stay stopped.
            if is_stopped {
                Ok(ControlFlow::StayPaused)
            } else {
                Ok(ControlFlow::Continue)
            }
        }
        "setExceptionBreakpoints" => {
            // Acknowledge — we don't support exception breakpoints yet
            let body = dapts::SetExceptionBreakpointsResponse { breakpoints: None };
            let response = dapts::Response::success(seq, body);
            send_response(writer, &command, response)?;
            Ok(ControlFlow::StayPaused)
        }
        "configurationDone" => {
            let response = dapts::Response::new(seq, true, None, None::<()>);
            send_response(writer, &command, response)?;
            // Always continue after config done, unless we are already stopped at breakpoint?
            // Usually config done means "go".
            Ok(ControlFlow::Continue)
        }
        "continue" => {
            let body = dapts::ContinueResponse {
                all_threads_continued: Some(true),
            };
            let response = dapts::Response::success(seq, body);
            send_response(writer, &command, response)?;
            Ok(ControlFlow::Continue)
        }
        "next" | "stepIn" | "stepOut" => {
            context.single_step()?;
            let response = dapts::Response::new(seq, true, None, None::<()>);
            send_response(writer, &command, response)?;
            Ok(ControlFlow::Continue)
        }
        "pause" => {
            // We want to pause
            let response = dapts::Response::new(seq, true, None, None::<()>);
            send_response(writer, &command, response)?;
            Ok(ControlFlow::StayPaused)
        }
        "threads" => {
            let body = dapts::ThreadsResponse {
                threads: vec![dapts::Thread {
                    id: 1,
                    name: "main".to_string(),
                }],
            };
            let response = dapts::Response::success(seq, body);
            send_response(writer, &command, response)?;
            Ok(ControlFlow::StayPaused)
        }
        "stackTrace" => {
            let frames = context.get_stack_trace();
            let stack_frames = frames
                .into_iter()
                .map(|f| dapts::StackFrame {
                    id: f.id as u64,
                    name: f.name,
                    source: None,
                    line: f.line as u32,
                    column: f.column as u32,
                    end_line: None,
                    end_column: None,
                    instruction_pointer_reference: None,
                    module_id: None,
                    presentation_hint: None,
                    can_restart: None,
                })
                .collect::<Vec<_>>();

            let total_frames = Some(stack_frames.len() as u64);
            let body = dapts::StackTraceResponse {
                stack_frames,
                total_frames,
            };
            let response = dapts::Response::success(seq, body);
            send_response(writer, &command, response)?;
            Ok(ControlFlow::StayPaused)
        }
        "scopes" => {
            // Return a single "Locals" scope
            let body = dapts::ScopesResponse {
                scopes: vec![dapts::Scope {
                    name: "Locals".to_string(),
                    variables_reference: 1, // Arbitrary ref ID for locals
                    expensive: false,
                    named_variables: None,
                    indexed_variables: None,
                    source: None,
                    line: None,
                    column: None,
                    end_line: None,
                    end_column: None,
                    presentation_hint: None,
                }],
            };
            let response = dapts::Response::success(seq, body);
            send_response(writer, &command, response)?;
            Ok(ControlFlow::StayPaused)
        }
        "variables" => {
            // Return empty variables for now
            let body = dapts::VariablesResponse { variables: vec![] };
            let response = dapts::Response::success(seq, body);
            send_response(writer, &command, response)?;
            Ok(ControlFlow::StayPaused)
        }
        "evaluate" => {
            // TODO: Implement expression evaluation
            let response = dapts::Response::error(seq, Some("not supported".to_string()), None);
            send_response(writer, &command, response)?;
            Ok(ControlFlow::StayPaused)
        }
        "disconnect" => {
            let response = dapts::Response::new(seq, true, None, None::<()>);
            send_response(writer, &command, response)?;
            // Use Error to break loop via Err?
            // Or just break loop.
            // But if we break, we might return to run.rs and continue running?
            // If disconnect, we might want to kill the game?
            // Or just stop debugging.
            return Err(anyhow::anyhow!("Client disconnected"));
        }
        other => {
            // Unknown command — send error response per DAP spec
            eprintln!("DAP: Unhandled command '{}'", other);
            let response =
                dapts::Response::error(seq, Some(format!("unsupported command: {}", other)), None);
            send_response(writer, &command, response)?;
            Ok(ControlFlow::StayPaused)
        }
    }
}

/// Send a DAP response using the base protocol framing.
fn send_response(
    writer: &mut TcpStream,
    command: &str,
    response: dapts::Response,
) -> std::io::Result<()> {
    // DAP base protocol requires: type, seq, request_seq, success, command, [message], [body]
    // dapts::Response has {request_seq, success, message, body} but is missing
    // the outer "type", "seq", and "command" fields.
    let mut envelope = serde_json::to_value(&response)?;
    let map = envelope.as_object_mut().unwrap();
    map.insert(
        "type".to_string(),
        serde_json::Value::String("response".to_string()),
    );
    map.insert(
        "command".to_string(),
        serde_json::Value::String(command.to_string()),
    );
    map.insert(
        "seq".to_string(),
        serde_json::Value::Number(next_seq().into()),
    );

    let json = serde_json::to_string(&envelope)?;
    let length = json.len();

    eprintln!("DAP sending response: {}", json);

    write!(writer, "Content-Length: {}\r\n\r\n{}", length, json)?;
    writer.flush()?;
    Ok(())
}

/// Send a DAP event using the base protocol framing.
fn send_event(
    writer: &mut TcpStream,
    event_name: &str,
    body: &impl serde::Serialize,
) -> std::io::Result<()> {
    let event = dapts::Event::new(next_seq(), event_name.to_string(), body);

    // Wrap with "type": "event" for base protocol
    let mut envelope = serde_json::to_value(&event)?;
    let map = envelope.as_object_mut().unwrap();
    map.insert(
        "type".to_string(),
        serde_json::Value::String("event".to_string()),
    );

    let json = serde_json::to_string(&envelope)?;
    let length = json.len();

    eprintln!("DAP sending event: {}", json);

    write!(writer, "Content-Length: {}\r\n\r\n{}", length, json)?;
    writer.flush()?;
    Ok(())
}
