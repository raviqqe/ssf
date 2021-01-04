#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub enum Primitive {
    Float32,
    Float64,
    Integer8,
    Integer32,
    Integer64,
    PointerInteger,
}
