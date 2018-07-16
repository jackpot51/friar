extern crate friar;
extern crate orbclient;

use friar::coordinate::Coordinate;
use friar::earth::Earth;
use friar::reference::Reference;
use friar::spheroid::Spheroid;
use orbclient::{Color, EventOption, Renderer, Window};

// Rediculously complicated formula for angles from http://pandora.nla.gov.au/pan/24764/20060809-0000/DSTO-TN-0640.pdf
fn complicated(plane: &Coordinate<Earth>, heading: f64, pitch: f64, roll: f64) -> (f64, f64, f64) {
    let plane_pos = plane.position();

    let north = plane.offset(1.0, 0.0, 0.0);
    let north_pos = north.position();
    let east = plane.offset(1.0, 90.0, 0.0);
    let east_pos = east.position();
    let down = plane.offset(-1.0, 0.0, 90.0);
    let down_pos = down.position();

    let x0 = plane_pos.vector(&north_pos).normalize();
    let y0 = plane_pos.vector(&east_pos).normalize();
    let z0 = plane_pos.vector(&down_pos).normalize();

    let x1 = x0.rotate(&z0, heading);
    let y1 = y0.rotate(&z0, heading);
    //let z1 = z0.rotate(&z0, heading);

    let x2 = x1.rotate(&y1, pitch);
    let y2 = y1.rotate(&y1, pitch);
    //let z2 = z1.rotate(&y1, pitch);

    let x3 = x2.rotate(&x2, roll);
    let y3 = y2.rotate(&x2, roll);
    //let z3 = z2.rotate(&x2, roll);

    {
        let x0 = plane.reference.vector(1.0, 0.0, 0.0);
        let y0 = plane.reference.vector(0.0, 1.0, 0.0);
        let z0 = plane.reference.vector(0.0, 0.0, 1.0);

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

        (phi.to_degrees(), theta.to_degrees(), psi.to_degrees())
    }
}

// Test conformance to http://pandora.nla.gov.au/pan/24764/20060809-0000/DSTO-TN-0640.pdf
fn test() {
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

fn main() {
    test();

    let mut w = Window::new(-1, -1, 800, 600, "FRIAR").unwrap();

    let earth = Earth;

    let red = earth.coordinate(39.73922277, -104.9888542, 1597.0);
    let orange = earth.coordinate(39.73922277, -104.9888542, 1567.0);
    let yellow = earth.coordinate(39.73922277, -104.9888542, 1537.0);
    let green = earth.coordinate(39.73923927, -104.98668697, 1600.0);
    let blue = earth.coordinate(39.73926402, -104.9847987, 1608.0);

    let spheres = vec![
        (red.position(), Color::rgb(0xFF, 0x00, 0x00), "red".to_string()),
        (orange.position(), Color::rgb(0xFF, 0x7F, 0x00), "orange".to_string()),
        (yellow.position(), Color::rgb(0xFF, 0xFF, 0x00), "yellow".to_string()),
        (green.position(), Color::rgb(0x00, 0xFF, 0x00), "green".to_string()),
        (blue.position(), Color::rgb(0x00, 0x00, 0xFF), "blue".to_string()),
    ];

    let origin = earth.coordinate(39.73924752, -104.99111798, 1597.0);
    let mut viewer = origin.duplicate();

    let mut redraw = true;
    let mut heading = viewer.heading(&red);
    let mut pitch = 0.0;
    let mut roll = 0.0;
    let mut circles = Vec::with_capacity(spheres.len());
    loop {
        if redraw {
            let viewer_pos = viewer.position();
            let viewer_rot = viewer.rotation();
            let calculated_rot = complicated(&viewer, heading, pitch, roll);
            let perspective_rot = (90.0 - calculated_rot.1, -calculated_rot.0, 90.0 + calculated_rot.2);
            let perspective = viewer_pos.perspective(perspective_rot.0, perspective_rot.1, perspective_rot.2);
            let viewport = perspective.viewport(0.0, 0.0, 1.0);
            let screen = viewport.screen(w.width() as f64, w.height() as f64, 4800.0);

            println!("rotation: {}, {}, {}", heading, pitch, roll);
            println!("viewer: {}", viewer);
            println!("viewer ECEF: {}", viewer_pos);
            println!("viewer rot: {:?}", viewer_rot);
            println!("calculated rot: {:?}", calculated_rot);
            println!("perspective rot: {:?}", perspective_rot);

            circles.clear();

            for sphere in spheres.iter() {
                let (px, py, pz) = screen.transform(&sphere.0);
                println!("{}: {}, {}, {}", sphere.2, px, py, pz);
                circles.push((px, py, pz, sphere.1));
            }

            circles.sort_unstable_by(|a, b| a.2.partial_cmp(&b.2).unwrap());

            w.set(Color::rgb(0, 0, 0));

            for circle in circles.iter() {
                if circle.2 > 0.0 {
                    w.circle(circle.0 as i32, circle.1 as i32, -circle.2 as i32, circle.3);
                }
            }

            w.sync();

            redraw = false;
        }

        for event in w.events() {
            match event.to_option() {
                EventOption::Key(key_event) => match key_event.scancode {
                    orbclient::K_W if key_event.pressed => {
                        viewer = viewer.offset(1.0, heading, pitch);
                        redraw = true;
                    },
                    orbclient::K_S if key_event.pressed => {
                        viewer = viewer.offset(-1.0, heading, pitch);
                        redraw = true;
                    },
                    orbclient::K_A if key_event.pressed => {
                        viewer = viewer.offset(-1.0, heading + 90.0, 0.0);
                        redraw = true;
                    },
                    orbclient::K_D if key_event.pressed => {
                        viewer = viewer.offset(1.0, heading + 90.0, 0.0);
                        redraw = true;
                    },
                    orbclient::K_Q if key_event.pressed => {
                        viewer = viewer.offset(1.0, 0.0, 90.0);
                        redraw = true;
                    },
                    orbclient::K_E if key_event.pressed => {
                        viewer = viewer.offset(-1.0, 0.0, 90.0);
                        redraw = true;
                    },

                    orbclient::K_J if key_event.pressed => {
                        heading -= 1.0;
                        redraw = true;
                    },
                    orbclient::K_L if key_event.pressed => {
                        heading += 1.0;
                        redraw = true;
                    },
                    orbclient::K_I if key_event.pressed => {
                        pitch += 1.0;
                        redraw = true;
                    },
                    orbclient::K_K if key_event.pressed => {
                        pitch -= 1.0;
                        redraw = true;
                    },
                    orbclient::K_U if key_event.pressed => {
                        roll -= 1.0;
                        redraw = true;
                    },
                    orbclient::K_O if key_event.pressed => {
                        roll += 1.0;
                        redraw = true;
                    },

                    _ => (),
                },
                EventOption::Quit(_quit_event) => return,
                _ => ()
            }
        }
    }
}
