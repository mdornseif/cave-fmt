extern crate pest;
#[macro_use]
extern crate pest_derive;

// mod th2;
// pub use crate::th2::*;


mod xvi;
pub use crate::xvi::*;

#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
        let result = 2 + 2;
        assert_eq!(result, 4);
    }
}
