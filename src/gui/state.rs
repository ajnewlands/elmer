use url::Url;

pub struct GuiState {
    pub filter_string: String,
    pub filter_headers: bool,
    pub filter_body: bool,
    pub regex: Option<regex::Regex>,
    pub regex_error: Option<String>,

    pub connection_state: ConnectionState,
    pub connection_modal_state: Option<ConnectionState>,
}

impl Default for GuiState {
    fn default() -> Self {
        Self {
            filter_string: String::default(),
            filter_body: false,
            filter_headers: true,
            regex: None,
            regex_error: None,
            connection_state: ConnectionState::default(),
            connection_modal_state: Some(ConnectionState::default()),
        }
    }
}

impl GuiState {
    pub fn update_regex(&mut self) {
        self.regex_error = None;
        if self.filter_string.is_empty() {
            self.regex = None;
        } else {
            match regex::Regex::new(&self.filter_string) {
                Ok(regex) => {
                    self.regex = Some(regex);
                }
                Err(e) => {
                    self.regex = None;
                    self.regex_error = Some(e.to_string())
                }
            }
        }
    }
}

#[derive(Clone)]
pub struct ConnectionState {
    pub hostname: String,
    pub vhost: String,
    pub username: String,
    pub password: String,
    pub tls: bool,
}
impl Default for ConnectionState {
    fn default() -> Self {
        Self {
            hostname: "localhost:5762".into(),
            username: "guest".into(),
            password: String::default(),
            vhost: "/".into(),
            tls: true,
        }
    }
}

impl ConnectionState {
    pub fn build_url(&self) -> anyhow::Result<Url> {
        let proto = if self.tls { "amqps" } else { "amqp" };

        let mut url = Url::parse(&format!(
            "{}://{}:{}@{}/{}",
            proto, self.username, self.password, self.hostname, self.vhost,
        ))?;

        let normalized_path = url
            .path_segments()
            .map(|segments| {
                segments
                    .filter(|s| !s.is_empty())
                    .collect::<Vec<_>>()
                    .join("/")
            })
            .unwrap_or_else(|| "".to_string());

        url.set_path(&normalized_path);

        Ok(url)
    }
}
