use std::net::SocketAddr;
use std::sync::{Arc, RwLock};
use std::char;

#[derive(Debug)]
pub enum RouteError {
    RouteNotFound,
}

#[derive(Clone, Debug)]
enum Segment {
    Asterisk,
    Greater,
    Normal(&'static str),
}

impl Segment {
    fn matched(&self, other: &Segment) -> bool {
        use self::Segment::*;
        match (self, other) {
            (&Asterisk, _) => true,
            (&Greater, _) => true,
            _ => false,
        }
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

#[derive(Clone, Debug)]
pub struct Subscriber {
    /**
     * Channel names, including reply subject (INBOX) names, are case-sensitive and must be non-empty alphanumeric strings with no embedded whitespace, and optionally token-delimited using the dot character (.), e.g.:
     * FOO, BAR, foo.bar, foo.BAR, FOO.BAR and FOO.BAR.BAZ are all valid subject names
     */
    channel_name: &'static str,
    socket_addr: SocketAddr,
}

type SubscriberCollection = Arc<RwLock<Vec<Subscriber>>>;

#[derive(Clone, Debug)]
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
            .map(|s| match s {
                "*" => Segment::Asterisk,
                ">" => Segment::Greater,
                _ => Segment::Normal(s),
            })
            .collect()
    }

    fn find_or_create_node(&self, segment: Segment, children: NodeCollection) -> Option<RouteNode> {
        {
            let nodes = children.read().unwrap();
            let matched_node: Option<&RouteNode> = nodes
                .iter()
                .filter(|n| n.segment == segment)
                .nth(1);
            if let Some(node) = matched_node {
                return Some(node.clone());
            }
        } // block for release read lock

        let node = RouteNode::new(segment.clone());
        let node_cloned = node.clone();
        let mut nodes = children.write().unwrap();
        nodes.push(node);

        Some(node_cloned)
    }

    fn find_node(&self, segment: &Segment, children: NodeCollection) -> Vec<RouteNode> {
        let nodes = children.read().unwrap();
        nodes
            .iter()
            .filter(|n| {
                n.segment.matched(segment)
            })
            .map(|n| n.clone())
            .collect()
    }

    pub fn add_subscriber(&mut self, subscriber: Subscriber) {
        let mut node_matched: Option<RouteNode> = None;

        let segments = self.convert_segments(subscriber.channel_name);
        for segment in segments {
            if node_matched.is_none() {
                let children = self.children.clone();
                node_matched = self.find_or_create_node(segment, children);
            } else {
                let children = node_matched.unwrap().children;
                node_matched = self.find_or_create_node(segment, children);
            }
        }

        let subscriber_collection: SubscriberCollection = node_matched.unwrap().subscribers.clone();
        let mut subscribers = subscriber_collection.write().unwrap();
        subscribers.push(subscriber.clone());
    }

    pub fn match_subscribers(&self, channel_name: &'static str) -> Result<Vec<Subscriber>, RouteError> {
        let segments: Vec<Segment> = self.convert_segments(channel_name);
        let mut nodes_matched: Vec<RouteNode> = vec![];
        for segment in segments {
            if nodes_matched.len() == 0 {
                let children = self.children.clone();
                nodes_matched = self.find_node(&segment, children);
            } else {
                let mut next_nodes: Vec<RouteNode> = vec![];
                for route_node in nodes_matched {
                    let children = route_node.children;
                    next_nodes.extend(self.find_node(&segment, children));
                }
                nodes_matched = next_nodes;
            }

            if nodes_matched.len() == 0 {
                return Err(RouteError::RouteNotFound);
            }
        }

        let subscribers: Vec<Subscriber> = nodes_matched
            .into_iter()
            .flat_map(|node| {
                let subscriber_coll = node.subscribers.read().unwrap();
                let subscribers: Vec<Subscriber> =
                    subscriber_coll.iter().map(|sub| sub.clone()).collect();
                subscribers.into_iter()
            })
            .collect();
        Ok(subscribers)
    }
}

#[cfg(test)]
mod tests {
    use router::*;

    #[test]
    fn test_add_subscriber_and_matched() {
        let mut router = Router::new();

        router.add_subscriber(Subscriber {
            channel_name: "test.channel1",
            socket_addr: "127.0.0.1:5014".parse().unwrap(),
        });
        router.add_subscriber(Subscriber {
            channel_name: "test.*",
            socket_addr: "127.0.0.1:5015".parse().unwrap(),
        });
        let result = router.match_subscribers("test.channel1");
        assert!(result.is_ok());
        let subscribers = result.unwrap();
        assert_eq!(1, subscribers.len());
        let subscriber = &subscribers[0];
        assert_eq!("test.channel1", subscriber.channel_name);
    }
}
