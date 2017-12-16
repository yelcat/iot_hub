use std::collections::HashMap;
use std::collections::hash_map::Entry::{Occupied, Vacant};
use std::sync::mpsc::{channel, Sender};
use std::thread::spawn;
use std::rc::Rc;
use std::cell::RefCell;

//use coap::{CoAPClient, CoAPRequest, IsMessage, MessageType, CoAPOption};

use self::TopicRoute::*;
use self::ProcessAction::*;

const SPLITTER: &str = ".";

struct Subscriber {
    subscriber_id: String,
}

impl Subscriber {
    pub fn send(&self, message: &String) {
        println!("{}: {}", self.subscriber_id, message);
    }
}

impl PartialEq for Subscriber {
    fn eq(&self, other: &Subscriber) -> bool {
        self.subscriber_id == other.subscriber_id
    }

    fn ne(&self, other: &Subscriber) -> bool {
        !self.eq(other)
    }
}

pub enum ProcessAction {
    Subscribe(String),
    UnSubscribe(String),
    SendMessage(String),
}

impl Clone for ProcessAction {
    fn clone(&self) -> Self {
        match self {
            &Subscribe(ref subscriber_id) => {
                Subscribe(subscriber_id.clone())
            }
            &UnSubscribe(ref subscriber_id) => {
                UnSubscribe(subscriber_id.clone())
            }
            &SendMessage(ref message) => {
                SendMessage(message.clone())
            }
        }
    }
}

type TopicRef = Rc<RefCell<TopicRoute>>;

pub enum TopicRoute {
    Root(HashMap<String, TopicRef>),
    Leaf(String, Sender<ProcessAction>, HashMap<String, TopicRef>),
}

impl TopicRoute {
    pub fn new_root() -> TopicRoute {
        TopicRoute::Root(HashMap::new())
    }
    
    pub fn new(topic_name: String) -> TopicRef {
        let (sender, receiver) = channel();

        spawn(move || {
            let mut subscribers: Vec<Subscriber> = Vec::new();
            loop {
                if let Ok(action) = receiver.recv() {
                    match action {
                        Subscribe(subscriber_id) => {
                            subscribers.push(Subscriber { subscriber_id })
                        }

                        UnSubscribe(subscriber_id) => {
                            subscribers.retain(|subscriber| {
                                subscriber.subscriber_id.eq(&subscriber_id)
                            });
                        }

                        SendMessage(message) => {
                            for subscriber in &subscribers {
                                subscriber.send(&message)
                            }
                        }
                    }
                }
            }
        });

        Rc::new(RefCell::new(TopicRoute::Leaf(topic_name, sender, HashMap::new())))
    }

    pub fn add_topic(&mut self, topic_name: &str) {
        if topic_name.contains('*') {
            panic!("Invalid topic name, which contains '*'");
        }

        let topic_paths: Vec<_> = topic_name.split(SPLITTER).collect();
        self.add_topic_by_path(topic_paths, 0);
    }

    pub fn find_topics(&self, topic_pattern: &str) -> Vec<TopicRef> {
        let topic_paths: Vec<_> = topic_pattern.split(SPLITTER).collect();
        self.find_topics_by_path(topic_paths, 0)
    }

    pub fn subscribe(&self, topic_pattern: &str, subscribe_id: &str) {
        self.process_action(topic_pattern, Subscribe(subscribe_id.to_string()));
    }

    pub fn unsubscribe(&self, topic_pattern: &str, subscribe_id: &str) {
        self.process_action(topic_pattern, UnSubscribe(subscribe_id.to_string()));
    }

    pub fn send(&self, topic_pattern: &str, message: &str) {
        self.process_action(topic_pattern, SendMessage(message.to_string()));
    }
    
    fn process_action(&self, topic_pattern: &str, action: ProcessAction) {
        match self {
            &Root(_) => {
                let topic_refs = self.find_topics(topic_pattern);
                for topic in topic_refs {
                    topic.borrow().process_action(topic_pattern, action.clone());
                }
            },
            &Leaf(_, ref sender, _)  => {
                sender.send(action).unwrap();
            }
        }
    }

    fn add_topic_by_path(&mut self, topic_paths: Vec<&str>, path_index: usize) {
        let last_index = topic_paths.len() - 1;
        let path = topic_paths[path_index].to_string();
        match self {
            &mut Root(ref mut children) |
            &mut Leaf(_, _, ref mut children) => {
                match children.entry(path) {
                    Vacant(entry) => {
                        if path_index == last_index {
                            entry.insert(TopicRoute::new(topic_paths.join(SPLITTER)));
                        } else {
                            entry
                                .insert(TopicRoute::new(topic_paths.join(SPLITTER)))
                                .borrow_mut()
                                .add_topic_by_path(topic_paths, path_index + 1);
                        }
                    }
                    Occupied(entry) => {
                        if path_index != last_index {
                            entry.get().borrow_mut().add_topic_by_path(
                                topic_paths,
                                path_index + 1,
                            );
                        }
                    }
                }
            }
        }
    }

    fn find_topics_by_path(&self, topic_paths: Vec<&str>, path_index: usize) -> Vec<TopicRef> {
        let last_index = topic_paths.len() - 1;
        let path = topic_paths[path_index].to_string();

        match self {
            &Root(ref children) |
            &Leaf(_, _, ref children) => {
                if path.eq("*") {
                    let mut nodes: Vec<TopicRef> = Vec::new();
                    for node in children.values() {
                        if path_index == last_index {
                            nodes.push(node.clone());
                        } else {
                            let next_nodes =
                                node.borrow().find_topics_by_path(topic_paths.clone(), path_index + 1);
                            nodes.extend(next_nodes);
                        }
                    }

                    return nodes;
                }

                match children.get(&path) {
                    Some(topic_node) => {
                        if path_index == last_index {
                            vec![topic_node.clone()]
                        } else {
                            topic_node.borrow().find_topics_by_path(topic_paths, path_index + 1)
                        }
                    }
                    None => vec![],
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_topic_nodes() {
        let mut root = TopicRoute::Root(HashMap::new());
        root.add_topic("floor1.device11.sensor1");
        root.add_topic("floor1.device11.sensor2");
        root.add_topic("floor2.device21.sensor1");
        root.add_topic("floor2.device21.sensor2");

        let topics1 = root.find_topics("floor1.device11.sensor1");
        assert_eq!(1, topics1.len());
        let topics2 = root.find_topics("floor2.device21.*");
        assert_eq!(2, topics2.len());

        root.process_action("floor2.device21.sensor1", Subscribe(String::from("sensor1subscriber")));
        root.process_action("floor2.device21.sensor2", Subscribe(String::from("sensor2subscriber")));
        root.process_action("floor2.device21", Subscribe(String::from("device21subscriber")));
        root.process_action("floor2.device21.*", SendMessage(String::from("Hello World!")));
    }
}