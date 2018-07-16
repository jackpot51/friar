extern crate friar;

use friar::earth::Earth;
use friar::reference::Reference;
use friar::spheroid::Spheroid;

// Test conformance to http://pandora.nla.gov.au/pan/24764/20060809-0000/DSTO-TN-0640.pdf
fn main() {
    let earth = Earth;

    let adelaide = earth.coordinate(-34.9, 138.5, 0.0);
    let brussels = earth.coordinate(50.8, 4.3, 0.0);

    println!("Adelaide: {} => {}", adelaide, adelaide.position());
    assert!(adelaide.position().vector(&earth.position(-3.92E6, 3.47E6, -3.63E6)).norm() < 10000.0);

    println!("Brussels: {} => {}", brussels, brussels.position());
    assert!(brussels.position().vector(&earth.position(4.03E6, 0.30E6, 4.92E6)).norm() < 10000.0);

    let heading = adelaide.heading(&brussels);
    println!("Heading: {}", heading);
    assert!((heading - 310.0).abs() < 1.0);

    let plane = earth.coordinate(adelaide.latitude, adelaide.longitude, 10000.0);
    let plane_pos = plane.position();
    println!("Plane: {} => {}", plane, plane_pos);
    assert!(plane_pos.vector(&earth.position(-3.93E6, 3.48E6, -3.63E6)).norm() < 10000.0);

    let north = plane.offset(1.0, 0.0, 0.0);
    let north_pos = north.position();
    let east = plane.offset(1.0, 90.0, 0.0);
    let east_pos = east.position();
    let down = plane.offset(-1.0, 0.0, 90.0);
    let down_pos = down.position();
    println!("North: {} => {}", north, north_pos);
    println!("East: {} => {}", east, east_pos);
    println!("Down: {} => {}", down, down_pos);

    let x0 = plane_pos.vector(&north_pos).normalize();
    let y0 = plane_pos.vector(&east_pos).normalize();
    let z0 = plane_pos.vector(&down_pos).normalize();

    println!("x0: {}", x0);
    println!("y0: {}", y0);
    println!("z0: {}", z0);

    let x1 = x0.rotate(&z0, 135.0);
    let y1 = y0.rotate(&z0, 135.0);
    let z1 = z0.rotate(&z0, 135.0);

    println!("x1: {}", x1);
    println!("y1: {}", y1);
    println!("z1: {}", z1);

    let x2 = x1.rotate(&y1, 20.0);
    let y2 = y1.rotate(&y1, 20.0);
    let z2 = z1.rotate(&y1, 20.0);

    println!("x2: {}", x2);
    println!("y2: {}", y2);
    println!("z2: {}", z2);

    let x3 = x2.rotate(&x2, 30.0);
    let y3 = y2.rotate(&x2, 30.0);
    let z3 = z2.rotate(&x2, 30.0);

    println!("x3: {}", x3);
    println!("y3: {}", y3);
    println!("z3: {}", z3);

    {
        let x0 = earth.vector(1.0, 0.0, 0.0);
        let y0 = earth.vector(0.0, 1.0, 0.0);
        let z0 = earth.vector(0.0, 0.0, 1.0);

        let psi = x3.dot(&y0).atan2(
            x3.dot(&x0)
        );
        let theta = (-x3.dot(&z0)).atan2(
            (x3.dot(&x0).powi(2) + x3.dot(&y0).powi(2)).sqrt()
        );

        let y1 = y0.rotate(&z0, psi.to_degrees());
        let z2 = z0.rotate(&y1, theta.to_degrees());

        let phi = y3.dot(&z2).atan2(
            y3.dot(&y1)
        );

        println!("psi: {}", psi.to_degrees());
        println!("theta: {}", theta.to_degrees());
        println!("phi: {}", phi.to_degrees());
    }
}
