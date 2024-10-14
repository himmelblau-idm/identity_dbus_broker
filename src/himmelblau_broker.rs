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
use async_trait::async_trait;
use bytes::{BufMut, BytesMut};
use futures::{SinkExt, StreamExt};
use libc::{uid_t, umask};
use std::error::Error;
use std::io;
use tokio::net::{UnixListener, UnixStream};
use tokio::sync::broadcast::Receiver;
use tokio::task::JoinHandle;
use tokio_util::codec::{Decoder, Encoder, Framed};
use tracing::{debug, error, trace};

#[async_trait]
pub trait HimmelblauBroker {
    async fn acquire_token_interactively(
        &mut self,
        protocol_version: String,
        correlation_id: String,
        request_json: String,
        uid: uid_t,
    ) -> Result<String, Box<dyn Error>>;
    async fn acquire_token_silently(
        &mut self,
        protocol_version: String,
        correlation_id: String,
        request_json: String,
        uid: uid_t,
    ) -> Result<String, Box<dyn Error>>;
    async fn get_accounts(
        &mut self,
        protocol_version: String,
        correlation_id: String,
        request_json: String,
        uid: uid_t,
    ) -> Result<String, Box<dyn Error>>;
    async fn remove_account(
        &mut self,
        protocol_version: String,
        correlation_id: String,
        request_json: String,
        uid: uid_t,
    ) -> Result<String, Box<dyn Error>>;
    async fn acquire_prt_sso_cookie(
        &mut self,
        protocol_version: String,
        correlation_id: String,
        request_json: String,
        uid: uid_t,
    ) -> Result<String, Box<dyn Error>>;
    async fn generate_signed_http_request(
        &mut self,
        protocol_version: String,
        correlation_id: String,
        request_json: String,
        uid: uid_t,
    ) -> Result<String, Box<dyn Error>>;
    async fn cancel_interactive_flow(
        &mut self,
        protocol_version: String,
        correlation_id: String,
        request_json: String,
        uid: uid_t,
    ) -> Result<String, Box<dyn Error>>;
    async fn get_linux_broker_version(
        &mut self,
        protocol_version: String,
        correlation_id: String,
        request_json: String,
        uid: uid_t,
    ) -> Result<String, Box<dyn Error>>;
}

#[derive(Default)]
struct ClientCodec;

impl Decoder for ClientCodec {
    type Error = io::Error;
    type Item = ClientRequest;

    fn decode(
        &mut self,
        src: &mut BytesMut,
    ) -> Result<Option<Self::Item>, Self::Error> {
        trace!("Attempting to decode request ...");
        match serde_json::from_slice::<ClientRequest>(src) {
            Ok(msg) => {
                // Clear the buffer for the next message.
                src.clear();
                Ok(Some(msg))
            }
            _ => Ok(None),
        }
    }
}

impl Encoder<String> for ClientCodec {
    type Error = io::Error;

    fn encode(
        &mut self,
        msg: String,
        dst: &mut BytesMut,
    ) -> Result<(), Self::Error> {
        dst.put(msg.as_bytes());
        Ok(())
    }
}

async fn handle_request<T>(
    sock: UnixStream,
    mut broker: T,
) -> Result<(), Box<dyn Error>>
where
    T: HimmelblauBroker + Send + 'static + Clone,
{
    let cred = sock.peer_cred().map_err(|e| {
        error!("Unable to verify peer credentials: {:?}", e);
        Box::new(e)
    })?;
    let uid = cred.uid();

    let mut reqs = Framed::new(sock, ClientCodec);

    while let Some(Ok(req)) = reqs.next().await {
        let resp = match req {
            ClientRequest::acquireTokenInteractively(
                protocol_version,
                correlation_id,
                request_json,
            ) => {
                broker
                    .acquire_token_interactively(
                        protocol_version,
                        correlation_id,
                        request_json,
                        uid,
                    )
                    .await?
            }
            ClientRequest::acquireTokenSilently(
                protocol_version,
                correlation_id,
                request_json,
            ) => {
                broker
                    .acquire_token_silently(
                        protocol_version,
                        correlation_id,
                        request_json,
                        uid,
                    )
                    .await?
            }
            ClientRequest::getAccounts(
                protocol_version,
                correlation_id,
                request_json,
            ) => {
                broker
                    .get_accounts(
                        protocol_version,
                        correlation_id,
                        request_json,
                        uid,
                    )
                    .await?
            }
            ClientRequest::removeAccount(
                protocol_version,
                correlation_id,
                request_json,
            ) => {
                broker
                    .remove_account(
                        protocol_version,
                        correlation_id,
                        request_json,
                        uid,
                    )
                    .await?
            }
            ClientRequest::acquirePrtSsoCookie(
                protocol_version,
                correlation_id,
                request_json,
            ) => {
                broker
                    .acquire_prt_sso_cookie(
                        protocol_version,
                        correlation_id,
                        request_json,
                        uid,
                    )
                    .await?
            }
            ClientRequest::generateSignedHttpRequest(
                protocol_version,
                correlation_id,
                request_json,
            ) => {
                broker
                    .generate_signed_http_request(
                        protocol_version,
                        correlation_id,
                        request_json,
                        uid,
                    )
                    .await?
            }
            ClientRequest::cancelInteractiveFlow(
                protocol_version,
                correlation_id,
                request_json,
            ) => {
                broker
                    .cancel_interactive_flow(
                        protocol_version,
                        correlation_id,
                        request_json,
                        uid,
                    )
                    .await?
            }
            ClientRequest::getLinuxBrokerVersion(
                protocol_version,
                correlation_id,
                request_json,
            ) => {
                broker
                    .get_linux_broker_version(
                        protocol_version,
                        correlation_id,
                        request_json,
                        uid,
                    )
                    .await?
            }
        };
        reqs.send(resp).await?;
        reqs.flush().await?;
        debug!("flushed response!");
    }

    debug!("Disconnecting client ...");
    Ok(())
}

pub async fn himmelblau_broker_serve<T>(
    broker: T,
    sock_path: &str,
    mut broadcast_rx: Receiver<bool>,
) -> Result<JoinHandle<()>, Box<dyn Error>>
where
    T: HimmelblauBroker + Send + 'static + Clone,
{
    // Set the umask while we open the path for most clients.
    let before = unsafe { umask(0) };
    let listener = UnixListener::bind(sock_path).map_err(|e| {
        error!("Failed to bind UNIX socket at {}", sock_path);
        Box::new(e)
    })?;
    // Undo umask changes.
    let _ = unsafe { umask(before) };

    Ok(tokio::spawn(async move {
        loop {
            tokio::select! {
                _ = broadcast_rx.recv() => {
                    break;
                }
                accept_res = listener.accept() => {
                    match accept_res {
                        Ok((socket, _addr)) => {
                            let broker_ref = broker.clone();
                            tokio::spawn(async move {
                                if let Err(e) = handle_request(socket, broker_ref.clone()).await {
                                    error!("handle_request error occurred; error = {:?}", e);
                                }
                            });
                        }
                        Err(e) => {
                            error!("Error while handling connection -> {:?}", e);
                        }
                    }
                }
            }
        }
    }))
}
