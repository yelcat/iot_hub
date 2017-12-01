//#[macro_use]
//extern crate log;
//extern crate env_logger;
//extern crate coap;

mod iot_hub;

use iot_hub::TopicNode;

fn main() {
    let mut root = TopicNode::new_root();
    root.add_topic("floor1.device11.sensor1");
    root.add_topic("floor1.device11.sensor2");
    root.add_topic("floor2.device21.sensor1");
    root.add_topic("floor2.device21.sensor2");

    root.subscribe("floor2.device21.sensor1", "sensor1subscriber");
    root.subscribe("floor2.device21.sensor2", "sensor2subscriber");
    root.send("floor2.device21.*", "Hello Subscriber!");
    root.unsubscribe("floor2.device21.sensor1", "sensor1subscriber");
    root.send("floor2.device21.*", "Hello Subscriber!");

    println!("Over!");
}
