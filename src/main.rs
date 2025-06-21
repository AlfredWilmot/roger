
// TODO:
//
// > background thread runs server that listens for locations and responds with
// where to go next based on the travel_fsm
//
// > foreground fsm that creates 100 client connections to the server
// (each client sends a random Location and prints the response + the ping delay)

/// some places to go
enum Location {
    HOME,
    CITY,
    WOODS,
    BEACH,
    FIELD,
    CAFE,
    SHOP,
    CATHEDRAL
}

/// some arbitrary travel itinerary
fn travel_fsm(loc: Location) -> Location {
    match loc {
        Location::HOME=> Location::CITY,
        Location::CITY => Location::WOODS,
        Location::WOODS => Location::BEACH,
        Location::BEACH=> Location::FIELD,
        Location::FIELD=> Location::CAFE,
        Location::CAFE=> Location::SHOP,
        Location::SHOP=> Location::CATHEDRAL,
        Location::CATHEDRAL=> Location::HOME,
    }
}

fn main() {
    println!("Hello, world!");
}
