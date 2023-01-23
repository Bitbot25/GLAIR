use super::*;

impl Reserve {
    pub fn generate_amd64(&self) {
        todo!()
    }
}

impl Write {
    pub fn generate_amd64(&self, buf: &mut Vec<u8>) {
        let Self { destination, value } = self;
        let destination = destination.unwrap_as_machine_register().unwrap_as_amd64();
        let value = value.unwrap_as_machine_register().unwrap_as_amd64();
    }
}
