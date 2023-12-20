use std::{
    marker::PhantomData,
    mem::{self, MaybeUninit},
    num::NonZeroUsize,
};

use bytemuck::Pod;
use wgpu::{Buffer, BufferDescriptor, BufferUsages};

use crate::gfx::Gfx;

// Inspired by
// https://nickmcd.me/2021/04/04/high-performance-voxel-engine/#voxel-data-rendering-systems,
// but with a buddy allocator inspired by
// https://github.com/Restioson/buddy-allocator-workshop#bitmap-tree-buddy-allocator
#[derive(Debug)]
pub struct Buddy<T: Pod> {
    buffer: Buffer,
    min_order: u8,
    pub alloc_tree: Box<[i8]>,

    // `buffer` holds items of type T
    _casper: PhantomData<T>,
}

impl<T: Pod> Buddy<T> {
    const STRIDE: usize = mem::size_of::<T>();
    const USED: i8 = i8::MIN;

    pub const fn capacity(&self) -> usize {
        (self.alloc_tree.len() / 2) << self.min_order
    }

    pub const fn max_order(&self) -> u8 {
        self.capacity().ilog2() as _
    }

    pub fn new(gfx: &Gfx, capacity: usize, min_order: u8) -> Self {
        let capacity = capacity.next_power_of_two();
        let max_order = capacity.ilog2() as u8;

        // Allocate tree to keep track of used/free blocks
        let num_nodes = (2 * capacity) >> min_order;
        let mut uninit_alloc_tree = Box::new_uninit_slice(num_nodes);

        // Initialize with a single block covering the entire buffer
        for level in 0..=(max_order - min_order) {
            let order = max_order - level;
            let level = level as usize;
            let slice = &mut uninit_alloc_tree[1 << level..2 << level];
            slice.fill(MaybeUninit::new(order as _));
        }

        // Initialize the first unused value of the array to avoid UB
        uninit_alloc_tree[0].write(i8::MIN);

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
            min_order,
            alloc_tree: unsafe { uninit_alloc_tree.assume_init() },
            _casper: PhantomData,
        }
    }

    fn update_parents(&mut self, mut block: usize) {
        while block > 1 {
            let a = self.alloc_tree[block ^ 0];
            let b = self.alloc_tree[block ^ 1];
            let merge = a == b;
            block >>= 1;

            let old_value = self.alloc_tree[block];
            let new_value = i8::max(a, b) + merge as i8;
            self.alloc_tree[block] = new_value;

            // If the value was not changed,
            // no further parent will not be affected
            if old_value == new_value {
                break;
            }
        }
    }

    pub fn alloc(&mut self, len: usize) -> Option<Handle<T>> {
        // Calculate block order needed,
        // capped to the minimum size available
        let len = len.next_power_of_two();
        let target_order = u8::max(len.ilog2() as u8, self.min_order);

        // Early exit if there is no big enough block
        if self.alloc_tree[1] < target_order as i8 {
            return None;
        }

        // Walk down the tree,
        // looking for a suitable block
        let mut block = 1;
        let levels_down = self.max_order() - target_order;
        for _ in 0..levels_down {
            block <<= 1;

            // jmi2k: when both are suitable,
            //        choose the smallest one to mitigate fragmentation
            let suitable = self.alloc_tree[block] >= target_order as i8;
            block ^= !suitable as usize;
        }

        // Claim block
        let handle = Handle::new(block);
        self.alloc_tree[block] = Self::USED;
        self.update_parents(block);
        handle
    }

    pub fn free(&mut self, handle: Handle<T>) {
        let block = handle.inner.get();
        let order = self.max_order() - block.ilog2() as u8;
        self.alloc_tree[block] = order as i8;
        self.update_parents(block);
    }

    pub fn write(&mut self, gfx: &Gfx, handle: &Handle<T>, data: &[T]) {
        let block = handle.inner.get();
        let order = self.max_order() - block.ilog2() as u8;
        let bias = 1 << block.ilog2();
        let offset = (block - bias) << order;
        let blob = bytemuck::cast_slice(data);
        gfx.queue.write_buffer(&self.buffer, offset as _, blob);
    }

    pub fn load(&mut self, gfx: &Gfx, data: &[T]) -> Option<Handle<T>> {
        let handle = self.alloc(data.len())?;
        self.write(gfx, &handle, data);
        Some(handle)
    }

    pub fn check_is_same(&self, other: &Self) -> bool {
        for i in 0..self.alloc_tree.len() {
            if self.alloc_tree[i] != other.alloc_tree[i] {
                return false;
            }
        }

        true
    }
}

#[repr(transparent)]
#[derive(Debug)]
pub struct Handle<T: Pod> {
    inner: NonZeroUsize,

    // Handle comes from a `Buddy<T>`
    _casper: PhantomData<T>,
}

impl<T: Pod> Handle<T> {
    fn new(block: usize) -> Option<Self> {
        let elf = Self {
            inner: NonZeroUsize::new(block)?,
            _casper: PhantomData,
        };

        Some(elf)
    }
}
