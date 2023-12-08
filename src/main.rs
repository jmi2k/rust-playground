use std::fmt::Display;

type Screen = [[(u16, Kind); 8]; 8];

const CLEAN_SCREEN: Screen = [[(0, Kind::Initial); 8]; 8];

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
        Quad { color: 3, x0: 0, y0: 0, width: 0, height: 0 },
        Quad { color: 1, x0: 1, y0: 0, width: 4, height: 0 },
        Quad { color: 4, x0: 6, y0: 0, width: 1, height: 0 },

        Quad { color: 3, x0: 0, y0: 1, width: 0, height: 0 },
        Quad { color: 1, x0: 1, y0: 1, width: 4, height: 0 },
        Quad { color: 4, x0: 6, y0: 1, width: 1, height: 0 },

        Quad { color: 3, x0: 0, y0: 2, width: 0, height: 0 },
        Quad { color: 1, x0: 1, y0: 2, width: 0, height: 0 },
        Quad { color: 1, x0: 4, y0: 2, width: 1, height: 0 },
        Quad { color: 4, x0: 6, y0: 2, width: 1, height: 0 },

        Quad { color: 3, x0: 0, y0: 3, width: 0, height: 0 },
        Quad { color: 1, x0: 1, y0: 3, width: 1, height: 0 },
        Quad { color: 2, x0: 5, y0: 3, width: 2, height: 0 },

        Quad { color: 3, x0: 0, y0: 4, width: 0, height: 0 },
        Quad { color: 1, x0: 1, y0: 4, width: 2, height: 0 },
        Quad { color: 2, x0: 4, y0: 4, width: 3, height: 0 },

        Quad { color: 3, x0: 0, y0: 5, width: 0, height: 0 },
        Quad { color: 1, x0: 1, y0: 5, width: 1, height: 0 },
        Quad { color: 1, x0: 5, y0: 5, width: 2, height: 0 },

        Quad { color: 3, x0: 0, y0: 6, width: 0, height: 0 },
        Quad { color: 1, x0: 1, y0: 6, width: 2, height: 0 },
        Quad { color: 2, x0: 4, y0: 6, width: 3, height: 0 },

        Quad { color: 3, x0: 0, y0: 7, width: 0, height: 0 },
        Quad { color: 1, x0: 1, y0: 7, width: 2, height: 0 },
        Quad { color: 2, x0: 4, y0: 7, width: 3, height: 0 },
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
}

fn render(mesh: &[Quad], screen: &mut Screen) {
    for &Quad { color, x0, y0, width, height } in mesh.iter() {
        for y in y0 ..= y0 + height as u16 {
        for x in x0 ..= x0 + width as u16 {
            let mut kind = Kind::Inside;
            if (x, y) == (x0, y0) { kind = Kind::Initial; }
            if (x, y) == (x0 + width as u16, y0 + height as u16) { kind = Kind::Final; }
            if (width, height) == (0, 0) { kind = Kind::Single; }
            screen[y as usize][x as usize] = (color, kind);
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
fn greedy2d(mesh: &mut Vec<Quad>) {
    let mut dest = 0;
    let mut back = 0;
    let mut lead = 0;

    loop {
        // 1: Advance lead
        // 2: Advance back
        // 3: Merge step
        // 4: End condition

        if lead == mesh.len() {
            if back != lead {
                // 2
                *unsafe { mesh.get_unchecked_mut(dest) } = *unsafe { mesh.get_unchecked(back) };
                dest += 1;
                back += 1;
                continue;
            } else {
                // 4
                break;
            }
        }

        let b = *unsafe { mesh.get_unchecked(back) };
        let l = *unsafe { mesh.get_unchecked(lead) };
        let d = unsafe { mesh.get_unchecked_mut(dest) };
        let Δy = l.y0 - b.y0 - b.height as u16;

        if Δy == 0 {
            // 1
            lead += 1;
        } else if Δy > 1 {
            // 2
            *d = b;
            dest += 1;
            back += 1;
        } else if b.x0 > l.x0 {
            // 1
            lead += 1;
        } else if l.x0 > b.x0 {
            // 2
            *d = b;
            dest += 1;
            back += 1;
        } else if b.width == l.width {
            // 3
            let new_height = b.height + 1;
            unsafe { mesh.get_unchecked_mut(lead) }.y0 -= new_height as u16;
            unsafe { mesh.get_unchecked_mut(lead) }.height = new_height;
            back += 1;
            lead += 1;
        } else {
            // 2
            *d = b;
            dest += 1;
            back += 1;
        }
    }

    mesh.truncate(dest);
}
