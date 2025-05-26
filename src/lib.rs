mod text;
mod style;
pub mod math;
mod id;
mod state;
mod ctx;
mod action;

pub use cosmic_text;

pub use id::Id;
pub use style::*;
pub use text::*;
pub use math::*;

pub fn add(left: u64, right: u64) -> u64 {
    left + right
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn it_works() {
        let result = add(2, 2);
        assert_eq!(result, 4);
    }
}
