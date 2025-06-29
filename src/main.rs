use std::sync::{Arc, Mutex};

use roger::{
    common::{Failure, Location, Message, Payload, Request, Response},
    server::travel_guide,
};

const SERVER_PORT: u16 = 8080;

fn main() {
    // create and populate the itinerary (with a counter keeping track of the curreng location)
    let itinerary = Arc::new(Mutex::new((0, Vec::<Location>::new())));
    itinerary.lock().unwrap().1.append(&mut vec![
        Location::HOME,
        Location::CHURCH,
        Location::WOODS,
    ]);

    travel_guide(SERVER_PORT, itinerary, |msg, itin| {
        // should only be getting messages of the "Request" type
        let request = match &msg.data {
            Payload::Request(request) => request,
            _ => return Message::new_response(Response::Failure(Failure::InvalidRequest)),
        };

        match request {
            // Tell the traveller the itinerary
            Request::List => {
                let list = itin.lock().unwrap().1.iter().cloned().collect();
                return Message::new_response(Response::List(list));
            }
            Request::Current => {
                let idx: usize = itin.lock().unwrap().0;
                if let Some(loc) = itin.lock().unwrap().1.get(idx) {
                    return Message::new_response(Response::Where(loc.clone()));
                } else {
                    return Message::new_response(Response::Failure(
                        Failure::LocationNotOnItinerary,
                    ));
                }
            }
            Request::Next => {
                if let Some(loc) = itin.lock().unwrap().1.get(itin.lock().unwrap().0 + 1) {
                    itin.lock().unwrap().0 += 1;
                    return Message::new_response(Response::Where(loc.clone()));
                } else {
                    return Message::new_response(Response::Done);
                }
            }
            _ => return Message::new_response(Response::Failure(Failure::InvalidRequest)),
        };
    });
}
