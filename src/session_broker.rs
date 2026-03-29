/*
   Unix Azure Entra ID implementation
   Copyright (C) David Mulder <dmulder@samba.org> 2024

   This program is free software: you can redistribute it and/or modify
   it under the terms of the GNU Lesser General Public License as published by
   the Free Software Foundation, either version 3 of the License, or
   (at your option) any later version.

   This program is distributed in the hope that it will be useful,
   but WITHOUT ANY WARRANTY; without even the implied warranty of
   MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE. See the
   GNU Lesser General Public License for more details.

   You should have received a copy of the GNU Lesser General Public License
   along with this program. If not, see <https://www.gnu.org/licenses/>.
*/
use crate::broker_proto::ClientRequest;
#[allow(unused_imports)]
use dbus::arg;
use dbus::blocking::Connection;
use dbus_crossroads as crossroads;
use std::error::Error;
use std::io::{Read, Write};
use std::os::unix::net::UnixStream;
use std::sync::Arc;
use std::time::Duration;
use tracing::{debug, error};

#[cfg(feature = "interactive")]
use crate::broker_proto::{InteractiveAuthCredential, InteractiveAuthResponse};

/// Trait for handling interactive authentication prompts.
///
/// Implementors can customize how each kind of prompt is presented to the
/// user (e.g. using `pinentry` for secrets and the `authenticator` crate
/// for FIDO2).
#[cfg(feature = "interactive")]
pub trait InteractivePromptHandler: Send + Sync {
    /// Prompt the user for a secret (password, PIN, MFA code).
    /// Returns `None` if the user cancels.
    fn prompt_secret(&self, description: &str, prompt: &str) -> Option<String>;
    /// Display an informational message to the user (non-blocking).
    fn show_message(&self, _message: &str) {}
    /// Perform a FIDO2 assertion and return the JSON-encoded result.
    /// Returns `None` if the user cancels or the operation fails.
    fn fido_auth(
        &self,
        fido_challenge: &str,
        fido_allow_list: &[String],
    ) -> Option<String>;
}

pub trait SessionBroker {
    fn acquire_token_interactively(
        &mut self,
        protocol_version: String,
        correlation_id: String,
        request_json: String,
    ) -> Result<String, dbus::MethodErr>;
    fn acquire_token_silently(
        &mut self,
        protocol_version: String,
        correlation_id: String,
        request_json: String,
    ) -> Result<String, dbus::MethodErr>;
    fn get_accounts(
        &mut self,
        protocol_version: String,
        correlation_id: String,
        request_json: String,
    ) -> Result<String, dbus::MethodErr>;
    fn remove_account(
        &mut self,
        protocol_version: String,
        correlation_id: String,
        request_json: String,
    ) -> Result<String, dbus::MethodErr>;
    fn acquire_prt_sso_cookie(
        &mut self,
        protocol_version: String,
        correlation_id: String,
        request_json: String,
    ) -> Result<String, dbus::MethodErr>;
    fn generate_signed_http_request(
        &mut self,
        protocol_version: String,
        correlation_id: String,
        request_json: String,
    ) -> Result<String, dbus::MethodErr>;
    fn cancel_interactive_flow(
        &mut self,
        protocol_version: String,
        correlation_id: String,
        request_json: String,
    ) -> Result<String, dbus::MethodErr>;
    fn get_linux_broker_version(
        &mut self,
        protocol_version: String,
        correlation_id: String,
        request_json: String,
    ) -> Result<String, dbus::MethodErr>;
}

