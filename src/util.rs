use entities::Loc;

pub fn bresenham<F>([mut x0, mut y0]: Loc, [x1, y1]: Loc, mut cb: F) where F: FnMut(i16, i16) -> bool {
    let dx = (x1 - x0).abs();
    let sx = if x0 < x1 { 1 } else { -1 };
    let dy = -1 * (y1 - y0).abs();
    let sy = if y0 < y1 { 1 } else { -1 };
    let mut e = dx + dy;

    loop {
        if cb(x0, y0) { break }
        if x0 == x1 && y0 == y1 { break }

        let e2 = 2 * e;
        if e2 >= dy {
            e += dy;
            x0 += sx;
        }
        if e2 <= dx {
            e += dx;
            y0 += sy;
        }
    }
}
