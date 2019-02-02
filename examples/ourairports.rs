extern crate friar;

use friar::ourairports;

fn main() {
    let airports = ourairports::Airport::all().unwrap();
    println!("Airports: {}", airports.len());

    let runways = ourairports::Runway::all().unwrap();
    println!("Runways: {}", runways.len());

    for airport in airports.iter() {
        if airport.ident == "KDEN" {
            println!("{:#?}", airport);
            for runway in runways.iter() {
                if runway.airport_ref == airport.id {
                    println!("{:#?}", runway);
                }
            }
        }
    }
}