fn register_session_broker<T>(
    cr: &mut crossroads::Crossroads,
) -> crossroads::IfaceToken<T>
where
    T: SessionBroker + Send + 'static,
{
    cr.register("com.microsoft.identity.Broker1", |b| {
        b.method(
            "acquireTokenInteractively",
            ("protocol_version", "correlation_id", "request_json"),
            ("result",),
            |_, t: &mut T, (protocol_version, correlation_id, request_json): (String, String, String)| {
                debug!("D-Bus call: acquireTokenInteractively, correlation_id={}", correlation_id);
                let result = t.acquire_token_interactively(
                    protocol_version,
                    correlation_id.clone(),
                    request_json,
                );
                debug!("D-Bus reply: acquireTokenInteractively, correlation_id={}, success={}",
                    correlation_id, result.is_ok());
                result.map(|x| (x,))
            },
        );
        b.method(
            "acquireTokenSilently",
            ("protocol_version", "correlation_id", "request_json"),
            ("result",),
            |_, t: &mut T, (protocol_version, correlation_id, request_json): (String, String, String)| {
                debug!("D-Bus call: acquireTokenSilently, correlation_id={}", correlation_id);
                let result = t.acquire_token_silently(
                    protocol_version,
                    correlation_id.clone(),
                    request_json,
                );
                debug!("D-Bus reply: acquireTokenSilently, correlation_id={}, success={}",
                    correlation_id, result.is_ok());
                result.map(|x| (x,))
            },
        );
        b.method(
            "getAccounts",
            ("protocol_version", "correlation_id", "request_json"),
            ("result",),
            |_, t: &mut T, (protocol_version, correlation_id, request_json): (String, String, String)| {
                debug!("D-Bus call: getAccounts, correlation_id={}", correlation_id);
                let result = t.get_accounts(protocol_version, correlation_id.clone(), request_json);
                debug!("D-Bus reply: getAccounts, correlation_id={}, success={}",
                    correlation_id, result.is_ok());
                result.map(|x| (x,))
            },
        );
        b.method(
            "removeAccount",
            ("protocol_version", "correlation_id", "request_json"),
            ("result",),
            |_, t: &mut T, (protocol_version, correlation_id, request_json): (String, String, String)| {
                debug!("D-Bus call: removeAccount, correlation_id={}", correlation_id);
                let result = t.remove_account(protocol_version, correlation_id.clone(), request_json);
                debug!("D-Bus reply: removeAccount, correlation_id={}, success={}",
                    correlation_id, result.is_ok());
                result.map(|x| (x,))
            },
        );
        b.method(
            "acquirePrtSsoCookie",
            ("protocol_version", "correlation_id", "request_json"),
            ("result",),
            |_, t: &mut T, (protocol_version, correlation_id, request_json): (String, String, String)| {
                debug!("D-Bus call: acquirePrtSsoCookie, correlation_id={}", correlation_id);
                let result = t.acquire_prt_sso_cookie(
                    protocol_version,
                    correlation_id.clone(),
                    request_json,
                );
                debug!("D-Bus reply: acquirePrtSsoCookie, correlation_id={}, success={}",
                    correlation_id, result.is_ok());
                result.map(|x| (x,))
            },
        );
        b.method(
            "generateSignedHttpRequest",
            ("protocol_version", "correlation_id", "request_json"),
            ("result",),
            |_, t: &mut T, (protocol_version, correlation_id, request_json): (String, String, String)| {
                debug!("D-Bus call: generateSignedHttpRequest, correlation_id={}", correlation_id);
                let result = t.generate_signed_http_request(
                    protocol_version,
                    correlation_id.clone(),
                    request_json,
                );
                debug!("D-Bus reply: generateSignedHttpRequest, correlation_id={}, success={}",
                    correlation_id, result.is_ok());
                result.map(|x| (x,))
            },
        );
        b.method(
            "cancelInteractiveFlow",
            ("protocol_version", "correlation_id", "request_json"),
            ("result",),
            |_, t: &mut T, (protocol_version, correlation_id, request_json): (String, String, String)| {
                debug!("D-Bus call: cancelInteractiveFlow, correlation_id={}", correlation_id);
                let result = t.cancel_interactive_flow(
                    protocol_version,
                    correlation_id.clone(),
                    request_json,
                );
                debug!("D-Bus reply: cancelInteractiveFlow, correlation_id={}, success={}",
                    correlation_id, result.is_ok());
                result.map(|x| (x,))
            },
        );
        b.method(
            "getLinuxBrokerVersion",
            ("protocol_version", "correlation_id", "request_json"),
            ("result",),
            |_, t: &mut T, (protocol_version, correlation_id, request_json): (String, String, String)| {
                debug!("D-Bus call: getLinuxBrokerVersion, correlation_id={}", correlation_id);
                let result = t.get_linux_broker_version(
                    protocol_version,
                    correlation_id.clone(),
                    request_json,
                );
                debug!("D-Bus reply: getLinuxBrokerVersion, correlation_id={}, success={}",
                    correlation_id, result.is_ok());
                result.map(|x| (x,))
            },
        );
    })
}

