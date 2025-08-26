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
use std::time::Duration;
use std::time::SystemTime;
use tracing::{debug, error};

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
            |_, t: &mut T, (protocol_version, correlation_id, request_json)| {
                t.acquire_token_interactively(
                    protocol_version,
                    correlation_id,
                    request_json,
                )
                .map(|x| (x,))
            },
        );
        b.method(
            "acquireTokenSilently",
            ("protocol_version", "correlation_id", "request_json"),
            ("result",),
            |_, t: &mut T, (protocol_version, correlation_id, request_json)| {
                t.acquire_token_silently(
                    protocol_version,
                    correlation_id,
                    request_json,
                )
                .map(|x| (x,))
            },
        );
        b.method(
            "getAccounts",
            ("protocol_version", "correlation_id", "request_json"),
            ("result",),
            |_, t: &mut T, (protocol_version, correlation_id, request_json)| {
                t.get_accounts(protocol_version, correlation_id, request_json)
                    .map(|x| (x,))
            },
        );
        b.method(
            "removeAccount",
            ("protocol_version", "correlation_id", "request_json"),
            ("result",),
            |_, t: &mut T, (protocol_version, correlation_id, request_json)| {
                t.remove_account(protocol_version, correlation_id, request_json)
                    .map(|x| (x,))
            },
        );
        b.method(
            "acquirePrtSsoCookie",
            ("protocol_version", "correlation_id", "request_json"),
            ("result",),
            |_, t: &mut T, (protocol_version, correlation_id, request_json)| {
                t.acquire_prt_sso_cookie(
                    protocol_version,
                    correlation_id,
                    request_json,
                )
                .map(|x| (x,))
            },
        );
        b.method(
            "generateSignedHttpRequest",
            ("protocol_version", "correlation_id", "request_json"),
            ("result",),
            |_, t: &mut T, (protocol_version, correlation_id, request_json)| {
                t.generate_signed_http_request(
                    protocol_version,
                    correlation_id,
                    request_json,
                )
                .map(|x| (x,))
            },
        );
        b.method(
            "cancelInteractiveFlow",
            ("protocol_version", "correlation_id", "request_json"),
            ("result",),
            |_, t: &mut T, (protocol_version, correlation_id, request_json)| {
                t.cancel_interactive_flow(
                    protocol_version,
                    correlation_id,
                    request_json,
                )
                .map(|x| (x,))
            },
        );
        b.method(
            "getLinuxBrokerVersion",
            ("protocol_version", "correlation_id", "request_json"),
            ("result",),
            |_, t: &mut T, (protocol_version, correlation_id, request_json)| {
                t.get_linux_broker_version(
                    protocol_version,
                    correlation_id,
                    request_json,
                )
                .map(|x| (x,))
            },
        );
    })
}

pub async fn session_broker_serve<T>(broker: T) -> Result<(), dbus::MethodErr>
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

    // Serve clients forever.
    cr.serve(&c)?;
    unreachable!()
}

struct HimmelblauSessionBroker {
    sock_path: String,
    timeout: u64,
}

impl HimmelblauSessionBroker {
    fn request(
        &self,
        message: ClientRequest,
    ) -> Result<String, Box<dyn Error>> {
        let mut stream = UnixStream::connect(&self.sock_path)
            .map_err(|e| {
                error!(
                    "Unix socket stream setup error while connecting to {} -> {:?}",
                    self.sock_path, e
                );
                e
            })
            .map_err(Box::new)?;

        stream
            .write_all(&serde_json::to_vec(&message)?)
            .and_then(|_| stream.flush())
            .map_err(|e| {
                error!("stream write error -> {:?}", e);
                e
            })
            .map_err(Box::new)?;

        // Now wait on the response.
        let start = SystemTime::now();
        let mut read_started = false;
        let mut data = Vec::with_capacity(1024);
        let mut counter = 0;
        let timeout = Duration::from_secs(self.timeout);

        loop {
            let mut buffer = [0; 1024];
            let durr =
                SystemTime::now().duration_since(start).map_err(Box::new)?;
            if durr > timeout {
                error!("Socket timeout");
                break;
            }
            match stream.read(&mut buffer) {
                Ok(0) => {
                    if read_started {
                        debug!("read_started true, we have completed");
                        break;
                    } else {
                        debug!("Waiting ...");
                        continue;
                    }
                }
                Ok(count) => {
                    data.extend_from_slice(&buffer);
                    counter += count;
                    if count == 1024 {
                        debug!("Filled 1024 bytes, looping ...");
                        read_started = true;
                        continue;
                    } else {
                        debug!("Filled {} bytes, complete", count);
                        break;
                    }
                }
                Err(e) => {
                    error!("Stream read failure from {:?} -> {:?}", &stream, e);
                    return Err(Box::new(e));
                }
            }
        }

        data.truncate(counter);

        Ok(String::from_utf8(data)?)
    }
}

impl SessionBroker for HimmelblauSessionBroker {
    fn acquire_token_interactively(
        &mut self,
        protocol_version: String,
        correlation_id: String,
        request_json: String,
    ) -> Result<String, dbus::MethodErr> {
        self.request(ClientRequest::acquireTokenInteractively(
            protocol_version,
            correlation_id,
            request_json,
        ))
        .map_err(|e| dbus::MethodErr::failed(&e))
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
) -> Result<(), dbus::MethodErr> {
    session_broker_serve(HimmelblauSessionBroker {
        sock_path: sock_path.to_string(),
        timeout,
    })
    .await
}
