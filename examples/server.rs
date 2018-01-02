extern crate iot_hub;

use iot_hub::server::Server;


fn main() {
    Server::new("127.0.0.1:5514");
}
