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
#[allow(unused_imports)]
use dbus::arg;
use dbus::blocking;
use dbus::blocking::Connection;
use dbus_crossroads as crossroads;
use std::time::Duration;

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

fn register_session_broker<T>(cr: &mut crossroads::Crossroads) -> crossroads::IfaceToken<T>
where
    T: SessionBroker + Send + 'static,
{
    cr.register("com.microsoft.identity.Broker1", |b| {
        b.method(
            "acquireTokenInteractively",
            ("protocol_version", "correlation_id", "request_json"),
            ("result",),
            |_, t: &mut T, (protocol_version, correlation_id, request_json)| {
                t.acquire_token_interactively(protocol_version, correlation_id, request_json)
                    .map(|x| (x,))
            },
        );
        b.method(
            "acquireTokenSilently",
            ("protocol_version", "correlation_id", "request_json"),
            ("result",),
            |_, t: &mut T, (protocol_version, correlation_id, request_json)| {
                t.acquire_token_silently(protocol_version, correlation_id, request_json)
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
                t.acquire_prt_sso_cookie(protocol_version, correlation_id, request_json)
                    .map(|x| (x,))
            },
        );
        b.method(
            "generateSignedHttpRequest",
            ("protocol_version", "correlation_id", "request_json"),
            ("result",),
            |_, t: &mut T, (protocol_version, correlation_id, request_json)| {
                t.generate_signed_http_request(protocol_version, correlation_id, request_json)
                    .map(|x| (x,))
            },
        );
        b.method(
            "cancelInteractiveFlow",
            ("protocol_version", "correlation_id", "request_json"),
            ("result",),
            |_, t: &mut T, (protocol_version, correlation_id, request_json)| {
                t.cancel_interactive_flow(protocol_version, correlation_id, request_json)
                    .map(|x| (x,))
            },
        );
        b.method(
            "getLinuxBrokerVersion",
            ("protocol_version", "correlation_id", "request_json"),
            ("result",),
            |_, t: &mut T, (protocol_version, correlation_id, request_json)| {
                t.get_linux_broker_version(protocol_version, correlation_id, request_json)
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

    cr.insert("/com/microsoft/identity/broker1", &[token], broker);

    // Serve clients forever.
    cr.serve(&c)?;
    unreachable!()
}

trait HimmelblauBroker {
    fn acquire_token_interactively(
        &mut self,
        protocol_version: String,
        correlation_id: String,
        request_json: String,
    ) -> Result<String, dbus::Error>;
    fn acquire_token_silently(
        &mut self,
        protocol_version: String,
        correlation_id: String,
        request_json: String,
    ) -> Result<String, dbus::Error>;
    fn get_accounts(
        &mut self,
        protocol_version: String,
        correlation_id: String,
        request_json: String,
    ) -> Result<String, dbus::Error>;
    fn remove_account(
        &mut self,
        protocol_version: String,
        correlation_id: String,
        request_json: String,
    ) -> Result<String, dbus::Error>;
    fn acquire_prt_sso_cookie(
        &mut self,
        protocol_version: String,
        correlation_id: String,
        request_json: String,
    ) -> Result<String, dbus::Error>;
    fn generate_signed_http_request(
        &mut self,
        protocol_version: String,
        correlation_id: String,
        request_json: String,
    ) -> Result<String, dbus::Error>;
    fn cancel_interactive_flow(
        &mut self,
        protocol_version: String,
        correlation_id: String,
        request_json: String,
    ) -> Result<String, dbus::Error>;
    fn get_linux_broker_version(
        &mut self,
        protocol_version: String,
        correlation_id: String,
        request_json: String,
    ) -> Result<String, dbus::Error>;
}

impl<'a, T: blocking::BlockingSender, C: ::std::ops::Deref<Target = T>> HimmelblauBroker
    for blocking::Proxy<'a, C>
{
    fn acquire_token_interactively(
        &mut self,
        protocol_version: String,
        correlation_id: String,
        request_json: String,
    ) -> Result<String, dbus::Error> {
        self.method_call(
            "org.samba.himmelblau",
            "acquireTokenInteractively",
            (protocol_version, correlation_id, request_json),
        )
        .map(|r: (String,)| r.0)
    }

    fn acquire_token_silently(
        &mut self,
        protocol_version: String,
        correlation_id: String,
        request_json: String,
    ) -> Result<String, dbus::Error> {
        self.method_call(
            "org.samba.himmelblau",
            "acquireTokenSilently",
            (protocol_version, correlation_id, request_json),
        )
        .map(|r: (String,)| r.0)
    }

    fn get_accounts(
        &mut self,
        protocol_version: String,
        correlation_id: String,
        request_json: String,
    ) -> Result<String, dbus::Error> {
        self.method_call(
            "org.samba.himmelblau",
            "getAccounts",
            (protocol_version, correlation_id, request_json),
        )
        .map(|r: (String,)| r.0)
    }

    fn remove_account(
        &mut self,
        protocol_version: String,
        correlation_id: String,
        request_json: String,
    ) -> Result<String, dbus::Error> {
        self.method_call(
            "org.samba.himmelblau",
            "removeAccount",
            (protocol_version, correlation_id, request_json),
        )
        .map(|r: (String,)| r.0)
    }

    fn acquire_prt_sso_cookie(
        &mut self,
        protocol_version: String,
        correlation_id: String,
        request_json: String,
    ) -> Result<String, dbus::Error> {
        self.method_call(
            "org.samba.himmelblau",
            "acquirePrtSsoCookie",
            (protocol_version, correlation_id, request_json),
        )
        .map(|r: (String,)| r.0)
    }

    fn generate_signed_http_request(
        &mut self,
        protocol_version: String,
        correlation_id: String,
        request_json: String,
    ) -> Result<String, dbus::Error> {
        self.method_call(
            "org.samba.himmelblau",
            "generateSignedHttpRequest",
            (protocol_version, correlation_id, request_json),
        )
        .map(|r: (String,)| r.0)
    }

    fn cancel_interactive_flow(
        &mut self,
        protocol_version: String,
        correlation_id: String,
        request_json: String,
    ) -> Result<String, dbus::Error> {
        self.method_call(
            "org.samba.himmelblau",
            "cancelInteractiveFlow",
            (protocol_version, correlation_id, request_json),
        )
        .map(|r: (String,)| r.0)
    }

    fn get_linux_broker_version(
        &mut self,
        protocol_version: String,
        correlation_id: String,
        request_json: String,
    ) -> Result<String, dbus::Error> {
        self.method_call(
            "org.samba.himmelblau",
            "getLinuxBrokerVersion",
            (protocol_version, correlation_id, request_json),
        )
        .map(|r: (String,)| r.0)
    }
}

struct HimmelblauSessionBroker;

impl SessionBroker for HimmelblauSessionBroker {
    fn acquire_token_interactively(
        &mut self,
        protocol_version: String,
        correlation_id: String,
        request_json: String,
    ) -> Result<String, dbus::MethodErr> {
        let conn = Connection::new_system()?;
        let mut proxy = conn.with_proxy(
            "org.samba.himmelblau",
            "/org/samba/himmelblau",
            Duration::from_millis(5000),
        );
        Ok(proxy.acquire_token_interactively(protocol_version, correlation_id, request_json)?)
    }

    fn acquire_token_silently(
        &mut self,
        protocol_version: String,
        correlation_id: String,
        request_json: String,
    ) -> Result<String, dbus::MethodErr> {
        let conn = Connection::new_system()?;
        let mut proxy = conn.with_proxy(
            "org.samba.himmelblau",
            "/org/samba/himmelblau",
            Duration::from_millis(5000),
        );
        Ok(proxy.acquire_token_silently(protocol_version, correlation_id, request_json)?)
    }

    fn get_accounts(
        &mut self,
        protocol_version: String,
        correlation_id: String,
        request_json: String,
    ) -> Result<String, dbus::MethodErr> {
        let conn = Connection::new_system()?;
        let mut proxy = conn.with_proxy(
            "org.samba.himmelblau",
            "/org/samba/himmelblau",
            Duration::from_millis(5000),
        );
        Ok(proxy.get_accounts(protocol_version, correlation_id, request_json)?)
    }

    fn remove_account(
        &mut self,
        protocol_version: String,
        correlation_id: String,
        request_json: String,
    ) -> Result<String, dbus::MethodErr> {
        let conn = Connection::new_system()?;
        let mut proxy = conn.with_proxy(
            "org.samba.himmelblau",
            "/org/samba/himmelblau",
            Duration::from_millis(5000),
        );
        Ok(proxy.remove_account(protocol_version, correlation_id, request_json)?)
    }

    fn acquire_prt_sso_cookie(
        &mut self,
        protocol_version: String,
        correlation_id: String,
        request_json: String,
    ) -> Result<String, dbus::MethodErr> {
        let conn = Connection::new_system()?;
        let mut proxy = conn.with_proxy(
            "org.samba.himmelblau",
            "/org/samba/himmelblau",
            Duration::from_millis(5000),
        );
        Ok(proxy.acquire_prt_sso_cookie(protocol_version, correlation_id, request_json)?)
    }

    fn generate_signed_http_request(
        &mut self,
        protocol_version: String,
        correlation_id: String,
        request_json: String,
    ) -> Result<String, dbus::MethodErr> {
        let conn = Connection::new_system()?;
        let mut proxy = conn.with_proxy(
            "org.samba.himmelblau",
            "/org/samba/himmelblau",
            Duration::from_millis(5000),
        );
        Ok(proxy.generate_signed_http_request(protocol_version, correlation_id, request_json)?)
    }

    fn cancel_interactive_flow(
        &mut self,
        protocol_version: String,
        correlation_id: String,
        request_json: String,
    ) -> Result<String, dbus::MethodErr> {
        let conn = Connection::new_system()?;
        let mut proxy = conn.with_proxy(
            "org.samba.himmelblau",
            "/org/samba/himmelblau",
            Duration::from_millis(5000),
        );
        Ok(proxy.cancel_interactive_flow(protocol_version, correlation_id, request_json)?)
    }

    fn get_linux_broker_version(
        &mut self,
        protocol_version: String,
        correlation_id: String,
        request_json: String,
    ) -> Result<String, dbus::MethodErr> {
        let conn = Connection::new_system()?;
        let mut proxy = conn.with_proxy(
            "org.samba.himmelblau",
            "/org/samba/himmelblau",
            Duration::from_millis(5000),
        );
        Ok(proxy.get_linux_broker_version(protocol_version, correlation_id, request_json)?)
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
pub async fn himmelblau_session_broker_serve() -> Result<(), dbus::MethodErr> {
    session_broker_serve(HimmelblauSessionBroker {}).await
}
