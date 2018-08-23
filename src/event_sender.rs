// Copyright 2018 MaidSafe.net limited.
//
// This SAFE Network Software is licensed to you under the MIT license <LICENSE-MIT
// https://opensource.org/licenses/MIT> or the Modified BSD license <LICENSE-BSD
// https://opensource.org/licenses/BSD-3-Clause>, at your option. This file may not be copied,
// modified, or distributed except according to those terms. Please review the Licences for the
// specific language governing permissions and limitations relating to use of the SAFE Network
// Software.

use std::fmt;
use std::sync::mpsc;

/// Errors that can be returned by `EventSender`
#[derive(Debug)]
pub enum EventSenderError<Category, EventSubset> {
    /// Error sending the event subset
    EventSubset(mpsc::SendError<EventSubset>),
    /// Error sending the event category
    Category(mpsc::SendError<Category>),
}

/// This structure is coded to achieve event-subsetting. Receivers in Rust are blocking. One cannot
/// listen to multiple receivers at the same time except by using `try_recv` which again is bad for
/// the same reasons spin-lock based on some sleep is bad (wasting cycles, 50% efficient on an
/// average etc.).
///
/// Consider a module that listens to signals from various other modules. Different
/// modules want to talk to this one. So one solution is make a common event set and all senders
/// (registered in all the interested modules) send events from the same set. This is bad for
/// maintenance. Wrong modules might use events not expected to originate from them since it is just
/// one huge event-set. Thus there is a need of event-subsetting and distribute this module-wise so
/// we prevent modules from using wrong events, completely by design and code-mechanics.
///
/// We also don't want to spawn threads listening to different receivers (which could force shared
/// ownership and is anyway silly otherwise too). This is what `EventSender` helps to salvage. A
/// simple mechanism that does what a `skip-list` in linked list does. It brings forth a concept of
/// an Umbrella event-category and an event subset. The creator of `EventSender` hard-codes the
/// category for different observers. Each category only links to a particular event-subset and
/// type information of this is put into `EventSender` too during its construction. Thus when
/// distributed, the modules cannot cheat (do the wrong thing) by trying to fire an event they are
/// not permitted to. Also a single thread listens to many receivers. All problems solved.
///
/// #Examples
///
/// ```
/// # #![allow(dead_code)]
/// # extern crate maidsafe_utilities;
/// # fn main() {
///     #[derive(Debug, Clone)]
///     enum EventCategory {
///         Network,
///         UserInterface,
///     }
///
///     #[derive(Debug)]
///     enum NetworkEvent {
///         Connected,
///         Disconnected,
///     }
///
///     #[derive(Debug)]
///     enum UiEvent {
///         CreateDirectory,
///         Terminate,
///     }
///
///     let (ui_event_tx, ui_event_rx) = std::sync::mpsc::channel();
///     let (category_tx, category_rx) = std::sync::mpsc::channel();
///     let (network_event_tx, network_event_rx) = std::sync::mpsc::channel();
///
///     let ui_event_sender = maidsafe_utilities::event_sender
///                                             ::EventSender::<EventCategory, UiEvent>
///                                             ::new(ui_event_tx,
///                                                   EventCategory::UserInterface,
///                                                   category_tx.clone());
///
///     let nw_event_sender = maidsafe_utilities::event_sender
///                                             ::EventSender::<EventCategory, NetworkEvent>
///                                             ::new(network_event_tx,
///                                                   EventCategory::Network,
///                                                   category_tx);
///
///     let _joiner = maidsafe_utilities::thread::named("EventListenerThread", move || {
///         for it in category_rx.iter() {
///             match it {
///                 EventCategory::Network => {
///                     if let Ok(network_event) = network_event_rx.try_recv() {
///                         match network_event {
///                             NetworkEvent::Connected    => { /* Do Something */ },
///                             NetworkEvent::Disconnected => { /* Do Something */ },
///                         }
///                     }
///                 },
///                 EventCategory::UserInterface => {
///                     if let Ok(ui_event) = ui_event_rx.try_recv() {
///                         match ui_event {
///                             UiEvent::Terminate       => break,
///                             UiEvent::CreateDirectory => { /* Do Something */ },
///                         }
///                     }
///                 }
///             }
///         }
///     });
///
///     assert!(nw_event_sender.send(NetworkEvent::Connected).is_ok());
///     assert!(ui_event_sender.send(UiEvent::CreateDirectory).is_ok());
///     assert!(ui_event_sender.send(UiEvent::Terminate).is_ok());
/// # }
#[derive(Debug)]
pub struct EventSender<Category, EventSubset> {
    event_tx: mpsc::Sender<EventSubset>,
    event_category: Category,
    event_category_tx: mpsc::Sender<Category>,
}

