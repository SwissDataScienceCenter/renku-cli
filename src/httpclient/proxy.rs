use reqwest::ClientBuilder;
use reqwest::{Proxy, Result};

#[derive(Debug, Clone)]
pub enum ProxySetting {
    System,
    None,
    Custom {
        url: String,
        user: Option<String>,
        password: Option<String>,
    },
}

impl ProxySetting {
    pub fn set(&self, builder: ClientBuilder) -> Result<ClientBuilder> {
        match self {
            ProxySetting::System => {
                log::debug!("Using system proxy (no changes to client)");
                Ok(builder)
            }
            ProxySetting::None => {
                log::info!("Setting no_proxy");
                Ok(builder.no_proxy())
            }
            ProxySetting::Custom {
                url,
                user,
                password,
            } => {
                log::info!("Using proxy: {:?}", url);
                let mut p = Proxy::all(url)?;
                if let Some(login) = user {
                    log::debug!("Use proxy auth: {:?}/***", login);
                    let pass = match password {
                        Some(p) => p,
                        None => "",
                    };
                    p = p.basic_auth(login, pass);
                }
                Ok(builder.proxy(p))
            }
        }
    }
}
