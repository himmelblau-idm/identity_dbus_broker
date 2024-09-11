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
use dbus::strings::BusName;
use libc::uid_t;
use std::time::Duration;

#[allow(dead_code)]
trait DBus {
    fn hello(&self) -> Result<String, dbus::Error>;
    fn request_name(&self, arg0: &str, arg1: u32) -> Result<u32, dbus::Error>;
    fn release_name(&self, arg0: &str) -> Result<u32, dbus::Error>;
    fn start_service_by_name(&self, arg0: &str, arg1: u32) -> Result<u32, dbus::Error>;
    fn update_activation_environment(
        &self,
        arg0: ::std::collections::HashMap<&str, &str>,
    ) -> Result<(), dbus::Error>;
    fn name_has_owner(&self, arg0: &str) -> Result<bool, dbus::Error>;
    fn list_names(&self) -> Result<Vec<String>, dbus::Error>;
    fn list_activatable_names(&self) -> Result<Vec<String>, dbus::Error>;
    fn add_match(&self, arg0: &str) -> Result<(), dbus::Error>;
    fn remove_match(&self, arg0: &str) -> Result<(), dbus::Error>;
    fn get_name_owner(&self, arg0: &str) -> Result<String, dbus::Error>;
    fn list_queued_owners(&self, arg0: &str) -> Result<Vec<String>, dbus::Error>;
    fn get_connection_unix_user(&self, arg0: &str) -> Result<u32, dbus::Error>;
    fn get_connection_unix_process_id(&self, arg0: &str) -> Result<u32, dbus::Error>;
    fn get_adt_audit_session_data(&self, arg0: &str) -> Result<Vec<u8>, dbus::Error>;
    fn get_connection_selinux_security_context(&self, arg0: &str) -> Result<Vec<u8>, dbus::Error>;
    fn reload_config(&self) -> Result<(), dbus::Error>;
    fn get_id(&self) -> Result<String, dbus::Error>;
    fn get_connection_credentials(&self, arg0: &str) -> Result<arg::PropMap, dbus::Error>;
    fn features(&self) -> Result<Vec<String>, dbus::Error>;
    fn interfaces(&self) -> Result<Vec<String>, dbus::Error>;
}

#[derive(Debug)]
struct DBusNameOwnerChanged {
    arg0: String,
    arg1: String,
    arg2: String,
}

impl arg::AppendAll for DBusNameOwnerChanged {
    fn append(&self, i: &mut arg::IterAppend) {
        arg::RefArg::append(&self.arg0, i);
        arg::RefArg::append(&self.arg1, i);
        arg::RefArg::append(&self.arg2, i);
    }
}

impl arg::ReadAll for DBusNameOwnerChanged {
    fn read(i: &mut arg::Iter) -> Result<Self, arg::TypeMismatchError> {
        Ok(DBusNameOwnerChanged {
            arg0: i.read()?,
            arg1: i.read()?,
            arg2: i.read()?,
        })
    }
}

impl dbus::message::SignalArgs for DBusNameOwnerChanged {
    const NAME: &'static str = "NameOwnerChanged";
    const INTERFACE: &'static str = "org.freedesktop.DBus";
}

#[derive(Debug)]
struct DBusNameLost {
    arg0: String,
}

impl arg::AppendAll for DBusNameLost {
    fn append(&self, i: &mut arg::IterAppend) {
        arg::RefArg::append(&self.arg0, i);
    }
}

impl arg::ReadAll for DBusNameLost {
    fn read(i: &mut arg::Iter) -> Result<Self, arg::TypeMismatchError> {
        Ok(DBusNameLost { arg0: i.read()? })
    }
}

impl dbus::message::SignalArgs for DBusNameLost {
    const NAME: &'static str = "NameLost";
    const INTERFACE: &'static str = "org.freedesktop.DBus";
}

#[derive(Debug)]
struct DBusNameAcquired {
    arg0: String,
}

impl arg::AppendAll for DBusNameAcquired {
    fn append(&self, i: &mut arg::IterAppend) {
        arg::RefArg::append(&self.arg0, i);
    }
}

impl arg::ReadAll for DBusNameAcquired {
    fn read(i: &mut arg::Iter) -> Result<Self, arg::TypeMismatchError> {
        Ok(DBusNameAcquired { arg0: i.read()? })
    }
}

impl dbus::message::SignalArgs for DBusNameAcquired {
    const NAME: &'static str = "NameAcquired";
    const INTERFACE: &'static str = "org.freedesktop.DBus";
}

#[derive(Debug)]
struct DBusActivatableServicesChanged {}

impl arg::AppendAll for DBusActivatableServicesChanged {
    fn append(&self, _: &mut arg::IterAppend) {}
}

impl arg::ReadAll for DBusActivatableServicesChanged {
    fn read(_: &mut arg::Iter) -> Result<Self, arg::TypeMismatchError> {
        Ok(DBusActivatableServicesChanged {})
    }
}

impl dbus::message::SignalArgs for DBusActivatableServicesChanged {
    const NAME: &'static str = "ActivatableServicesChanged";
    const INTERFACE: &'static str = "org.freedesktop.DBus";
}

