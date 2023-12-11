#![feature(isqrt)]
#![feature(new_uninit)]

mod gfx;

use std::{
    marker::PhantomData,
    mem::{self, MaybeUninit},
    sync::Arc,
};

use bytemuck::Pod;
use gfx::Gfx;
use wgpu::{Buffer, BufferDescriptor, BufferUsages};
use winit::{
    dpi::PhysicalSize,
    event_loop::{ControlFlow, EventLoop},
    window::WindowBuilder,
};

type QuadRef = u64;

// Inspired by
// https://nickmcd.me/2021/04/04/high-performance-voxel-engine/#voxel-data-rendering-systems,
// but with a buddy allocator instead of fixed-size buckets.
#[derive(Debug)]
pub struct Buddy<T: Pod> {
    buffer: Buffer,
    alloc_tree: Box<[u8]>,

    // `buffer` holds items of type T
    _casper: PhantomData<T>,
}

impl<T: Pod> Buddy<T> {
    const STRIDE: usize = mem::size_of::<T>();
    const USED: u8 = u8::MAX;

    pub fn capacity(&self) -> usize {
        (self.buffer.size() / Self::STRIDE as u64) as _
    }

    pub fn new(gfx: &Gfx, capacity: usize, min_order: u8) -> Self {
        let capacity = capacity.next_power_of_two();
        let max_order = capacity.ilog2() as u8;

        debug_assert!(
            max_order >= min_order,
            "minimum order too big for requested capacity"
        );

        // Allocate and initialize tree to keep track of used/free blocks
        let tree_height = max_order - min_order + 1;
        let num_leaves = 1 << tree_height;
        let mut uninit_alloc_tree = Box::new_zeroed_slice(num_leaves);

        for level in 0..tree_height {
            let order = MaybeUninit::new(max_order - level);
            let len = 1 << level;
            uninit_alloc_tree[len..2 * len].fill(order);
        }

        // Allocate buffer to hold items
        let descriptor = BufferDescriptor {
            label: None,
            size: Self::STRIDE as u64 * capacity as u64,
            usage: BufferUsages::STORAGE | BufferUsages::COPY_DST,
            mapped_at_creation: false,
        };

        let buffer = gfx.device.create_buffer(&descriptor);

        Self {
            buffer,
            alloc_tree: unsafe { uninit_alloc_tree.assume_init() },
            _casper: PhantomData,
        }
    }

    pub fn alloc(&mut self, len: usize) -> Option<usize> {
        let len = len.next_power_of_two();
        let order = len.ilog2() as u8;

        // Early return if request cannot be satisfied
        if self.alloc_tree[1] < order {
            return None;
        }

        println!("Allocating block {} elements long (order {})", len, order);
        todo!()
    }

    pub fn free(&mut self, len: usize) {
        todo!()
    }

    pub fn write(&mut self, gfx: &Gfx, data: &[T]) {
        todo!()
    }

    pub fn load(&mut self, gfx: &Gfx, data: &[T]) {
        todo!()
    }
}

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
    //let mut quad_buddy = Buddy::<QuadRef>::new(&gfx, 0x200_0000, 5);
    let mut quad_buddy = Buddy::<QuadRef>::new(&gfx, 4, 0);
    println!("{:?}", quad_buddy);
    quad_buddy.alloc(3);

    let _ = event_loop.run(|_, _| {});
}
