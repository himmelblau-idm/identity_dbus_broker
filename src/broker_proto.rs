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

use serde::{Deserialize, Serialize};
use std::fmt;

/// Response from the daemon during an interactive authentication flow.
#[cfg(feature = "interactive")]
#[derive(Serialize, Deserialize)]
pub enum InteractiveAuthResponse {
    /// Prompt the user for their password.
    PromptPassword,
    /// Prompt the user for their Hello PIN.
    PromptPin,
    /// Prompt the user for an MFA verification code.
    PromptMFACode { msg: String },
    /// The user must approve an MFA request on another device; poll.
    PromptMFAPoll { msg: String, polling_interval: u32 },
    /// Prompt the user for FIDO2 authentication.
    PromptFido {
        fido_challenge: String,
        fido_allow_list: Vec<String>,
    },
    /// Authentication succeeded; contains the broker token response JSON.
    Success { token_response: String },
    /// Authentication was denied.
    Denied { msg: String },
}

#[cfg(feature = "interactive")]
impl std::fmt::Debug for InteractiveAuthResponse {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::PromptPassword => f.write_str("PromptPassword"),
            Self::PromptPin => f.write_str("PromptPin"),
            Self::PromptMFACode { msg } => {
                f.debug_struct("PromptMFACode").field("msg", msg).finish()
            }
            Self::PromptMFAPoll { msg, polling_interval } => {
                f.debug_struct("PromptMFAPoll")
                    .field("msg", msg)
                    .field("polling_interval", polling_interval)
                    .finish()
            }
            Self::PromptFido { .. } => f.write_str("PromptFido { .. }"),
            Self::Success { .. } => f.write_str("Success { token_response: [REDACTED] }"),
            Self::Denied { msg } => {
                f.debug_struct("Denied").field("msg", msg).finish()
            }
        }
    }
}

/// Credential submitted by the session broker during an interactive step.
#[cfg(feature = "interactive")]
#[derive(Serialize, Deserialize)]
pub enum InteractiveAuthCredential {
    Password { cred: String },
    Pin { cred: String },
    MFACode { cred: String },
    MFAPoll { poll_attempt: u32 },
    Fido { assertion: String },
}

#[cfg(feature = "interactive")]
impl std::fmt::Debug for InteractiveAuthCredential {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Password { .. } => f.write_str("Password { cred: [REDACTED] }"),
            Self::Pin { .. } => f.write_str("Pin { cred: [REDACTED] }"),
            Self::MFACode { .. } => f.write_str("MFACode { cred: [REDACTED] }"),
            Self::MFAPoll { poll_attempt } => {
                f.debug_struct("MFAPoll").field("poll_attempt", poll_attempt).finish()
            }
            Self::Fido { .. } => f.write_str("Fido { assertion: [REDACTED] }"),
        }
    }
}

#[allow(non_camel_case_types)]
#[derive(Serialize, Deserialize)]
pub enum ClientRequest {
    acquireTokenInteractively(String, String, String),
    acquireTokenSilently(String, String, String),
    getAccounts(String, String, String),
    removeAccount(String, String, String),
    acquirePrtSsoCookie(String, String, String),
    generateSignedHttpRequest(String, String, String),
    cancelInteractiveFlow(String, String, String),
    getLinuxBrokerVersion(String, String, String),
    /// Start an interactive auth session (protocol_version, correlation_id, request_json).
    #[cfg(feature = "interactive")]
    interactiveAuthInit(String, String, String),
    /// Submit a credential for an in-flight interactive session (correlation_id, credential_json).
    #[cfg(feature = "interactive")]
    interactiveAuthStep(String, String),
    /// Cancel an in-flight interactive session (correlation_id).
    #[cfg(feature = "interactive")]
    interactiveAuthCancel(String),
}

impl ClientRequest {
    pub fn method_name(&self) -> &'static str {
        match self {
            ClientRequest::acquireTokenInteractively(..) => "acquireTokenInteractively",
            ClientRequest::acquireTokenSilently(..) => "acquireTokenSilently",
            ClientRequest::getAccounts(..) => "getAccounts",
            ClientRequest::removeAccount(..) => "removeAccount",
            ClientRequest::acquirePrtSsoCookie(..) => "acquirePrtSsoCookie",
            ClientRequest::generateSignedHttpRequest(..) => "generateSignedHttpRequest",
            ClientRequest::cancelInteractiveFlow(..) => "cancelInteractiveFlow",
            ClientRequest::getLinuxBrokerVersion(..) => "getLinuxBrokerVersion",
            #[cfg(feature = "interactive")]
            ClientRequest::interactiveAuthInit(..) => "interactiveAuthInit",
            #[cfg(feature = "interactive")]
            ClientRequest::interactiveAuthStep(..) => "interactiveAuthStep",
            #[cfg(feature = "interactive")]
            ClientRequest::interactiveAuthCancel(..) => "interactiveAuthCancel",
        }
    }

    pub fn correlation_id(&self) -> &str {
        match self {
            ClientRequest::acquireTokenInteractively(_, cid, _)
            | ClientRequest::acquireTokenSilently(_, cid, _)
            | ClientRequest::getAccounts(_, cid, _)
            | ClientRequest::removeAccount(_, cid, _)
            | ClientRequest::acquirePrtSsoCookie(_, cid, _)
            | ClientRequest::generateSignedHttpRequest(_, cid, _)
            | ClientRequest::cancelInteractiveFlow(_, cid, _)
            | ClientRequest::getLinuxBrokerVersion(_, cid, _) => cid,
            #[cfg(feature = "interactive")]
            ClientRequest::interactiveAuthInit(_, cid, _) => cid,
            #[cfg(feature = "interactive")]
            ClientRequest::interactiveAuthStep(cid, _) => cid,
            #[cfg(feature = "interactive")]
            ClientRequest::interactiveAuthCancel(cid) => cid,
        }
    }
}

impl fmt::Display for ClientRequest {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}(correlation_id={})", self.method_name(), self.correlation_id())
    }
}
