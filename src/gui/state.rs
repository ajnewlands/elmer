#[derive(PartialEq, Eq)]
pub enum ConnectionStatus {
    Disconnected,
    Connecting,
    Connected,
}

pub struct FilterState {
    pub filter_string: String,
    pub filter_headers: bool,
    pub filter_body: bool,
    pub regex: Option<regex::Regex>,
    pub regex_error: Option<String>,
}

impl Default for FilterState {
    fn default() -> Self {
        Self {
            filter_string: String::default(),
            filter_body: false,
            filter_headers: true,
            regex: None,
            regex_error: None,
        }
    }
}

pub struct GuiState {
    pub filter_state: FilterState,
    pub connection_parameters: ConnectionParams,
    pub connection_modal_parameters: Option<ConnectionParams>,
    pub connection: ConnectionStatus,
    pub show_subscriptions: bool,
}

impl Default for GuiState {
    fn default() -> Self {
        Self {
            filter_state: FilterState::default(),
            connection_parameters: ConnectionParams::default(),
            connection_modal_parameters: Some(ConnectionParams::default()),
            connection: ConnectionStatus::Disconnected,
            show_subscriptions: false,
        }
    }
}

impl GuiState {
    pub fn update_regex(&mut self) {
        self.filter_state.regex_error = None;
        if self.filter_state.filter_string.is_empty() {
            self.filter_state.regex = None;
        } else {
            match regex::Regex::new(&self.filter_state.filter_string) {
                Ok(regex) => {
                    self.filter_state.regex = Some(regex);
                }
                Err(e) => {
                    self.filter_state.regex = None;
                    self.filter_state.regex_error = Some(e.to_string())
                }
            }
        }
    }
}

#[derive(Clone)]
pub struct ConnectionParams {
    pub hostname: String,
    pub vhost: String,
    pub exchange: String,
    pub username: String,
    pub password: String,
    pub tls: bool,
    /// Create a default unqualified (i.e. to everything) subscription?
    pub wildcard: bool,
    /// Usually 5672
    pub port: String,
    pub validation_error: Option<String>, // compute it here to avoid repeated recomputes in immediate mode.
}
impl Default for ConnectionParams {
    fn default() -> Self {
        Self {
            hostname: "localhost".into(),
            username: "guest".into(),
            password: String::default(),
            vhost: "/".into(),
            tls: true,
            port: "5672".into(),
            validation_error: None,
            wildcard: true,
            exchange: String::default(),
        }
    }
}

use lapin::uri;
impl ConnectionParams {
    pub fn build_url(&self) -> uri::AMQPUri {
        let scheme: uri::AMQPScheme = if self.tls {
            uri::AMQPScheme::AMQPS
        } else {
            uri::AMQPScheme::AMQP
        };

        let userinfo = uri::AMQPUserInfo {
            username: self.username.clone(),
            password: self.password.clone(),
        };

        let authority = uri::AMQPAuthority {
            userinfo,
            host: self.hostname.clone(),
            port: self.port.parse().expect("Port not u16"), // Should be ensured by form validation
        };

        let query = uri::AMQPQueryString::default();

        let uri = uri::AMQPUri {
            scheme,
            authority,
            vhost: self.vhost.clone(),
            query,
        };

        uri
    }

    pub fn validate(&mut self) {
        match self.port.parse::<u16>() {
            Ok(_) => (),
            Err(_) => {
                self.validation_error = Some("Port must be a valid integer < 65535".into());
                return;
            }
        }
        self.validation_error = None;
    }
}