/// Callbacks for getting and setting the log level at runtime.
/// Used by the LogControl1 D-Bus interface to bridge into whatever
/// logging framework the caller uses (e.g. tracing-subscriber reload).
pub struct LogLevelCallbacks {
    pub get: Arc<dyn Fn() -> String + Send + Sync>,
    pub set: Arc<dyn Fn(&str) -> Result<(), String> + Send + Sync>,
}

struct LogControlState {
    syslog_identifier: String,
    callbacks: LogLevelCallbacks,
}

fn register_log_control1(
    cr: &mut crossroads::Crossroads,
) -> crossroads::IfaceToken<LogControlState> {
    cr.register("org.freedesktop.LogControl1", |b| {
        b.property::<String, _>("LogLevel")
            .emits_changed_false()
            .get(|_ctx, data: &mut LogControlState| {
                Ok((data.callbacks.get)())
            })
            .set(
                |_ctx, data: &mut LogControlState, value: String| {
                    (data.callbacks.set)(&value).map_err(|e| {
                        dbus::MethodErr::failed(&e)
                    })?;
                    Ok(None)
                },
            );
        b.property::<String, _>("LogTarget")
            .emits_changed_false()
            .get(|_ctx, _data: &mut LogControlState| {
                Ok("journal".to_string())
            })
            .set(
                |_ctx, _data: &mut LogControlState, _value: String| {
                    Err(("org.freedesktop.DBus.Error.NotSupported",
                        "Setting log target is not supported".to_string()).into())
                },
            );
        b.property::<String, _>("SyslogIdentifier")
            .emits_changed_false()
            .get(|_ctx, data: &mut LogControlState| {
                Ok(data.syslog_identifier.clone())
            });
    })
}

pub async fn session_broker_serve<T>(
    broker: T,
    syslog_identifier: &str,
    log_callbacks: LogLevelCallbacks,
) -> Result<(), dbus::MethodErr>
where
    T: SessionBroker + Send + 'static,
{
    // Start up a connection to the session bus and request a name
    let c = Connection::new_session()?;
    c.request_name("com.microsoft.identity.broker1", false, true, false)?;

    let mut cr = crossroads::Crossroads::new();

    let token = register_session_broker::<T>(&mut cr);
    let peer = cr.register("org.freedesktop.DBus.Peer", |b| {
        b.method("Ping", (), (),
            |_, _, (): ()| Ok(()));
    });
    cr.insert("/com/microsoft/identity/broker1", &[token, peer], broker);

    let log_control_token = register_log_control1(&mut cr);
    cr.insert(
        "/org/freedesktop/LogControl1",
        &[log_control_token],
        LogControlState {
            syslog_identifier: syslog_identifier.to_string(),
            callbacks: log_callbacks,
        },
    );

    // Serve clients forever.
    cr.serve(&c)?;
    unreachable!()
}

/// Read a complete JSON response from the stream by attempting
/// deserialization after each chunk. This avoids requiring a
/// framing protocol and works with the raw-JSON wire format.
fn recv_json_response(
    stream: &mut UnixStream,
    timeout: Duration,
    request_name: &dyn std::fmt::Display,
) -> Result<String, Box<dyn Error>> {
    const MAX_RESPONSE_SIZE: usize = 4 * 1024 * 1024; // 4 MiB
    stream.set_read_timeout(Some(timeout))?;

    let mut data = Vec::with_capacity(4096);
    let mut buf = [0u8; 4096];

    loop {
        match stream.read(&mut buf) {
            Ok(0) => break,
            Ok(n) => {
                data.extend_from_slice(&buf[..n]);
                if data.len() > MAX_RESPONSE_SIZE {
                    return Err(format!(
                        "Response for {} exceeded max size ({} bytes)",
                        request_name, MAX_RESPONSE_SIZE
                    ).into());
                }
                // Try to parse, a complete JSON value means we're done.
                if serde_json::from_slice::<serde_json::Value>(&data).is_ok() {
                    break;
                }
            }
            Err(ref e) if e.kind() == std::io::ErrorKind::WouldBlock
                || e.kind() == std::io::ErrorKind::TimedOut =>
            {
                // If we already have data, try to parse what we have.
                if !data.is_empty()
                    && serde_json::from_slice::<serde_json::Value>(&data).is_ok()
                {
                    break;
                }
                error!(
                    "Socket timeout waiting for daemon response to {}",
                    request_name
                );
                return Err(Box::new(std::io::Error::new(
                    std::io::ErrorKind::TimedOut,
                    format!("Timeout waiting for response to {}", request_name),
                )));
            }
            Err(e) => {
                error!("Stream read failure for {} -> {:?}", request_name, e);
                return Err(Box::new(e));
            }
        }
    }

    // Verify we got a complete JSON value before returning.
    if data.is_empty() || serde_json::from_slice::<serde_json::Value>(&data).is_err() {
        error!(
            "Unexpected EOF after {} bytes for {}, no valid JSON response",
            data.len(), request_name
        );
        return Err(Box::new(std::io::Error::new(
            std::io::ErrorKind::UnexpectedEof,
            format!("Unexpected EOF waiting for response to {}", request_name),
        )));
    }

    debug!("Received {} bytes from daemon for {}", data.len(), request_name);
    Ok(String::from_utf8(data)?)
}