impl<Category: fmt::Debug + Clone, EventSubset: fmt::Debug> EventSender<Category, EventSubset> {
    /// Create a new instance of `EventSender`. Category type, category value and EventSubset type
    /// are baked into `EventSender` to disallow user code from misusing it.
    pub fn new(
        event_tx: mpsc::Sender<EventSubset>,
        event_category: Category,
        event_category_tx: mpsc::Sender<Category>,
    ) -> EventSender<Category, EventSubset> {
        EventSender {
            event_tx,
            event_category,
            event_category_tx,
        }
    }

    /// Fire an allowed event/signal to the observer.
    pub fn send(&self, event: EventSubset) -> Result<(), EventSenderError<Category, EventSubset>> {
        if let Err(error) = self.event_tx.send(event) {
            return Err(EventSenderError::EventSubset(error));
        }
        if let Err(error) = self.event_category_tx.send(self.event_category.clone()) {
            return Err(EventSenderError::Category(error));
        }

        Ok(())
    }
}

// (Spandan) Need to manually implement this because the default derived one seems faulty in that
// it requires EventSubset to be clonable even though mpsc::Sender<EventSubset> does
// not require EventSubset to be clonable for itself being cloned.
impl<Category: fmt::Debug + Clone, EventSubset: fmt::Debug> Clone
    for EventSender<Category, EventSubset>
{
    fn clone(&self) -> EventSender<Category, EventSubset> {
        EventSender {
            event_tx: self.event_tx.clone(),
            event_category: self.event_category.clone(),
            event_category_tx: self.event_category_tx.clone(),
        }
    }
}

/// Category of events meant for a `MaidSafe` observer listening to both, routing and crust events
#[derive(Clone, Debug)]
pub enum MaidSafeEventCategory {
    /// Used by Crust to indicate a Crust Event has been fired
    Crust,
    /// Used by Routing to indicate a Routing Event has been fired
    Routing,
}

/// Observer that Crust (and users of Routing if required) must allow to be registered
pub type MaidSafeObserver<EventSubset> = EventSender<MaidSafeEventCategory, EventSubset>;

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::mpsc;

    #[test]
    fn marshall_multiple_events() {
        type UiEventSender = EventSender<EventCategory, UiEvent>;
        type NetworkEventSender = EventSender<EventCategory, NetworkEvent>;

        const TOKEN: u32 = 9876;
        const DIR_NAME: &str = "NewDirectory";

        #[derive(Clone, Debug)]
        enum EventCategory {
            Network,
            UserInterface,
        }

        #[derive(Debug)]
        enum NetworkEvent {
            Connected(u32),
            Disconnected,
        }

        #[derive(Debug)]
        enum UiEvent {
            CreateDirectory(String),
            Terminate,
        }

        let (ui_event_tx, ui_event_rx) = mpsc::channel();
        let (category_tx, category_rx) = mpsc::channel();
        let (network_event_tx, network_event_rx) = mpsc::channel();

        let ui_event_sender = UiEventSender::new(
            ui_event_tx,
            EventCategory::UserInterface,
            category_tx.clone(),
        );

        let nw_event_sender =
            NetworkEventSender::new(network_event_tx, EventCategory::Network, category_tx);

        let _joiner = ::thread::named("EventListenerThread", move || {
            for it in category_rx.iter() {
                match it {
                    EventCategory::Network => {
                        if let Ok(network_event) = network_event_rx.try_recv() {
                            if let NetworkEvent::Connected(token) = network_event {
                                assert_eq!(token, TOKEN)
                            } else {
                                panic!("Shouldn't have received this event: {:?}", network_event)
                            }
                        }
                    }
                    EventCategory::UserInterface => {
                        if let Ok(ui_event) = ui_event_rx.try_recv() {
                            match ui_event {
                                UiEvent::CreateDirectory(name) => assert_eq!(name, DIR_NAME),
                                UiEvent::Terminate => break,
                            }
                        }
                    }
                }
            }
        });

        assert!(nw_event_sender.send(NetworkEvent::Connected(TOKEN)).is_ok());
        assert!(
            ui_event_sender
                .send(UiEvent::CreateDirectory(DIR_NAME.to_string()))
                .is_ok()
        );
        assert!(ui_event_sender.send(UiEvent::Terminate).is_ok());

        ::std::thread::sleep(::std::time::Duration::from_millis(500));

        assert!(ui_event_sender.send(UiEvent::Terminate).is_err());
        assert!(nw_event_sender.send(NetworkEvent::Disconnected).is_err());

        let result = ui_event_sender
            .send(UiEvent::CreateDirectory(DIR_NAME.to_owned()))
            .err();
        if let EventSenderError::EventSubset(send_err) = unwrap!(result) {
            if let UiEvent::CreateDirectory(dir_name) = send_err.0 {
                assert_eq!(dir_name, DIR_NAME)
            } else {
                panic!("Expected a different event !")
            }
        } else {
            panic!("Expected a different error !")
        }
    }
}
