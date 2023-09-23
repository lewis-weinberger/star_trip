/// A simple macro to expand an index,
/// given first the sector co-ordinate (x, y) and
/// second the system co-ordinate (X, Y)
#[macro_export]
macro_rules! index {
    ($x:expr, $y:expr, $X:expr, $Y:expr) => {
        $x + SECTORS * ($y + SECTORS * ($X + SYSTEMS * $Y))
    };

    ($p:ident) => {{
        let (x, y, xx, yy) = $p;
        x + SECTORS * (y + SECTORS * (xx + SYSTEMS * yy))
    }};
}

/// Convert numbers to ASCII byte strings (decimal representation)
/// and concatenate with provided byte strings.
/// Requires the `DisplayByte` trait to be implemented
///
/// # Examples
///
/// ```
/// let b = bconcat!(b"I am ", 45, b" years old!");
/// assert_eq!(b, b"I am 45 years old!");
/// ```
#[macro_export]
macro_rules! bconcat {
    ($($e:expr),*) => {
        {
            &[$($e.display_bytes(),)*].concat()
        }
    };
}

/// Determine if an entity is nearby
#[macro_export]
macro_rules! nearby {
    ($state:ident, $($entity:tt),*) => {
        'nearby: {
            let (i, j, xx, yy) = $state.position;
            for (x, y) in adjacent(i, j) {
                match $state.galaxy[index!(x, y, xx, yy)] {
                    $(
                        Some(Entity::$entity) => {
                            break 'nearby Some((x, y, Entity::$entity));
                        },
                    )*
                    _ => ()
                }
            }
            None
        }
    }
}