struct HimmelblauSessionBroker {
    sock_path: String,
    timeout: u64,
    #[cfg(feature = "interactive")]
    prompt_handler: Arc<dyn InteractivePromptHandler>,
}

impl HimmelblauSessionBroker {
    fn request(
        &self,
        message: ClientRequest,
    ) -> Result<String, Box<dyn Error>> {
        debug!(
            "Connecting to daemon socket {} for request {}",
            self.sock_path, message
        );
        let mut stream = UnixStream::connect(&self.sock_path)
            .map_err(|e| {
                error!(
                    "Unix socket stream setup error while connecting to {} -> {:?}",
                    self.sock_path, e
                );
                e
            })
            .map_err(Box::new)?;
        debug!("Connected to daemon socket for {}", message);

        let serialized = serde_json::to_vec(&message)?;
        debug!(
            "Sending {} bytes to daemon for {}",
            serialized.len(),
            message
        );
        stream
            .write_all(&serialized)
            .and_then(|_| stream.flush())
            .map_err(|e| {
                error!("stream write error for {} -> {:?}", message, e);
                e
            })
            .map_err(Box::new)?;
        debug!("Request sent to daemon for {}, waiting for response", message);

        let timeout = Duration::from_secs(self.timeout);
        recv_json_response(&mut stream, timeout, &message)
    }

    /// Send a request on an already-open stream and read the response.
    #[cfg(feature = "interactive")]
    fn send_and_recv(
        &self,
        stream: &mut UnixStream,
        message: ClientRequest,
    ) -> Result<String, Box<dyn Error>> {
        let serialized = serde_json::to_vec(&message)?;
        debug!(
            "Sending {} bytes to daemon for {}",
            serialized.len(),
            message
        );
        stream
            .write_all(&serialized)
            .and_then(|_| stream.flush())
            .map_err(|e| {
                error!("stream write error for {} -> {:?}", message, e);
                e
            })
            .map_err(Box::new)?;

        let timeout = Duration::from_secs(self.timeout);
        recv_json_response(stream, timeout, &message)
    }

