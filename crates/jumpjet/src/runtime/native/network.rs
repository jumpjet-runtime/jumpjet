use std::net::SocketAddr;

use wasmtime::component::Resource;
use wasmtime::Result;
use web_transport::{Client, ClientBuilder, Server, quinn::ServerBuilder};

use crate::jumpjet::runtime::network::*;
use super::state::JumpjetRuntimeState;

impl Host for JumpjetRuntimeState {
    async fn client(&mut self, config: NetworkClientConfig,) -> Resource<NetworkClient> {
        let client = ClientBuilder::new().with_system_roots().unwrap();
        self.table.push(client).unwrap()
    }

    async fn server(&mut self, config: NetworkServerConfig,) -> Resource<NetworkServer> {
        // let server = ServerBuilder::new().with_addr(SocketAddr::from(([0, 0, 0, 0], 443)))
        //     .with_certificate(chain, key);
        self.table.push(server).unwrap()
    }

    async fn http_client(&mut self) -> Resource<NetworkHttpClient> {
        let client = reqwest::Client::new();
        self.table.push(client).unwrap()
    }
}

impl HostNetworkClient for JumpjetRuntimeState {
    async fn connect(&mut self, client: Resource<NetworkClient>, endpoint: String) -> Resource<NetworkConnection> {
        let client = self.table.get(&client).unwrap();
        self.table.push(client.connect(endpoint).await.unwrap()).unwrap()
    }

    async fn drop(&mut self, rep: Resource<NetworkClient>) -> Result<()> {
        self.table.delete(rep);
        Ok(())
    }
}

impl HostNetworkServer for JumpjetRuntimeState {
    async fn accept(&mut self, server: Resource<NetworkServer>) -> Resource<NetworkConnection> {
        let server = self.table.get(&server).unwrap();
        let incoming_session = server.accept().await;
        let incoming_request = incoming_session.await.unwrap();
        let connection = incoming_request.accept().await.unwrap();
        self.table.push(connection).unwrap()
    }

    async fn drop(&mut self, rep: Resource<NetworkServer>) -> Result<()> {
        self.table.delete(rep);
        Ok(())
    }
}

impl HostNetworkHttpClient for JumpjetRuntimeState {
    async fn request(&mut self, client: Resource<NetworkHttpClient>, req: HttpRequest) -> Result<HttpResponse, NetError> {
        let client = self.table.get(&client).unwrap();
        let mut request = client.request(req.method.into(), req.url);
        if req.body.is_some() {
            request = request.body(req.body.unwrap());
        }
        let response = request.send().await.unwrap();

        Ok(HttpResponse {
            status: response.status().as_u16(),
            headers: response.headers().iter().map(|(name, value)| HttpHeader {
                name: name.as_str().to_owned(),
                value: value.to_str().unwrap().to_owned()
            }).collect(),
            body: response.bytes().await.unwrap().to_vec(),
        })
    }

    async fn drop(&mut self, rep: Resource<NetworkHttpClient>) -> Result<()> {
        self.table.delete(rep);
        Ok(())
    }
}

impl HostNetworkConnection for JumpjetRuntimeState {
    async fn send(&mut self, connection: Resource<NetworkConnection>, data: Vec<u8>) -> Result<(), NetError> {
        let connection = self.table.get(&connection).unwrap();
        connection.send_datagram(data).unwrap();
        Ok(())
    }
    
    async fn receive(&mut self, connection: Resource<NetworkConnection>, max_bytes: u32,) -> Result<Vec<u8>, NetError> {
        let connection = self.table.get(&connection).unwrap();
        let datagram = connection.receive_datagram().await.unwrap();
        Ok(datagram.to_vec())
    }
    
    async fn close(&mut self, connection: Resource<NetworkConnection>) -> () {
        let connection = self.table.get(&connection).unwrap();
        connection.close(VarInt::from_u32(0), b"Closed");
    }

    async fn drop(&mut self, rep: Resource<NetworkConnection>) -> Result<()> {
        self.table.delete(rep).unwrap();
        Ok(())
    }
}
