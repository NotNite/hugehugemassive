use hcloud::{
    apis::{
        configuration::Configuration,
        servers_api::{
            create_server, delete_server, list_servers, CreateServerParams, DeleteServerParams,
            ListServersParams,
        },
    },
    models::CreateServerRequest,
};
use std::collections::HashMap;

pub use hcloud;

#[derive(thiserror::Error, Debug)]
pub enum Error {
    #[error("server not found")]
    ServerNotFound,

    #[error("hetzner API error")]
    HetznerApi,
}

#[derive(Debug, Clone)]
pub struct Hetzner {
    configuration: Configuration,
}

impl Hetzner {
    pub fn new(access_token: String) -> Self {
        let mut cfg = Configuration::new();
        cfg.bearer_access_token = Some(access_token);
        Self { configuration: cfg }
    }

    pub async fn create_server(
        &self,
        server_config: ServerConfig,
        name: String,
    ) -> Result<hcloud::models::Server, Error> {
        let mut req =
            CreateServerRequest::new(server_config.image, name, server_config.instance_type);
        req.location = Some(server_config.zone);
        req.ssh_keys = Some(server_config.ssh_keys);
        req.labels = server_config.labels;
        req.user_data = server_config.cloud_init;

        let params = CreateServerParams {
            create_server_request: Some(req),
        };

        let resp = create_server(&self.configuration, params)
            .await
            .map_err(|_| Error::HetznerApi)?;
        Ok(*resp.server)
    }

    pub async fn get_servers(&self) -> Result<Vec<hcloud::models::Server>, Error> {
        let params = ListServersParams {
            ..Default::default()
        };

        let resp = list_servers(&self.configuration, params)
            .await
            .map_err(|_| Error::HetznerApi)?;

        Ok(resp.servers)
    }

    pub async fn delete_server(&self, id: i32) -> Result<(), Error> {
        let params = DeleteServerParams { id };
        delete_server(&self.configuration, params)
            .await
            .map_err(|_| Error::HetznerApi)?;

        Ok(())
    }
}

#[derive(Debug, Clone)]
pub struct ServerConfig {
    pub image: String,
    pub instance_type: String,
    pub zone: String,
    pub ssh_keys: Vec<String>,
    pub cloud_init: Option<String>,
    pub labels: Option<HashMap<String, String>>,
}

impl Default for ServerConfig {
    fn default() -> Self {
        Self {
            image: "debian-11".to_string(),
            instance_type: "cpx11".to_string(),
            zone: "ash".to_string(),
            ssh_keys: vec![],
            cloud_init: None,
            labels: None,
        }
    }
}
