use willow::{Program, ProgramData};

#[derive(Program)]
#[willow(path = "foo")]
pub struct Foo {
    data: ProgramData,
}
