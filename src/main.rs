//#[macro_use]
//extern crate log;
//extern crate env_logger;
//extern crate coap;
extern crate futures;
extern crate futures_cpupool;

use futures::Future;
use futures_cpupool::Builder;

mod route;

use route::TopicRoute;

fn main() {
    let mut root = TopicRoute::new_root();
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

    // Create a worker thread pool with four threads
    let pool = Builder::new()
        .create();

    // Execute some work on the thread pool, optionally closing over data.
    let a = pool.spawn_fn(move || {
        let result: Result<_, ()> = Ok(2);

        result
    });
    let b = pool.spawn_fn(move || {
        let result: Result<_, ()> = Ok(100);

        result
    });

    // Express some further computation once the work is completed on the thread
    // pool.
    let c = a.join(b).map(|(a, b)| a + b).wait().unwrap();

    // Print out the result
    println!("{:?}", c);
}