    /// Drive the interactive authentication flow over a single connection.
    ///
    /// 1. Sends `interactiveAuthInit` to the daemon.
    /// 2. Reads intermediate prompts and collects credentials via `pinentry`.
    /// 3. Sends `interactiveAuthStep` until the daemon returns success or denied.
    #[cfg(feature = "interactive")]
    fn request_interactive(
        &self,
        protocol_version: String,
        correlation_id: String,
        request_json: String,
    ) -> Result<String, Box<dyn Error>> {
        debug!(
            "Starting interactive auth flow, correlation_id={}",
            correlation_id
        );

        let mut stream = UnixStream::connect(&self.sock_path).map_err(|e| {
            error!(
                "Interactive: socket connect error to {} -> {:?}",
                self.sock_path, e
            );
            Box::new(e)
        })?;

        // Step 1: Send init
        let init_resp = self.send_and_recv(
            &mut stream,
            ClientRequest::interactiveAuthInit(
                protocol_version,
                correlation_id.clone(),
                request_json,
            ),
        )?;

        let mut resp: InteractiveAuthResponse = serde_json::from_str(&init_resp)?;

        // Step 2: Loop prompting until success or denial
        let max_steps = 10;
        for step in 0..max_steps {
            let response_kind = match &resp {
                InteractiveAuthResponse::Success { .. } => "Success",
                InteractiveAuthResponse::Denied { .. } => "Denied",
                InteractiveAuthResponse::PromptPassword => "PromptPassword",
                InteractiveAuthResponse::PromptPin => "PromptPin",
                InteractiveAuthResponse::PromptMFACode { .. } => "PromptMFACode",
                InteractiveAuthResponse::PromptMFAPoll { .. } => "PromptMFAPoll",
                InteractiveAuthResponse::PromptFido { .. } => "PromptFido",
            };
            debug!(
                "Interactive auth step {}, correlation_id={}, response={}",
                step, correlation_id, response_kind
            );
            let cred = match &resp {
                InteractiveAuthResponse::Success { token_response } => {
                    return Ok(token_response.clone());
                }
                InteractiveAuthResponse::Denied { msg } => {
                    return Err(msg.clone().into());
                }
                InteractiveAuthResponse::PromptPassword => {
                    match self.prompt_handler.prompt_secret(
                        "Entra ID Authentication",
                        "Password:",
                    ) {
                        Some(cred) => InteractiveAuthCredential::Password { cred },
                        None => {
                            let _ = self.send_and_recv(
                                &mut stream,
                                ClientRequest::interactiveAuthCancel(correlation_id.clone()),
                            );
                            return Err("User cancelled authentication".into());
                        }
                    }
                }
                InteractiveAuthResponse::PromptPin => {
                    match self.prompt_handler.prompt_secret(
                        "Entra ID Authentication",
                        "Hello PIN:",
                    ) {
                        Some(cred) => InteractiveAuthCredential::Pin { cred },
                        None => {
                            let _ = self.send_and_recv(
                                &mut stream,
                                ClientRequest::interactiveAuthCancel(correlation_id.clone()),
                            );
                            return Err("User cancelled authentication".into());
                        }
                    }
                }
                InteractiveAuthResponse::PromptMFACode { msg } => {
                    match self.prompt_handler.prompt_secret("Entra ID MFA", msg) {
                        Some(cred) => InteractiveAuthCredential::MFACode { cred },
                        None => {
                            let _ = self.send_and_recv(
                                &mut stream,
                                ClientRequest::interactiveAuthCancel(correlation_id.clone()),
                            );
                            return Err("User cancelled authentication".into());
                        }
                    }
                }
                InteractiveAuthResponse::PromptMFAPoll {
                    msg, polling_interval,
                } => {
                    if !msg.is_empty() {
                        self.prompt_handler.show_message(msg);
                    }
                    std::thread::sleep(Duration::from_secs(*polling_interval as u64));
                    InteractiveAuthCredential::MFAPoll {
                        poll_attempt: step as u32,
                    }
                }
                InteractiveAuthResponse::PromptFido {
                    fido_challenge,
                    fido_allow_list,
                } => {
                    match self
                        .prompt_handler
                        .fido_auth(fido_challenge, fido_allow_list)
                    {
                        Some(assertion) => InteractiveAuthCredential::Fido { assertion },
                        None => {
                            let _ = self.send_and_recv(
                                &mut stream,
                                ClientRequest::interactiveAuthCancel(correlation_id.clone()),
                            );
                            return Err("FIDO2 authentication failed or cancelled".into());
                        }
                    }
                }
            };

            let cred_json = serde_json::to_string(&cred)?;
            let step_resp = self.send_and_recv(
                &mut stream,
                ClientRequest::interactiveAuthStep(correlation_id.clone(), cred_json),
            )?;
            resp = serde_json::from_str(&step_resp)?;
        }

        let _ = self.send_and_recv(
            &mut stream,
            ClientRequest::interactiveAuthCancel(correlation_id),
        );
        Err("Interactive auth: too many steps".into())
    }
}

impl SessionBroker for HimmelblauSessionBroker {
    fn acquire_token_interactively(
        &mut self,
        protocol_version: String,
        correlation_id: String,
        request_json: String,
    ) -> Result<String, dbus::MethodErr> {
        // First try the existing daemon-side path (one-shot request).
        let initial_result = self.request(ClientRequest::acquireTokenInteractively(
            protocol_version.clone(),
            correlation_id.clone(),
            request_json.clone(),
        ));

        match &initial_result {
            Ok(resp) => {
                // Check if the response contains a valid token
                if let Ok(v) = serde_json::from_str::<serde_json::Value>(resp) {
                    if v.get("brokerTokenResponse")
                        .and_then(|t| t.get("accessToken"))
                        .is_some()
                    {
                        return initial_result.map_err(|e| dbus::MethodErr::failed(&e));
                    }
                }
            }
            Err(_) => {}
        }

        #[cfg(feature = "interactive")]
        {
            debug!(
                "One-shot acquisition did not return a token for correlation_id={}, starting interactive flow",
                correlation_id
            );
            return self
                .request_interactive(protocol_version, correlation_id, request_json)
                .map_err(|e| dbus::MethodErr::failed(&e));
        }

        #[cfg(not(feature = "interactive"))]
        {
            initial_result.map_err(|e| dbus::MethodErr::failed(&e))
        }
    }

