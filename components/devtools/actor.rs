/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

/// General actor system infrastructure.

use std::any::{Any, AnyRefExt, AnyMutRefExt};
use std::collections::hashmap::HashMap;
use std::io::TcpStream;
use std::mem::{transmute, transmute_copy};
use std::raw::TraitObject;
use serialize::json;

/// A common trait for all devtools actors that encompasses an immutable name
/// and the ability to process messages that are directed to particular actors.
/// TODO: ensure the name is immutable
pub trait Actor: Any {
    fn handle_message(&self,
                      registry: &ActorRegistry,
                      msg_type: &String,
                      msg: &json::Object,
                      stream: &mut TcpStream) -> bool;
    fn name(&self) -> String;
}

impl<'a> AnyMutRefExt<'a> for &'a mut Actor {
    fn downcast_mut<T: 'static>(self) -> Option<&'a mut T> {
        if self.is::<T>() {
            unsafe {
                // Get the raw representation of the trait object
                let to: TraitObject = transmute_copy(&self);

                // Extract the data pointer
                Some(transmute(to.data))
            }
        } else {
            None
        }
    }
}

impl<'a> AnyRefExt<'a> for &'a Actor {
    fn is<T: 'static>(self) -> bool {
        //FIXME: This implementation is bogus since get_type_id is private now.
        //       However, this implementation is only needed so long as there's a Rust bug
        //       that prevents downcast_ref from giving realistic return values, and this is
        //       ok since we're careful with the types we pull out of the hashmap.
        /*let t = TypeId::of::<T>();
        let boxed = self.get_type_id();
        t == boxed*/
        true
    }

    fn downcast_ref<T: 'static>(self) -> Option<&'a T> {
        if self.is::<T>() {
            unsafe {
                // Get the raw representation of the trait object
                let to: TraitObject = transmute_copy(&self);

                // Extract the data pointer
                Some(transmute(to.data))
            }
        } else {
            None
        }
    }
}

/// A list of known, owned actors.
pub struct ActorRegistry {
    actors: HashMap<String, Box<Actor+Send+Sized>>,
}

impl ActorRegistry {
    /// Create an empty registry.
    pub fn new() -> ActorRegistry {
        ActorRegistry {
            actors: HashMap::new(),
        }
    }

    /// Add an actor to the registry of known actors that can receive messages.
    pub fn register(&mut self, actor: Box<Actor+Send+Sized>) {
        self.actors.insert(actor.name().to_string(), actor);
    }

    /// Find an actor by registered name
    pub fn find<'a, T: 'static>(&'a self, name: &str) -> &'a T {
        //FIXME: Rust bug forces us to implement bogus Any for Actor since downcast_ref currently
        //       fails for unknown reasons.
        /*let actor: &Actor+Send+Sized = *self.actors.find(&name.to_string()).unwrap();
        (actor as &Any).downcast_ref::<T>().unwrap()*/
        self.actors.find(&name.to_string()).unwrap().as_ref::<T>().unwrap()
    }

    /// Find an actor by registered name
    pub fn find_mut<'a, T: 'static>(&'a mut self, name: &str) -> &'a mut T {
        //FIXME: Rust bug forces us to implement bogus Any for Actor since downcast_ref currently
        //       fails for unknown reasons.
        /*let actor: &mut Actor+Send+Sized = *self.actors.find_mut(&name.to_string()).unwrap();
        (actor as &mut Any).downcast_mut::<T>().unwrap()*/
        self.actors.find_mut(&name.to_string()).unwrap().downcast_mut::<T>().unwrap()
    }

    /// Attempt to process a message as directed by its `to` property. If the actor is not
    /// found or does not indicate that it knew how to process the message, ignore the failure.
    pub fn handle_message(&self, msg: &json::Object, stream: &mut TcpStream) {
        let to = msg.find(&"to".to_string()).unwrap().as_string().unwrap();
        match self.actors.find(&to.to_string()) {
            None => println!("message received for unknown actor \"{:s}\"", to),
            Some(actor) => {
                let msg_type = msg.find(&"type".to_string()).unwrap().as_string().unwrap();
                if !actor.handle_message(self, &msg_type.to_string(), msg, stream) {
                    println!("unexpected message type \"{:s}\" found for actor \"{:s}\"",
                             msg_type, to);
                }
            }
        }
    }
}
