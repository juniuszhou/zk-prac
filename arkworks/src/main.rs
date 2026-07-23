#[cfg(test)]
mod tests;

mod custom_field;
mod fri;
mod mini_zk;

fn main() {
    // mini_zk::run_mini_zk_demo();
    fri::run_fri_demo();
}