impl<'a, T: blocking::BlockingSender, C: ::std::ops::Deref<Target = T>> DBus
    for blocking::Proxy<'a, C>
{
    fn hello(&self) -> Result<String, dbus::Error> {
        self.method_call("org.freedesktop.DBus", "Hello", ())
            .map(|r: (String,)| r.0)
    }

    fn request_name(&self, arg0: &str, arg1: u32) -> Result<u32, dbus::Error> {
        self.method_call("org.freedesktop.DBus", "RequestName", (arg0, arg1))
            .map(|r: (u32,)| r.0)
    }

    fn release_name(&self, arg0: &str) -> Result<u32, dbus::Error> {
        self.method_call("org.freedesktop.DBus", "ReleaseName", (arg0,))
            .map(|r: (u32,)| r.0)
    }

    fn start_service_by_name(&self, arg0: &str, arg1: u32) -> Result<u32, dbus::Error> {
        self.method_call("org.freedesktop.DBus", "StartServiceByName", (arg0, arg1))
            .map(|r: (u32,)| r.0)
    }

    fn update_activation_environment(
        &self,
        arg0: ::std::collections::HashMap<&str, &str>,
    ) -> Result<(), dbus::Error> {
        self.method_call(
            "org.freedesktop.DBus",
            "UpdateActivationEnvironment",
            (arg0,),
        )
    }

    fn name_has_owner(&self, arg0: &str) -> Result<bool, dbus::Error> {
        self.method_call("org.freedesktop.DBus", "NameHasOwner", (arg0,))
            .map(|r: (bool,)| r.0)
    }

    fn list_names(&self) -> Result<Vec<String>, dbus::Error> {
        self.method_call("org.freedesktop.DBus", "ListNames", ())
            .map(|r: (Vec<String>,)| r.0)
    }

    fn list_activatable_names(&self) -> Result<Vec<String>, dbus::Error> {
        self.method_call("org.freedesktop.DBus", "ListActivatableNames", ())
            .map(|r: (Vec<String>,)| r.0)
    }

    fn add_match(&self, arg0: &str) -> Result<(), dbus::Error> {
        self.method_call("org.freedesktop.DBus", "AddMatch", (arg0,))
    }

    fn remove_match(&self, arg0: &str) -> Result<(), dbus::Error> {
        self.method_call("org.freedesktop.DBus", "RemoveMatch", (arg0,))
    }

    fn get_name_owner(&self, arg0: &str) -> Result<String, dbus::Error> {
        self.method_call("org.freedesktop.DBus", "GetNameOwner", (arg0,))
            .map(|r: (String,)| r.0)
    }

    fn list_queued_owners(&self, arg0: &str) -> Result<Vec<String>, dbus::Error> {
        self.method_call("org.freedesktop.DBus", "ListQueuedOwners", (arg0,))
            .map(|r: (Vec<String>,)| r.0)
    }

    fn get_connection_unix_user(&self, arg0: &str) -> Result<u32, dbus::Error> {
        self.method_call("org.freedesktop.DBus", "GetConnectionUnixUser", (arg0,))
            .map(|r: (u32,)| r.0)
    }

    fn get_connection_unix_process_id(&self, arg0: &str) -> Result<u32, dbus::Error> {
        self.method_call(
            "org.freedesktop.DBus",
            "GetConnectionUnixProcessID",
            (arg0,),
        )
        .map(|r: (u32,)| r.0)
    }

    fn get_adt_audit_session_data(&self, arg0: &str) -> Result<Vec<u8>, dbus::Error> {
        self.method_call("org.freedesktop.DBus", "GetAdtAuditSessionData", (arg0,))
            .map(|r: (Vec<u8>,)| r.0)
    }

    fn get_connection_selinux_security_context(&self, arg0: &str) -> Result<Vec<u8>, dbus::Error> {
        self.method_call(
            "org.freedesktop.DBus",
            "GetConnectionSELinuxSecurityContext",
            (arg0,),
        )
        .map(|r: (Vec<u8>,)| r.0)
    }

    fn reload_config(&self) -> Result<(), dbus::Error> {
        self.method_call("org.freedesktop.DBus", "ReloadConfig", ())
    }

    fn get_id(&self) -> Result<String, dbus::Error> {
        self.method_call("org.freedesktop.DBus", "GetId", ())
            .map(|r: (String,)| r.0)
    }

    fn get_connection_credentials(&self, arg0: &str) -> Result<arg::PropMap, dbus::Error> {
        self.method_call("org.freedesktop.DBus", "GetConnectionCredentials", (arg0,))
            .map(|r: (arg::PropMap,)| r.0)
    }

    fn features(&self) -> Result<Vec<String>, dbus::Error> {
        <Self as blocking::stdintf::org_freedesktop_dbus::Properties>::get(
            self,
            "org.freedesktop.DBus",
            "Features",
        )
    }

    fn interfaces(&self) -> Result<Vec<String>, dbus::Error> {
        <Self as blocking::stdintf::org_freedesktop_dbus::Properties>::get(
            self,
            "org.freedesktop.DBus",
            "Interfaces",
        )
    }
}

pub fn get_peer_uid(sender: BusName) -> Result<uid_t, dbus::Error> {
    let conn = Connection::new_system()?;
    let proxy = conn.with_proxy(
        "org.freedesktop.DBus",
        "/org/freedesktop/DBus",
        Duration::from_millis(5000),
    );
    proxy.get_connection_unix_user(&sender)
}
