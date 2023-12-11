use std::fmt::Display;

type Screen = [[(u16, Kind); 8]; 8];

const CLEAN_SCREEN: Screen = [[(0, Kind::Initial); 8]; 8];

type QuadRef = u64;

pub fn quad_ref(
    offset: usize,
    location: (i32, i32, i32),
    sky_exposure: u8,
    width: u8,
    height: u8,
) -> QuadRef {
    debug_assert!(sky_exposure < 16, "sky exposure out of bounds");

    let location = (location.0 & 31, location.1 & 31, location.2 & 31);
    let sky_exposure = sky_exposure & 15;
    let width = width & 31;
    let height = height & 31;

    offset as u64
        | (location.0 as u64) << 32
        | (location.1 as u64) << 37
        | (location.2 as u64) << 42
        | (sky_exposure as u64) << 47
        | (width as u64) << 51
        | (height as u64) << 56
}

pub fn extend_quad_ref_w(quad_ref: &mut QuadRef) {
    *quad_ref += 1 << 51;
}

pub fn extend_quad_ref_h(quad_ref: &mut QuadRef) {
    *quad_ref += 1 << 56;
}

#[derive(Clone, Copy, Debug)]
struct Quad {
    color: u16,
    x0: u16,
    y0: u16,
    width: u8,
    height: u8,
}

#[derive(Clone, Copy, Debug)]
enum Kind {
    Initial,
    Inside,
    Final,
    Single,
}

impl Display for Kind {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let str = match *self {
            Self::Initial => "██",
            Self::Inside => "░░",
            Self::Final => "▒▒",
            Self::Single => "█▒",
        };

        f.write_str(str)
    }
}

fn main() {
    let mut mesh = vec![
        quad_ref(3, (0, 0, 0), 0, 0, 0),
        quad_ref(1, (1, 0, 0), 0, 4, 0),
        quad_ref(4, (6, 0, 0), 0, 1, 0),

        quad_ref(3, (0, 1, 0), 0, 0, 0),
        quad_ref(1, (1, 1, 0), 0, 4, 0),
        quad_ref(4, (6, 1, 0), 0, 1, 0),

        quad_ref(3, (0, 2, 0), 0, 0, 0),
        quad_ref(1, (1, 2, 0), 0, 0, 0),
        quad_ref(1, (4, 2, 0), 0, 1, 0),
        quad_ref(4, (6, 2, 0), 0, 1, 0),

        quad_ref(3, (0, 3, 0), 0, 0, 0),
        quad_ref(1, (1, 3, 0), 0, 1, 0),
        quad_ref(2, (5, 3, 0), 0, 2, 0),

        quad_ref(3, (0, 4, 0), 0, 0, 0),
        quad_ref(1, (1, 4, 0), 0, 2, 0),
        quad_ref(2, (4, 4, 0), 0, 3, 0),

        quad_ref(3, (0, 5, 0), 0, 0, 0),
        quad_ref(1, (1, 5, 0), 0, 1, 0),
        quad_ref(1, (5, 5, 0), 0, 2, 0),

        quad_ref(3, (0, 6, 0), 0, 0, 0),
        quad_ref(1, (1, 6, 0), 0, 2, 0),
        quad_ref(2, (4, 6, 0), 0, 3, 0),

        quad_ref(3, (0, 7, 0), 0, 0, 0),
        quad_ref(1, (1, 7, 0), 0, 2, 0),
        quad_ref(2, (4, 7, 0), 0, 3, 0),
    ];

    println!();
    println!("1D greedy meshing ({} rects)", mesh.len());
    println!();

    let mut screen = CLEAN_SCREEN;
    render(&mesh, &mut screen);
    display(&screen);

    greedy2d(&mut mesh);

    println!();
    println!("2D greedy meshing ({} rects)", mesh.len());
    println!();

    let mut screen = CLEAN_SCREEN;

    render(&mesh, &mut screen);
    display(&screen);

    println!()
}

fn render(mesh: &[QuadRef], screen: &mut Screen) {
    for qref in mesh.iter() {
        let color = qref & 0xFFFF_FFFF;
        let x0 = (qref >> 32) & 0x1F;
        let y0 = (qref >> 37) & 0x1F;
        let width = (qref >> 51) & 0x1F;
        let height = (qref >> 56) & 0x1F;

        for y in y0 ..= y0 + height {
        for x in x0 ..= x0 + width {
            let mut kind = Kind::Inside;
            if (x, y) == (x0, y0) { kind = Kind::Initial; }
            if (x, y) == (x0 + width, y0 + height) { kind = Kind::Final; }
            if (width, height) == (0, 0) { kind = Kind::Single; }

            screen[y as usize][x as usize] = (color as u16, kind);
        }
        }
    }
}

fn display(screen: &Screen) {
    for line in screen {
        for (color, kind) in line {
            if *color == 0 {
                print!("\x1B[1;3{}m  \x1B[0m", color);
            } else {
                print!("\x1B[1;3{}m{}\x1B[0m", color, kind);
            }
        }

        println!();
    }
}

#[inline(never)]
fn greedy2d(mesh: &mut Vec<QuadRef>) {
    let mut dest = 0;
    let mut back = 0;
    let mut lead = 0;

    let xm = 0x0000_001F_0000_0000;
    let yo = 37;

    while back < mesh.len() {
        if lead == mesh.len() {
            *unsafe { mesh.get_unchecked_mut(dest) } = *unsafe { mesh.get_unchecked(back) };
            dest += 1;
            back += 1;
            continue;
        }

        let b = *unsafe { mesh.get_unchecked(back) };
        let l = *unsafe { mesh.get_unchecked(lead) };

        let bcwx0 = b & 0x00F8_0000_FFFF_FFFF | xm;
        let bh = (b >> 56) & 0x1F;
        let bx0 = (b >> 32) & 0x1F;
        let by0 = (b >> yo) & 0x1F;

        let lcwx0 = l & 0x00F8_0000_FFFF_FFFF | xm;
        let lx0 = (l >> 32) & 0x1F;
        let ly0 = (l >> yo) & 0x1F;

        let Δy = ly0 - by0 - bh;

        if Δy == 0 {
            lead += 1;
        } else if Δy > 1 {
            *unsafe { mesh.get_unchecked_mut(dest) } = b;
            dest += 1;
            back += 1;
        } else if bx0 > lx0 {
            lead += 1;
        } else if bcwx0 == lcwx0 {
            *unsafe { mesh.get_unchecked_mut(lead) } = b;
            extend_quad_ref_h(unsafe { mesh.get_unchecked_mut(lead) });
            back += 1;
            lead += 1;
        } else {
            *unsafe { mesh.get_unchecked_mut(dest) } = b;
            dest += 1;
            back += 1;
        }
    }

    mesh.truncate(dest);
}
