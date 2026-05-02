#[derive(Clone, Debug, Eq, PartialEq)]
pub struct OtherParam {
    name: Vec<u8>,
    values: Vec<Vec<u8>>,
}

impl OtherParam {
    pub fn name(&self) -> &[u8] {
        &self.name
    }

    pub fn values(&self) -> &[Vec<u8>] {
        &self.values
    }

    pub fn parse(name: Vec<u8>, values: Vec<Vec<u8>>) -> Self {
        OtherParam { name, values }
    }

    pub fn new(name: Vec<u8>, values: Vec<Vec<u8>>) -> Self {
        OtherParam { name, values }
    }
}
