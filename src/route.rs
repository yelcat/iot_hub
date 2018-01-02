use std::net::SocketAddr;
use std::sync::{Arc,RwLock};
use std::rc::Rc;
use std::char;

enum Segment {
    Asterisk,
    Greater,
    Normal(&'static str),
}

pub struct Subscriber {
    /**
     * Channel names, including reply subject (INBOX) names, are case-sensitive and must be non-empty alphanumeric strings with no embedded whitespace, and optionally token-delimited using the dot character (.), e.g.:
     * FOO, BAR, foo.bar, foo.BAR, FOO.BAR and FOO.BAR.BAZ are all valid subject names
     */
    channel_name: &'static str,
    socket_addr: SocketAddr,
}

type SubscriberRef = Rc<Subscriber>;
type SubscriberCollection = Arc<RwLock<Vec<SubscriberRef>>>;

struct RouteNode {
    segment: Segment,
    subscribers: SubscriberCollection,
    children: NodeCollection,
}

impl RouteNode {
    pub fn new(segment: Segment) -> RouteNode {
        RouteNode {
            segment,
            subscribers: Arc::new(RwLock::new(vec![])),
            children: Arc::new(RwLock::new(vec![])),
        }
    }
}

impl Subscriber {
    pub fn new(channel_name: &'static str, socket_addr: SocketAddr) -> SubscriberRef {
        Rc::new(Subscriber {
            channel_name,
            socket_addr,
        })
    }
}

impl PartialEq for Segment {
    fn eq(&self, other: &Segment) -> bool {
        use self::Segment::*;
        match (self, other) {
            (&Normal(ref c1), &Normal(ref c2)) => c1 == c2,
            (&Asterisk, &Asterisk) => true,
            (&Greater, &Greater) => true,
            _ => false,
        }
    }
}

type NodeCollection = Arc<RwLock<Vec<RouteNode>>>;

pub struct Router {
    children: NodeCollection,
}

impl Router {
    pub fn new() -> Router {
        Router {
            children: Arc::new(RwLock::new(vec![])),
        }
    }

    fn convert_segments(&self, channel_name: &'static str) -> Vec<Segment> {
        channel_name
            .split('.')
            .map(|s|
                 match s {
                     "*" => {
                         Segment::Asterisk
                     },
                     ">" => {
                         Segment::Greater
                     },
                     _ => {
                         Segment::Normal(s)
                     }
                 }
            )
            .collect()
    }

    pub fn add_subscriber(&self, subscriber: SubscriberRef) {
        let segments: Vec<Segment> = self.convert_segments(subscriber.clone().channel_name);
        for seg in segments {
            let children = self.children.clone();
            let subscriber_ref = subscriber.clone();
            let nodes = children.read().unwrap();

            let matched: Option<&RouteNode> = nodes.iter().filter(|n| n.segment == seg ).nth(1);
            if let Some(ref node) = matched {
                let subscriber_collection: SubscriberCollection = node.subscribers.clone();
                let mut subscribers = subscriber_collection.write().unwrap();
                subscribers.push(subscriber_ref);
            } else {
                let node = RouteNode::new(seg);
                let subscriber_collection: SubscriberCollection = node.subscribers.clone();
                let mut subscribers = subscriber_collection.write().unwrap();
                subscribers.push(subscriber_ref);

                let mut nodes = children.write().unwrap();
                nodes.push(node);
            }
        }
    }

    pub fn match_subscribers(&self, channel_name: &'static str) -> Vec<SubscriberRef> {
        let segments: Vec<Segment> = self.convert_segments(channel_name);
        for seg in segments {
            let children = self.children.clone();

        }
    }

}

#[cfg(test)]
mod tests {
    use route::*;
    
    #[test]
    fn test_add_subscriber_and_matched() {
        let router = Router::new();
        let addr: SocketAddr = "127.0.0.1:5014".parse().unwrap();
        router.add_subscriber(Subscriber::new("test.channel", addr));
        let subscribers: Vec<SubscriberRef> = router.match_subscribers("test.channel");
     }
}