    fn acquire_token_silently(
        &mut self,
        protocol_version: String,
        correlation_id: String,
        request_json: String,
    ) -> Result<String, dbus::MethodErr> {
        self.request(ClientRequest::acquireTokenSilently(
            protocol_version,
            correlation_id,
            request_json,
        ))
        .map_err(|e| dbus::MethodErr::failed(&e))
    }

    fn get_accounts(
        &mut self,
        protocol_version: String,
        correlation_id: String,
        request_json: String,
    ) -> Result<String, dbus::MethodErr> {
        self.request(ClientRequest::getAccounts(
            protocol_version,
            correlation_id,
            request_json,
        ))
        .map_err(|e| dbus::MethodErr::failed(&e))
    }

    fn remove_account(
        &mut self,
        protocol_version: String,
        correlation_id: String,
        request_json: String,
    ) -> Result<String, dbus::MethodErr> {
        self.request(ClientRequest::removeAccount(
            protocol_version,
            correlation_id,
            request_json,
        ))
        .map_err(|e| dbus::MethodErr::failed(&e))
    }

    fn acquire_prt_sso_cookie(
        &mut self,
        protocol_version: String,
        correlation_id: String,
        request_json: String,
    ) -> Result<String, dbus::MethodErr> {
        self.request(ClientRequest::acquirePrtSsoCookie(
            protocol_version,
            correlation_id,
            request_json,
        ))
        .map_err(|e| dbus::MethodErr::failed(&e))
    }

    fn generate_signed_http_request(
        &mut self,
        protocol_version: String,
        correlation_id: String,
        request_json: String,
    ) -> Result<String, dbus::MethodErr> {
        self.request(ClientRequest::generateSignedHttpRequest(
            protocol_version,
            correlation_id,
            request_json,
        ))
        .map_err(|e| dbus::MethodErr::failed(&e))
    }

    fn cancel_interactive_flow(
        &mut self,
        protocol_version: String,
        correlation_id: String,
        request_json: String,
    ) -> Result<String, dbus::MethodErr> {
        self.request(ClientRequest::cancelInteractiveFlow(
            protocol_version,
            correlation_id,
            request_json,
        ))
        .map_err(|e| dbus::MethodErr::failed(&e))
    }

    fn get_linux_broker_version(
        &mut self,
        protocol_version: String,
        correlation_id: String,
        request_json: String,
    ) -> Result<String, dbus::MethodErr> {
        self.request(ClientRequest::getLinuxBrokerVersion(
            protocol_version,
            correlation_id,
            request_json,
        ))
        .map_err(|e| dbus::MethodErr::failed(&e))
    }
}

/* The session Broker is simply a DBus session service which forwards messages
 * to the Himmelblau Broker. This layer is necessary because this service
 * imitates the existing Microsoft Broker. Imitating Microsoft's service buys
 * us compatibility with existing projects, such as linux-entra-sso, and with
 * Microsoft's own apps (Edge, Teams, etc). How well this is compatible with
 * Microsoft's apps will depend on the implementation of the HimmelblauBroker
 * trait in src/himmelblau_broker.rs.
 *
 * To utilize this service, simply call `session_broker_serve()`
 * from a binary, then install that binary as a session service on your
 * Linux distribution.
 */
pub async fn himmelblau_session_broker_serve(
    sock_path: &str,
    timeout: u64,
    log_callbacks: LogLevelCallbacks,
    #[cfg(feature = "interactive")]
    prompt_handler: Arc<dyn InteractivePromptHandler>,
) -> Result<(), dbus::MethodErr> {
    session_broker_serve(
        HimmelblauSessionBroker {
            sock_path: sock_path.to_string(),
            timeout,
            #[cfg(feature = "interactive")]
            prompt_handler,
        },
        "himmelblau_broker",
        log_callbacks,
    )
    .await
}
