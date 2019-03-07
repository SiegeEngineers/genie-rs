pub use genie_cpx as cpx;
pub use genie_scx as scx;
pub use genie_hki as hki;

pub use genie_cpx::Campaign;
pub use genie_scx::Scenario;
pub use genie_hki::HotkeyInfo;

#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
        assert_eq!(2 + 2, 4);
    }
}
