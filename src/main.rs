#![feature(iter_collect_into)]
#![feature(new_uninit)]

mod buddy;
mod gfx;

use std::{mem, sync::Arc, time::Instant};

use gfx::Gfx;
use rand::Rng;
use winit::{
    dpi::PhysicalSize,
    event_loop::{ControlFlow, EventLoop},
    window::WindowBuilder,
};

use crate::buddy::Buddy;

type QuadRef = u64;

#[pollster::main]
async fn main() {
    let event_loop = EventLoop::new().unwrap();
    event_loop.set_control_flow(ControlFlow::Poll);

    let window = Arc::new(
        WindowBuilder::new()
            .with_title("aXial")
            .with_inner_size(PhysicalSize::new(854, 480))
            .build(&event_loop)
            .unwrap(),
    );

    let gfx = Gfx::new(window).await;

    // let capacity = 0x200_0000;
    let size = 0x1000_0000;
    let capacity = size / mem::size_of::<QuadRef>();
    let min_order = 8;

    println!("{} bytes", size);
    println!("{} entries", capacity);
    println!("{} minimum alloc", 1 << min_order);
    println!();

    // Time buddy creation
    let then = Instant::now();
    let untouched_quad_buddy = Buddy::<QuadRef>::new(&gfx, capacity, min_order);
    let mut quad_buddy = Buddy::<QuadRef>::new(&gfx, capacity, min_order);
    println!("init\t\t{:?}", then.elapsed());

    // Perform as many allocations as possible
    // (minimum size allocations)
    let mut handles = Vec::with_capacity(capacity >> min_order);

    // Time single buddy allocation
    let then = Instant::now();
    let handle = quad_buddy.alloc(1 << min_order).unwrap();
    println!("alloc 1\t\t{:?}", then.elapsed());

    // Time single buddy free
    let then = Instant::now();
    quad_buddy.free(handle);
    println!("free 1\t\t{:?}", then.elapsed());

    // Time largest buddy allocation run
    let then = Instant::now();
    for _ in 0..handles.capacity() {
        if let Some(handle) = quad_buddy.alloc(1 << min_order) {
            handles.push(handle);
        } else {
            // All those allocations must succeed
            unreachable!();
        }
    }
    println!("alloc {}\t{:?}", handles.capacity() - 1, then.elapsed());

    // Any further allocation must fail
    assert!(quad_buddy.alloc(1).is_none());

    // Time buddy free
    let then = Instant::now();
    let capacity = handles.capacity();
    for handle in handles {
        quad_buddy.free(handle);
    }
    println!("free {}\t{:?}", capacity, then.elapsed());

    // The buddy must be left in the same state as it was after its creation
    assert!(untouched_quad_buddy.check_is_same(&quad_buddy));

    let _ = event_loop.run(|_, _| {});
}
