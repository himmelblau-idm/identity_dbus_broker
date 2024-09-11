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
use crate::freedesktop::get_peer_uid;
#[allow(unused_imports)]
use dbus::arg;
use dbus::blocking::Connection;
use dbus_crossroads as crossroads;
use libc::uid_t;

pub trait HimmelblauBroker {
    fn acquire_token_interactively(
        &mut self,
        protocol_version: String,
        correlation_id: String,
        request_json: String,
        uid: uid_t,
    ) -> Result<String, dbus::MethodErr>;
    fn acquire_token_silently(
        &mut self,
        protocol_version: String,
        correlation_id: String,
        request_json: String,
        uid: uid_t,
    ) -> Result<String, dbus::MethodErr>;
    fn get_accounts(
        &mut self,
        protocol_version: String,
        correlation_id: String,
        request_json: String,
        uid: uid_t,
    ) -> Result<String, dbus::MethodErr>;
    fn remove_account(
        &mut self,
        protocol_version: String,
        correlation_id: String,
        request_json: String,
        uid: uid_t,
    ) -> Result<String, dbus::MethodErr>;
    fn acquire_prt_sso_cookie(
        &mut self,
        protocol_version: String,
        correlation_id: String,
        request_json: String,
        uid: uid_t,
    ) -> Result<String, dbus::MethodErr>;
    fn generate_signed_http_request(
        &mut self,
        protocol_version: String,
        correlation_id: String,
        request_json: String,
        uid: uid_t,
    ) -> Result<String, dbus::MethodErr>;
    fn cancel_interactive_flow(
        &mut self,
        protocol_version: String,
        correlation_id: String,
        request_json: String,
        uid: uid_t,
    ) -> Result<String, dbus::MethodErr>;
    fn get_linux_broker_version(
        &mut self,
        protocol_version: String,
        correlation_id: String,
        request_json: String,
        uid: uid_t,
    ) -> Result<String, dbus::MethodErr>;
}

fn register_himmelblau_broker<T>(cr: &mut crossroads::Crossroads) -> crossroads::IfaceToken<T>
where
    T: HimmelblauBroker + Send + 'static,
{
    cr.register("org.samba.himmelblau", |b| {
        b.method(
            "acquireTokenInteractively",
            ("protocol_version", "correlation_id", "request_json"),
            ("result",),
            |ctx: &mut crossroads::Context,
             t: &mut T,
             (protocol_version, correlation_id, request_json)| {
                let sender = ctx
                    .message()
                    .sender()
                    .ok_or(dbus::MethodErr::failed("Sender not specified"))?;
                let uid = get_peer_uid(sender)?;
                t.acquire_token_interactively(protocol_version, correlation_id, request_json, uid)
                    .map(|x| (x,))
            },
        );
        b.method(
            "acquireTokenSilently",
            ("protocol_version", "correlation_id", "request_json"),
            ("result",),
            |ctx: &mut crossroads::Context,
             t: &mut T,
             (protocol_version, correlation_id, request_json)| {
                let sender = ctx
                    .message()
                    .sender()
                    .ok_or(dbus::MethodErr::failed("Sender not specified"))?;
                let uid = get_peer_uid(sender)?;
                t.acquire_token_silently(protocol_version, correlation_id, request_json, uid)
                    .map(|x| (x,))
            },
        );
        b.method(
            "getAccounts",
            ("protocol_version", "correlation_id", "request_json"),
            ("result",),
            |ctx: &mut crossroads::Context,
             t: &mut T,
             (protocol_version, correlation_id, request_json)| {
                let sender = ctx
                    .message()
                    .sender()
                    .ok_or(dbus::MethodErr::failed("Sender not specified"))?;
                let uid = get_peer_uid(sender)?;
                t.get_accounts(protocol_version, correlation_id, request_json, uid)
                    .map(|x| (x,))
            },
        );
        b.method(
            "removeAccount",
            ("protocol_version", "correlation_id", "request_json"),
            ("result",),
            |ctx: &mut crossroads::Context,
             t: &mut T,
             (protocol_version, correlation_id, request_json)| {
                let sender = ctx
                    .message()
                    .sender()
                    .ok_or(dbus::MethodErr::failed("Sender not specified"))?;
                let uid = get_peer_uid(sender)?;
                t.remove_account(protocol_version, correlation_id, request_json, uid)
                    .map(|x| (x,))
            },
        );
        b.method(
            "acquirePrtSsoCookie",
            ("protocol_version", "correlation_id", "request_json"),
            ("result",),
            |ctx: &mut crossroads::Context,
             t: &mut T,
             (protocol_version, correlation_id, request_json)| {
                let sender = ctx
                    .message()
                    .sender()
                    .ok_or(dbus::MethodErr::failed("Sender not specified"))?;
                let uid = get_peer_uid(sender)?;
                t.acquire_prt_sso_cookie(protocol_version, correlation_id, request_json, uid)
                    .map(|x| (x,))
            },
        );
        b.method(
            "generateSignedHttpRequest",
            ("protocol_version", "correlation_id", "request_json"),
            ("result",),
            |ctx: &mut crossroads::Context,
             t: &mut T,
             (protocol_version, correlation_id, request_json)| {
                let sender = ctx
                    .message()
                    .sender()
                    .ok_or(dbus::MethodErr::failed("Sender not specified"))?;
                let uid = get_peer_uid(sender)?;
                t.generate_signed_http_request(protocol_version, correlation_id, request_json, uid)
                    .map(|x| (x,))
            },
        );
        b.method(
            "cancelInteractiveFlow",
            ("protocol_version", "correlation_id", "request_json"),
            ("result",),
            |ctx: &mut crossroads::Context,
             t: &mut T,
             (protocol_version, correlation_id, request_json)| {
                let sender = ctx
                    .message()
                    .sender()
                    .ok_or(dbus::MethodErr::failed("Sender not specified"))?;
                let uid = get_peer_uid(sender)?;
                t.cancel_interactive_flow(protocol_version, correlation_id, request_json, uid)
                    .map(|x| (x,))
            },
        );
        b.method(
            "getLinuxBrokerVersion",
            ("protocol_version", "correlation_id", "request_json"),
            ("result",),
            |ctx: &mut crossroads::Context,
             t: &mut T,
             (protocol_version, correlation_id, request_json)| {
                let sender = ctx
                    .message()
                    .sender()
                    .ok_or(dbus::MethodErr::failed("Sender not specified"))?;
                let uid = get_peer_uid(sender)?;
                t.get_linux_broker_version(protocol_version, correlation_id, request_json, uid)
                    .map(|x| (x,))
            },
        );
    })
}

pub async fn himmelblau_broker_serve<T>(broker: T) -> Result<(), dbus::MethodErr>
where
    T: HimmelblauBroker + Send + 'static,
{
    // Start up a connection to the system bus and request a name
    let c = Connection::new_system()?;
    c.request_name("org.samba.himmelblau", false, true, false)?;

    let mut cr = crossroads::Crossroads::new();
    let token = register_himmelblau_broker::<T>(&mut cr);

    cr.insert("/org/samba/himmelblau", &[token], broker);

    // Serve clients forever.
    cr.serve(&c)?;
    unreachable!()
}
